use std::os::raw::{c_int, c_void};

use sirena_modules as modules;

use crate::cstr;
use crate::log;

static mut STATE_VARIABLE_FILTER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct StateVariableFilter {
    pd_obj: pd_sys::t_object,
    filter_module: modules::state_variable_filter::StateVariableFilter,
    signal_dummy: f32,
}

fn perform(
    filter: &mut StateVariableFilter,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    outlets[0].copy_from_slice(&inlets[0]);
    filter.filter_module.process(outlets[0]);
}

unsafe extern "C" fn set_frequency(
    state_variable_filter: *mut StateVariableFilter,
    value: pd_sys::t_float,
) {
    (*state_variable_filter).filter_module.set_frequency(value);
}

unsafe extern "C" fn set_q_factor(
    state_variable_filter: *mut StateVariableFilter,
    value: pd_sys::t_float,
) {
    (*state_variable_filter).filter_module.set_q_factor(value);
}

unsafe extern "C" fn new(
    initial_frequency: pd_sys::t_float,
    initial_q_factor: pd_sys::t_float,
) -> *mut c_void {
    let state_variable_filter =
        pd_sys::pd_new(STATE_VARIABLE_FILTER_CLASS.unwrap()) as *mut StateVariableFilter;

    let frame_rate = pd_sys::sys_getsr();
    (*state_variable_filter).filter_module =
        modules::state_variable_filter::StateVariableFilter::new(frame_rate as u32);

    (*state_variable_filter)
        .filter_module
        .set_bandform(modules::state_variable_filter::LowPass)
        .set_frequency(initial_frequency)
        .set_q_factor(initial_q_factor);

    pd_sys::outlet_new(&mut (*state_variable_filter).pd_obj, &mut pd_sys::s_signal);

    state_variable_filter as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    STATE_VARIABLE_FILTER_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = StateVariableFilter,
        dummy_offset = offset_of!(StateVariableFilter => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_frequency_method(class);
    register_set_q_factor_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[statevariablefilter~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("statevariablefilter~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float, pd_sys::t_float) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<StateVariableFilter>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_frequency_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter, pd_sys::t_float),
            _,
        >(set_frequency)),
        pd_sys::gensym(cstr::cstr("frequency").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}

unsafe fn register_set_q_factor_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter, pd_sys::t_float),
            _,
        >(set_q_factor)),
        pd_sys::gensym(cstr::cstr("q").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}
