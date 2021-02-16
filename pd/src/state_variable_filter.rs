use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;

static mut STATE_VARIABLE_FILTER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct StateVariableFilter {
    pd_obj: pd_sys::t_object,
    filter_module: sirena::state_variable_filter::StateVariableFilter,
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

unsafe extern "C" fn set_lowpass(state_variable_filter: *mut StateVariableFilter) {
    (*state_variable_filter)
        .filter_module
        .set_bandform(sirena::state_variable_filter::LowPass);
}

unsafe extern "C" fn set_highpass(state_variable_filter: *mut StateVariableFilter) {
    (*state_variable_filter)
        .filter_module
        .set_bandform(sirena::state_variable_filter::HighPass);
}

unsafe extern "C" fn set_bandpass(state_variable_filter: *mut StateVariableFilter) {
    (*state_variable_filter)
        .filter_module
        .set_bandform(sirena::state_variable_filter::BandPass);
}

unsafe extern "C" fn set_bandreject(state_variable_filter: *mut StateVariableFilter) {
    (*state_variable_filter)
        .filter_module
        .set_bandform(sirena::state_variable_filter::BandReject);
}

unsafe extern "C" fn new(
    _name: *mut pd_sys::t_symbol,
    args_count: c_int,
    args: *const pd_sys::t_atom,
) -> *mut c_void {
    let state_variable_filter =
        pd_sys::pd_new(STATE_VARIABLE_FILTER_CLASS.unwrap()) as *mut StateVariableFilter;

    let frame_rate = pd_sys::sys_getsr();
    (*state_variable_filter).filter_module =
        sirena::state_variable_filter::StateVariableFilter::new(frame_rate as u32);

    let initial_q_factor = pd_sys::atom_getfloatarg(2, args_count, args);
    let initial_frequency = pd_sys::atom_getfloatarg(1, args_count, args);
    let initial_bandform = if args_count >= 1 {
        let args = std::slice::from_raw_parts(&args, 1);
        let bandform = pd_sys::atom_getsymbol(args[0]);
        if bandform == pd_sys::gensym(cstr::cstr("lowpass").as_ptr()) {
            sirena::state_variable_filter::LowPass
        } else if bandform == pd_sys::gensym(cstr::cstr("highpass").as_ptr()) {
            sirena::state_variable_filter::HighPass
        } else if bandform == pd_sys::gensym(cstr::cstr("bandpass").as_ptr()) {
            sirena::state_variable_filter::BandPass
        } else if bandform == pd_sys::gensym(cstr::cstr("bandreject").as_ptr()) {
            sirena::state_variable_filter::BandReject
        } else {
            sirena::state_variable_filter::LowPass
        }
    } else {
        sirena::state_variable_filter::LowPass
    };

    (*state_variable_filter)
        .filter_module
        .set_bandform(initial_bandform)
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
    register_set_lowpass_method(class);
    register_set_highpass_method(class);
    register_set_bandpass_method(class);
    register_set_bandreject_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[statevariablefilter~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("statevariablefilter~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(
                *mut pd_sys::t_symbol,
                c_int,
                *const pd_sys::t_atom,
            ) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<StateVariableFilter>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_GIMME,
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

unsafe fn register_set_lowpass_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter),
            _,
        >(set_lowpass)),
        pd_sys::gensym(cstr::cstr("lowpass").as_ptr()),
        0,
    );
}

unsafe fn register_set_highpass_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter),
            _,
        >(set_highpass)),
        pd_sys::gensym(cstr::cstr("highpass").as_ptr()),
        0,
    );
}

unsafe fn register_set_bandpass_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter),
            _,
        >(set_bandpass)),
        pd_sys::gensym(cstr::cstr("bandpass").as_ptr()),
        0,
    );
}

unsafe fn register_set_bandreject_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut StateVariableFilter),
            _,
        >(set_bandreject)),
        pd_sys::gensym(cstr::cstr("bandreject").as_ptr()),
        0,
    );
}
