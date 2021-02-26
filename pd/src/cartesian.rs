use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;

static mut CARTESIAN_CLASS: Option<*mut pd_sys::_class> = None;

lazy_static! {
    static ref WAVETABLE_A: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let sine_wave = sirena::wavetable_oscillator::sine();
        sirena::wavetable_oscillator::Wavetable::new(sine_wave, sample_rate)
    };
    static ref WAVETABLE_B: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let saw_wave = sirena::wavetable_oscillator::triangle();
        sirena::wavetable_oscillator::Wavetable::new(saw_wave, sample_rate)
    };
    static ref WAVETABLE_C: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let saw_wave = sirena::wavetable_oscillator::saw();
        sirena::wavetable_oscillator::Wavetable::new(saw_wave, sample_rate)
    };
}

#[repr(C)]
struct Cartesian<'a> {
    pd_obj: pd_sys::t_object,
    cartesian_module: sirena::cartesian::Cartesian<'a>,
    signal_dummy: f32,
}

fn perform(
    cartesian: &mut Cartesian,
    _number_of_frames: usize,
    _inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    for x in outlets[0].iter_mut() {
        *x = cartesian.cartesian_module.tick();
    }
}

unsafe extern "C" fn set_frequency(cartesian: *mut Cartesian, value: pd_sys::t_float) {
    (*cartesian).cartesian_module.set_frequency(value);
}

unsafe extern "C" fn set_x(cartesian: *mut Cartesian, value: pd_sys::t_float) {
    (*cartesian).cartesian_module.set_x(value);
}

unsafe extern "C" fn set_y(cartesian: *mut Cartesian, value: pd_sys::t_float) {
    (*cartesian).cartesian_module.set_y(value);
}

unsafe extern "C" fn set_enabled_voices(cartesian: *mut Cartesian, value: pd_sys::t_float) {
    let enabled_voices = f32::max(f32::min(value, 7.0), 1.0) as u32;
    (*cartesian).cartesian_module.set_enabled_voices(enabled_voices);
}

unsafe extern "C" fn set_detune(cartesian: *mut Cartesian, value: pd_sys::t_float) {
    (*cartesian).cartesian_module.set_detune(value);
}

unsafe extern "C" fn new() -> *mut c_void {
    let cartesian = pd_sys::pd_new(CARTESIAN_CLASS.unwrap()) as *mut Cartesian;

    let sample_rate = pd_sys::sys_getsr() as u32;
    let cartesian_module =
        sirena::cartesian::Cartesian::new(&WAVETABLE_A, &WAVETABLE_B, &WAVETABLE_C, sample_rate);

    (*cartesian).cartesian_module = cartesian_module;

    pd_sys::outlet_new(&mut (*cartesian).pd_obj, &mut pd_sys::s_signal);

    cartesian as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    CARTESIAN_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Cartesian,
        dummy_offset = offset_of!(Cartesian => signal_dummy),
        number_of_inlets = 0,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_frequency_method(class);
    register_set_x_method(class);
    register_set_y_method(class);
    register_set_enabled_voices_method(class);
    register_set_detune_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[cartesian~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("cartesian~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn() -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<Cartesian>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe fn register_set_frequency_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Cartesian, pd_sys::t_float),
            _,
        >(set_frequency)),
        &mut pd_sys::s_float,
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_x_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Cartesian, pd_sys::t_float),
            _,
        >(set_x)),
        pd_sys::gensym(cstr::cstr("x").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_y_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Cartesian, pd_sys::t_float),
            _,
        >(set_y)),
        pd_sys::gensym(cstr::cstr("y").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_enabled_voices_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Cartesian, pd_sys::t_float),
            _,
        >(set_enabled_voices)),
        pd_sys::gensym(cstr::cstr("voices").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_detune_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Cartesian, pd_sys::t_float),
            _,
        >(set_detune)),
        pd_sys::gensym(cstr::cstr("detune").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}
