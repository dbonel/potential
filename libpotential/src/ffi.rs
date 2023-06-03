#[cxx::bridge(namespace = "rustlib")]
pub mod bridge {
    extern "Rust" {}
}

// Generic helper to avoid too much boilerplate in FFI _new functions
fn new_default_raw<T>() -> *mut T
where
    T: Default,
{
    let t = T::default();
    let b = Box::new(t);
    Box::into_raw(b)
}

// Another generic helper to avoid boilerplate in FFI _drop functions
fn drop_raw<T>(ptr: *mut T) {
    assert!(!ptr.is_null());
    let b = unsafe { Box::from_raw(ptr) };
    drop(b);
}
