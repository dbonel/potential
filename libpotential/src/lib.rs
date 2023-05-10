use std::ffi::c_int;

#[no_mangle]
pub unsafe extern "C" fn mag_sign_decompose(
    composed_inputs: *const f32,
    composed_inputs_len: c_int,
    magnitude_outputs: *mut f32,
    magnitude_outputs_len: c_int,
    sign_outputs: *mut f32,
    sign_outputs_len: c_int,
) {
    let composed = unsafe {
        debug_assert!(!composed_inputs.is_null());
        std::slice::from_raw_parts(composed_inputs, composed_inputs_len as usize)
    };
    let magnitude = unsafe {
        debug_assert!(!magnitude_outputs.is_null());
        std::slice::from_raw_parts_mut(magnitude_outputs, magnitude_outputs_len as usize)
    };
    let sign = unsafe {
        debug_assert!(!sign_outputs.is_null());
        std::slice::from_raw_parts_mut(sign_outputs, sign_outputs_len as usize)
    };
    mag_sign::decompose(composed, magnitude, sign)
}

#[no_mangle]
pub unsafe extern "C" fn mag_sign_recompose(
    magnitude_inputs: *const f32,
    magnitude_inputs_len: c_int,
    sign_inputs: *const f32,
    sign_inputs_len: c_int,
    composed_outputs: *mut f32,
    composed_outputs_len: c_int,
) {
    let magnitude = unsafe {
        debug_assert!(!magnitude_inputs.is_null());
        std::slice::from_raw_parts(magnitude_inputs, magnitude_inputs_len as usize)
    };
    let sign = unsafe {
        debug_assert!(!sign_inputs.is_null());
        std::slice::from_raw_parts(sign_inputs, sign_inputs_len as usize)
    };
    let composed = unsafe {
        debug_assert!(!composed_outputs.is_null());
        std::slice::from_raw_parts_mut(composed_outputs, composed_outputs_len as usize)
    };
    mag_sign::recompose(magnitude, sign, composed)
}

pub mod mag_sign;
pub mod util;
