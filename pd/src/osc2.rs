use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;

static mut OSC2_CLASS: Option<*mut pd_sys::_class> = None;

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
    static ref WAVETABLE_D: sirena::wavetable_oscillator::Wavetable = {
        let sample_rate = unsafe { pd_sys::sys_getsr() as u32 };
        let pulse_wave = sirena::wavetable_oscillator::pulse(0.5);
        sirena::wavetable_oscillator::Wavetable::new(pulse_wave, sample_rate)
    };
}

#[repr(C)]
struct Osc2<'a> {
    pd_obj: pd_sys::t_object,
    osc2_module: sirena::osc2::Osc2<'a>,
    signal_dummy: f32,
}

fn perform(
    osc2: &mut Osc2,
    _number_of_frames: usize,
    _inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    osc2.osc2_module.populate(&mut outlets[0]);
}

unsafe extern "C" fn set_frequency(osc2: *mut Osc2, value: pd_sys::t_float) {
    (*osc2).osc2_module.set_frequency(value);
}

unsafe extern "C" fn set_detune(osc2: *mut Osc2, value: pd_sys::t_float) {
    (*osc2).osc2_module.set_detune(value);
}

unsafe extern "C" fn set_wavetable(osc2: *mut Osc2, value: pd_sys::t_float) {
    (*osc2).osc2_module.set_wavetable(value);
}

unsafe extern "C" fn new() -> *mut c_void {
    let osc2 = pd_sys::pd_new(OSC2_CLASS.unwrap()) as *mut Osc2;

    let sample_rate = pd_sys::sys_getsr() as u32;
    let osc2_module = sirena::osc2::Osc2::new(
        [
            &WAVETABLE_A,
            &WAVETABLE_B,
            &WAVETABLE_C,
            &WAVETABLE_D,
            &WAVETABLE_A,
            &WAVETABLE_B,
            &WAVETABLE_C,
            &WAVETABLE_D,
        ],
        sample_rate,
    );

    (*osc2).osc2_module = osc2_module;

    pd_sys::outlet_new(&mut (*osc2).pd_obj, &mut pd_sys::s_signal);

    osc2 as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    OSC2_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Osc2,
        dummy_offset = offset_of!(Osc2 => signal_dummy),
        number_of_inlets = 0,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_frequency_method(class);
    register_set_detune_method(class);
    register_set_wavetable_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[osc2~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("osc2~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn() -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<Osc2>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe fn register_set_frequency_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Osc2, pd_sys::t_float),
            _,
        >(set_frequency)),
        &mut pd_sys::s_float,
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_detune_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Osc2, pd_sys::t_float),
            _,
        >(set_detune)),
        pd_sys::gensym(cstr::cstr("d").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_wavetable_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Osc2, pd_sys::t_float),
            _,
        >(set_wavetable)),
        pd_sys::gensym(cstr::cstr("w").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}
