use std::ffi::c_int;
use std::slice;

use crate::polyshuffle::PolyShuffle;

#[no_mangle]
pub unsafe extern "C" fn polyshuffle_new() -> *mut PolyShuffle {
    let polyshuffle = PolyShuffle::new();
    let polyshuffle = Box::new(polyshuffle);
    Box::into_raw(polyshuffle)
}

#[no_mangle]
pub unsafe extern "C" fn polyshuffle_free(polyshuffle: *mut PolyShuffle) {
    assert_eq!(polyshuffle.is_null(), false);
    let polyshuffle = unsafe { Box::from_raw(polyshuffle) };
    drop(polyshuffle)
}

#[no_mangle]
pub unsafe extern "C" fn polyshuffle_process(
    polyshuffle: *mut PolyShuffle,
    inputs: *const f32,
    inputs_len: c_int,
    outputs: *mut f32,
    outputs_len: c_int,
    shuffle_in: f32,
) -> c_int {
    debug_assert_eq!(polyshuffle.is_null(), false);
    debug_assert_eq!(inputs.is_null(), false);
    debug_assert_eq!(outputs.is_null(), false);

    let inputs_len = inputs_len as usize;
    let outputs_len = outputs_len as usize;

    let polyshuffle = unsafe { &mut *polyshuffle };
    let inputs = unsafe { slice::from_raw_parts(inputs, inputs_len) };
    let outputs = unsafe { slice::from_raw_parts_mut(outputs, outputs_len) };

    let output_count = polyshuffle.process(inputs, outputs, shuffle_in);
    output_count as c_int
}
