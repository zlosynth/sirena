use std::os::raw::{c_int, c_void};

use sirena_modules as modules;

use crate::cstr;
use crate::log;
use crate::numbers::Pin;
use crate::time;

static mut ALL_PASS_FILTER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct AllPassFilter {
    pd_obj: pd_sys::t_object,
    filter_module: modules::all_pass_filter::AllPassFilter,
    signal_dummy: f32,
}

fn perform(
    delay: &mut AllPassFilter,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    outlets[0].copy_from_slice(&inlets[0]);
    delay.filter_module.process(outlets[0]);
}

unsafe extern "C" fn set_delay(all_pass_filter: *mut AllPassFilter, value: pd_sys::t_float) {
    let frame_rate = pd_sys::sys_getsr();
    let frames = time::time_to_frames(value, frame_rate);
    (*all_pass_filter).filter_module.set_delay(frames);
}

unsafe extern "C" fn set_gain(all_pass_filter: *mut AllPassFilter, value: pd_sys::t_float) {
    let value = value.pin(0.0, 0.999);
    (*all_pass_filter).filter_module.set_gain(value);
}

unsafe extern "C" fn new(
    initial_delay: pd_sys::t_float,
    initial_gain: pd_sys::t_float,
) -> *mut c_void {
    let all_pass_filter = pd_sys::pd_new(ALL_PASS_FILTER_CLASS.unwrap()) as *mut AllPassFilter;

    (*all_pass_filter).filter_module = modules::all_pass_filter::AllPassFilter::new();

    let frame_rate = pd_sys::sys_getsr();
    let frames = time::time_to_frames(initial_delay, frame_rate);
    (*all_pass_filter).filter_module.set_delay(frames);

    let initial_gain = initial_gain.pin(0.0, 0.999);
    (*all_pass_filter).filter_module.set_gain(initial_gain);

    pd_sys::outlet_new(&mut (*all_pass_filter).pd_obj, &mut pd_sys::s_signal);

    all_pass_filter as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    ALL_PASS_FILTER_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = AllPassFilter,
        dummy_offset = offset_of!(AllPassFilter => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_delay_method(class);
    register_set_gain_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[allpassfilter~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("allpassfilter~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float, pd_sys::t_float) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<AllPassFilter>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_delay_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut AllPassFilter, pd_sys::t_float),
            _,
        >(set_delay)),
        pd_sys::gensym(cstr::cstr("delay").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}

unsafe fn register_set_gain_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut AllPassFilter, pd_sys::t_float),
            _,
        >(set_gain)),
        pd_sys::gensym(cstr::cstr("gain").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}
