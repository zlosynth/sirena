use std::os::raw::{c_int, c_void};

use sirena_modules as modules;

use crate::cstr;
use crate::log;
use crate::numbers::Pin;
use crate::time;

static mut COMB_FILTER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct CombFilter {
    pd_obj: pd_sys::t_object,
    filter_module: modules::comb_filter::CombFilter,
    signal_dummy: f32,
}

fn perform(
    delay: &mut CombFilter,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    outlets[0].copy_from_slice(&inlets[0]);
    delay.filter_module.process(outlets[0]);
}

unsafe extern "C" fn set_delay(comb_filter: *mut CombFilter, value: pd_sys::t_float) {
    let frame_rate = pd_sys::sys_getsr();
    (*comb_filter)
        .filter_module
        .set_delay(time::time_to_frames(value, frame_rate));
}

unsafe extern "C" fn set_gain(comb_filter: *mut CombFilter, value: pd_sys::t_float) {
    let value = value.pin(0.0, 0.999);
    (*comb_filter).filter_module.set_gain(value);
}

unsafe extern "C" fn new(
    initial_delay: pd_sys::t_float,
    initial_gain: pd_sys::t_float,
) -> *mut c_void {
    let comb_filter = pd_sys::pd_new(COMB_FILTER_CLASS.unwrap()) as *mut CombFilter;

    (*comb_filter).filter_module = modules::comb_filter::CombFilter::new();
    let frame_rate = pd_sys::sys_getsr();
    (*comb_filter)
        .filter_module
        .set_delay(time::time_to_frames(initial_delay, frame_rate));
    let initial_gain = initial_gain.pin(0.0, 0.999);
    (*comb_filter).filter_module.set_gain(initial_gain);

    pd_sys::outlet_new(&mut (*comb_filter).pd_obj, &mut pd_sys::s_signal);

    comb_filter as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn setup() {
    let class = create_class();

    COMB_FILTER_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = CombFilter,
        dummy_offset = offset_of!(CombFilter => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_delay_method(class);
    register_set_gain_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[combfilter~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("combfilter~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float, pd_sys::t_float) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<CombFilter>(),
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
            unsafe extern "C" fn(*mut CombFilter, pd_sys::t_float),
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
            unsafe extern "C" fn(*mut CombFilter, pd_sys::t_float),
            _,
        >(set_gain)),
        pd_sys::gensym(cstr::cstr("gain").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}
