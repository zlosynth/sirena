use std::os::raw::{c_int, c_void};

use sirena_modules as modules;

use crate::cstr;
use crate::log;
use crate::time;

static mut DELAY_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Delay {
    pd_obj: pd_sys::t_object,
    delay_module: modules::delay::Delay,
    signal_dummy: f32,
}

fn perform(
    delay: &mut Delay,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    outlets[0].copy_from_slice(&inlets[0]);
    delay.delay_module.process(outlets[0]);
}

unsafe extern "C" fn delay_set_delay(delay: *mut Delay, value: pd_sys::t_float) {
    let frame_rate = pd_sys::sys_getsr();
    (*delay)
        .delay_module
        .set_delay(time::time_to_frames(value, frame_rate).min(modules::delay::MAX_DELAY));
}

unsafe extern "C" fn delay_new(initial_delay: pd_sys::t_float) -> *mut c_void {
    let delay = pd_sys::pd_new(DELAY_CLASS.unwrap()) as *mut Delay;

    (*delay).delay_module = modules::delay::Delay::new();
    let frame_rate = pd_sys::sys_getsr();
    (*delay)
        .delay_module
        .set_delay(time::time_to_frames(initial_delay, frame_rate).min(modules::delay::MAX_DELAY));

    pd_sys::outlet_new(&mut (*delay).pd_obj, &mut pd_sys::s_signal);

    delay as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn delay_setup() {
    let class = create_class();

    DELAY_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Delay,
        dummy_offset = offset_of!(Delay => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_delay_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[delay~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("delay~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float) -> *mut c_void,
            _,
        >(delay_new)),
        None,
        std::mem::size_of::<Delay>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_delay_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Delay, pd_sys::t_float),
            _,
        >(delay_set_delay)),
        pd_sys::gensym(cstr::cstr("delay").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}
