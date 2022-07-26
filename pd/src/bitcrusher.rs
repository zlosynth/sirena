use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;

static mut BITCRUSHER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Bitcrusher {
    pd_obj: pd_sys::t_object,
    bitcrusher_module: sirena::bitcrusher::Bitcrusher,
    signal_dummy: f32,
}

fn perform(
    bitcrusher: &mut Bitcrusher,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    outlets[0].copy_from_slice(inlets[0]);
    bitcrusher.bitcrusher_module.process(outlets[0]);
}

unsafe extern "C" fn set_rate(bitcrusher: *mut Bitcrusher, value: pd_sys::t_float) {
    (*bitcrusher)
        .bitcrusher_module
        .set_rate((value as u32).max(1));
}

unsafe extern "C" fn set_resolution(bitcrusher: *mut Bitcrusher, value: pd_sys::t_float) {
    (*bitcrusher)
        .bitcrusher_module
        .set_resolution((value as i32).max(1).min(24));
}

unsafe extern "C" fn new() -> *mut c_void {
    let bitcrusher = pd_sys::pd_new(BITCRUSHER_CLASS.unwrap()) as *mut Bitcrusher;
    (*bitcrusher).bitcrusher_module = sirena::bitcrusher::Bitcrusher::new();

    pd_sys::outlet_new(&mut (*bitcrusher).pd_obj, &mut pd_sys::s_signal);

    bitcrusher as *mut c_void
}

pub unsafe extern "C" fn setup() {
    let class = create_class();

    BITCRUSHER_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = Bitcrusher,
        dummy_offset = offset_of!(Bitcrusher => signal_dummy),
        number_of_inlets = 1,
        number_of_outlets = 1,
        callback = perform
    );

    register_set_rate_method(class);
    register_set_resolution_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[bitcrusher~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("bitcrusher~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn() -> *mut c_void,
            _,
        >(new)),
        None,
        std::mem::size_of::<Bitcrusher>(),
        pd_sys::CLASS_DEFAULT as i32,
        0,
    )
}

unsafe fn register_set_rate_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Bitcrusher, pd_sys::t_float),
            _,
        >(set_rate)),
        pd_sys::gensym(cstr::cstr("rate").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}

unsafe fn register_set_resolution_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Bitcrusher, pd_sys::t_float),
            _,
        >(set_resolution)),
        pd_sys::gensym(cstr::cstr("resolution").as_ptr()),
        pd_sys::t_atomtype::A_FLOAT,
        0,
    );
}
