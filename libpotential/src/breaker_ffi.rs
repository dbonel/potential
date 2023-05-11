use std::ffi::c_int;

use crate::breaker::Breaker;

#[no_mangle]
pub unsafe extern "C" fn breaker_new() -> *mut Breaker {
    let breaker = Breaker::default();
    let breaker = Box::new(breaker);
    Box::into_raw(breaker)
}

#[no_mangle]
pub unsafe extern "C" fn breaker_free(breaker: *mut Breaker) {
    assert_eq!(breaker.is_null(), false);
    let breaker = unsafe { Box::from_raw(breaker) };
    drop(breaker)
}

#[no_mangle]
pub unsafe extern "C" fn breaker_process(
    breaker: *mut Breaker,
    left_in: *const f32,
    left_in_len: c_int,
    right_in: *const f32,
    right_in_len: c_int,
    reset_in: f32,
    tripped_status: *mut bool,
    tripped_gate: *mut f32,
    left_out: *mut f32,
    left_out_len: c_int,
    right_out: *mut f32,
    right_out_len: c_int,
) {
    debug_assert_eq!(breaker.is_null(), false);
    debug_assert_eq!(left_in.is_null(), false);
    debug_assert_eq!(right_in.is_null(), false);
    debug_assert_eq!(tripped_status.is_null(), false);
    debug_assert_eq!(tripped_gate.is_null(), false);
    debug_assert_eq!(left_out.is_null(), false);
    debug_assert_eq!(right_out.is_null(), false);

    let left_in_len = left_in_len as usize;
    let right_in_len = right_in_len as usize;
    let left_out_len = left_out_len as usize;
    let right_out_len = right_out_len as usize;

    let breaker = unsafe { &mut *breaker };
    let left_in = unsafe { std::slice::from_raw_parts(left_in, left_in_len) };
    let right_in = unsafe { std::slice::from_raw_parts(right_in, right_in_len) };
    let tripped_status = unsafe { &mut *tripped_status };
    let tripped_gate = unsafe { &mut *tripped_gate };
    let left_out = unsafe { std::slice::from_raw_parts_mut(left_out, left_out_len) };
    let right_out = unsafe { std::slice::from_raw_parts_mut(right_out, right_out_len) };

    breaker.process(
        left_in,
        right_in,
        reset_in,
        tripped_status,
        tripped_gate,
        left_out,
        right_out,
    )
}
