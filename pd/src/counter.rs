use std::os::raw::c_void;

use crate::cstr;
use crate::log;
use crate::wrapper;

static mut COUNTER_CLASS: Option<*mut pd_sys::_class> = None;

#[repr(C)]
struct Counter {
    pd_obj: pd_sys::t_object,
    count: f32,
}

unsafe extern "C" fn counter_new(initial_value: pd_sys::t_float) -> *mut c_void {
    let counter = pd_sys::pd_new(COUNTER_CLASS.unwrap()) as *mut Counter;

    (*counter).count = initial_value;

    pd_sys::outlet_new(&mut (*counter).pd_obj, &mut pd_sys::s_float);

    counter as *mut c_void
}

unsafe extern "C" fn counter_set(counter: *mut Counter, value: pd_sys::t_float) {
    (*counter).count = value;
}

unsafe extern "C" fn counter_bang(counter: *mut Counter) {
    let counter = &mut *counter;
    let count = counter.count;
    counter.count += 1.0;
    pd_sys::outlet_float(counter.pd_obj.te_outlet, count);
}

#[no_mangle]
pub unsafe extern "C" fn counter_setup() {
    log::info("[counter] initializing");

    let class = create_class();

    COUNTER_CLASS = Some(class);

    wrapper::register_bang_method(class, counter_bang);
    register_set_method(class);
}

unsafe fn create_class() -> *mut pd_sys::_class {
    pd_sys::class_new(
        pd_sys::gensym(cstr::cstr("counter").as_ptr()),
        Some(std::mem::transmute::<
            unsafe extern "C" fn(pd_sys::t_float) -> *mut ::std::os::raw::c_void,
            _,
        >(counter_new)),
        None,
        std::mem::size_of::<Counter>(),
        pd_sys::CLASS_DEFAULT as i32,
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    )
}

unsafe fn register_set_method(class: *mut pd_sys::_class) {
    pd_sys::class_addmethod(
        class,
        Some(std::mem::transmute::<
            unsafe extern "C" fn(*mut Counter, pd_sys::t_float),
            _,
        >(counter_set)),
        pd_sys::gensym(cstr::cstr("set").as_ptr()),
        pd_sys::t_atomtype::A_DEFFLOAT,
        0,
    );
}
