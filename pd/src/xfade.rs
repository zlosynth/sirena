use std::os::raw::{c_int, c_void};

use crate::cstr;
use crate::log;
use crate::numbers::Pin;

static mut XFADE_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct XFade {
    pd_obj: pd_sys::t_object,
    in2_inlet: *mut pd_sys::_inlet,
    ratio_inlet: *mut pd_sys::_inlet,
    ratio: f32,
    signal_dummy: f32,
}

fn perform(
    xfade: &mut XFade,
    _number_of_frames: usize,
    inlets: &[&mut [pd_sys::t_float]],
    outlets: &mut [&mut [pd_sys::t_float]],
) {
    let ratio = xfade.ratio.pin(0.0, 1.0);

    for (out, (in1, in2)) in outlets[0]
        .iter_mut()
        .zip(inlets[0].iter().zip(inlets[1].iter()))
    {
        *out = *in1 * (1.0 - ratio) + *in2 * ratio;
    }
}

unsafe extern "C" fn xfade_new(initial_ratio: pd_sys::t_float) -> *mut c_void {
    let xfade = pd_sys::pd_new(XFADE_CLASS.unwrap()) as *mut XFade;

    (*xfade).ratio = initial_ratio;

    (*xfade).in2_inlet = pd_sys::inlet_new(
        &mut (*xfade).pd_obj,
        &mut (*xfade).pd_obj.te_g.g_pd,
        &mut pd_sys::s_signal,
        &mut pd_sys::s_signal,
    );
    (*xfade).ratio_inlet = pd_sys::floatinlet_new(&mut (*xfade).pd_obj, &mut (*xfade).ratio);
    pd_sys::outlet_new(&mut (*xfade).pd_obj, &mut pd_sys::s_signal);

    xfade as *mut c_void
}

#[no_mangle]
pub unsafe extern "C" fn xfade_setup() {
    let class = create_class();

    XFADE_CLASS = Some(class);

    register_dsp_method!(
        class,
        receiver = XFade,
        dummy_offset = offset_of!(XFade => signal_dummy),
        number_of_inlets = 2,
        number_of_outlets = 1,
        callback = perform
    );
}

unsafe fn create_class() -> *mut pd_sys::_class {
    log::info("[xfade~] initializing");

    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("xfade~").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float) -> *mut c_void,
            _,
        >(xfade_new)),
        None,
        std::mem::size_of::<XFade>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}
