use sirena_cmsis_dsp_sys as cmsis_dsp_sys;

pub fn abs_f32(src: &[f32], dst: &mut [f32]) {
    unsafe {
        cmsis_dsp_sys::arm_abs_f32(src.as_ptr(), dst.as_mut_ptr(), src.len() as u32);
    }
}
