#![deny(clippy::transmute_ptr_to_ptr)]
#![allow(clippy::missing_safety_doc)]

#[macro_use]
extern crate field_offset;

#[macro_use]
mod wrapper;

pub mod counter;
mod cstr;
mod log;
pub mod xfade;

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