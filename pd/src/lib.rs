#![deny(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate field_offset;

#[macro_use]
extern crate lazy_static;

#[macro_use]
mod wrapper;

pub mod all_pass_filter;
pub mod cartesian;
pub mod comb_filter;
pub mod counter;
pub mod delay;
pub mod state_variable_filter;
pub mod wavetable_oscillator;
pub mod wavetable_oscillator_2;
pub mod xfade;

mod cstr;
mod log;
mod numbers;
mod time;

use std::os::raw::c_void;

static mut SIRENA_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Sirena {
    _pd_obj: pd_sys::t_object,
}

unsafe extern "C" fn sirena_new() -> *mut c_void {
    let counter = pd_sys::pd_new(SIRENA_CLASS.unwrap()) as *mut Sirena;

    counter as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn sirena_setup() {
    log::info("[sirena] initializing");

    let class = create_class();

    SIRENA_CLASS = Some(class);

    counter::counter_setup();
    xfade::xfade_setup();
    delay::delay_setup();
    comb_filter::setup();
    all_pass_filter::setup();
    wavetable_oscillator::setup();
    wavetable_oscillator_2::setup();
    state_variable_filter::setup();
    cartesian::setup();
}

unsafe fn create_class() -> *mut pd_sys::_class {
    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("sirena").as_ptr()),
        Some(sirena_new),
        None,
        std::mem::size_of::<Sirena>(),
        pd_sys::CLASS_NOINLET as i32,
        0,
    )
}
