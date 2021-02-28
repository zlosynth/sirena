use sirena::wavetable_oscillator::Oscillator as _;
use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;

static mut WAVETABLE_OSCILLATOR_2_CLASS: Option<*mut pd_sys::_class> = None;

lazy_static! {
    static ref WAVETABLE_A: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let sine_wave = sirena::wavetable_oscillator::sine();
        sirena::wavetable_oscillator::Wavetable::new(sine_wave, sample_rate)
    };
    static ref WAVETABLE_B: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let triangle_wave = sirena::wavetable_oscillator::triangle();
        sirena::wavetable_oscillator::Wavetable::new(triangle_wave, sample_rate)
    };
    static ref WAVETABLE_C: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let saw_wave = sirena::wavetable_oscillator::saw();
        sirena::wavetable_oscillator::Wavetable::new(saw_wave, sample_rate)
    };
}

#[repr(C)]
struct WavetableOscillator3<'a> {
    pd_obj: pd_sys::t_object,
    oscillator_module: sirena::wavetable_oscillator::CircularWavetableOscillator<'a>,
    signal_dummy: f32,
}

fn perform(
    wavetable_oscillator: &mut WavetableOscillator3,
    _number_of_frames: usize,
    _inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    wavetable_oscillator
        .oscillator_module
        .populate(&mut outlets[0]);
}

unsafe extern "C" fn set_frequency(
    wavetable_oscillator: *mut WavetableOscillator3,
    value: pd_sys::t_float,
) {
    (*wavetable_oscillator)
        .oscillator_module
        .set_frequency(value);
}

unsafe extern "C" fn new(initial_frequency: pd_sys::t_float) -> *mut c_void {
    let wavetable_oscillator =
        pd_sys::pd_new(WAVETABLE_OSCILLATOR_2_CLASS.unwrap()) as *mut WavetableOscillator3;

    let sample_rate = pd_sys::sys_getsr() as u32;
    let mut oscillator = sirena::wavetable_oscillator::CircularWavetableOscillator::new(
        [&WAVETABLE_A; 8],
        sample_rate,
    );
    oscillator.set_frequency(initial_frequency);
    (*wavetable_oscillator).oscillator_module = oscillator;

    pd_sys::outlet_new(&mut (*wavetable_oscillator).pd_obj, &mut pd_sys::s_signal);

    wavetable_oscillator as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    WAVETABLE_OSCILLATOR_2_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = WavetableOscillator3,
        dummy_offset = offset_of!(WavetableOscillator3 => signal_dummy),
        number_of_inlets = 0,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_frequency_method(class)
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[wavetableoscillator3~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("wavetableoscillator3~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float) -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<WavetableOscillator3>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_frequency_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut WavetableOscillator3, pd_sys::t_float),
            _,
        >(set_frequency)),
        &mut pd_sys::s_float,
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}
