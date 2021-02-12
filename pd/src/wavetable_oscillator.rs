use std::os::raw::{c_int, c_void};

use sirena_modules as modules;

use crate::cstr;
use crate::log;

static mut WAVETABLE_OSCILLATOR_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct WavetableOscillator {
    pd_obj: pd_sys::t_object,
    oscillator_module: modules::wavetable_oscillator::WavetableOscillator,
    signal_dummy: f32,
}

fn perform(
    wavetable_oscillator: &mut WavetableOscillator,
    _number_of_frames: usize,
    _inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    for x in outlets[0].iter_mut() {
        *x = wavetable_oscillator.oscillator_module.tick();
    }
}

unsafe extern "C" fn set_frequency(
    wavetable_oscillator: *mut WavetableOscillator,
    value: pd_sys::t_float,
) {
    (*wavetable_oscillator)
        .oscillator_module
        .set_frequency(value);
}

unsafe extern "C" fn new(initial_frequency: pd_sys::t_float) -> *mut c_void {
    let wavetable_oscillator =
        pd_sys::pd_new(WAVETABLE_OSCILLATOR_CLASS.unwrap()) as *mut WavetableOscillator;

    let sample_rate = pd_sys::sys_getsr() as u32;
    let saw_wave = modules::wavetable_oscillator::saw(256);
    let wavetable = modules::wavetable_oscillator::Wavetable::new(saw_wave);
    let mut oscillator =
        modules::wavetable_oscillator::WavetableOscillator::new(wavetable, sample_rate);
    oscillator.set_frequency(initial_frequency);
    (*wavetable_oscillator).oscillator_module = oscillator;

    pd_sys::outlet_new(&mut (*wavetable_oscillator).pd_obj, &mut pd_sys::s_signal);

    wavetable_oscillator as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    WAVETABLE_OSCILLATOR_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = WavetableOscillator,
        dummy_offset = offset_of!(WavetableOscillator => signal_dummy),
        number_of_inlets = 0,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_frequency_method(class)
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[wavetableoscillator~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("wavetableoscillator~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<WavetableOscillator>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_frequency_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut WavetableOscillator, pd_sys::t_float),
            _,
        >(set_frequency)),
        &mut pd_sys::s_float,
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}
