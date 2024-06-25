use crate::module_config::ModuleConfigInfo;
use crate::rack::Port;

#[cxx::bridge(namespace = "rustlib")]
pub mod bridge {
    extern "Rust" {
        type Port;

        type ModuleConfigInfo;
        fn get_input_port_count(self: &ModuleConfigInfo) -> usize;
        fn get_input_port_name(self: &ModuleConfigInfo, index: usize) -> *const c_char;
        fn get_output_port_count(self: &ModuleConfigInfo) -> usize;
        fn get_output_port_name(self: &ModuleConfigInfo, index: usize) -> *const c_char;
        unsafe fn module_config_free(ptr: *mut ModuleConfigInfo);

        type Breaker;
        unsafe fn process_raw(
            self: &mut Breaker,
            inputs: *const Port,
            outputs: *mut Port,
            tripped_status: &mut bool,
        );
        fn breaker_new() -> *mut Breaker;
        unsafe fn breaker_free(ptr: *mut Breaker);

        unsafe fn mag_sign_process_raw(inputs: *const Port, outputs: *mut Port);

        type PolyShuffle;
        unsafe fn process_raw(self: &mut PolyShuffle, inputs: *const Port, outputs: *mut Port);
        fn get_module_config_info(self: &PolyShuffle) -> *mut ModuleConfigInfo;
        fn polyshuffle_new() -> *mut PolyShuffle;
        unsafe fn polyshuffle_free(ptr: *mut PolyShuffle);
    }
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

pub fn module_config_free(ptr: *mut ModuleConfigInfo) {
    drop_raw(ptr)
}

use crate::breaker::Breaker;
pub fn breaker_new() -> *mut Breaker {
    new_default_raw()
}
pub fn breaker_free(ptr: *mut Breaker) {
    drop_raw(ptr)
}

use crate::mag_sign::mag_sign_process_raw;

use crate::polyshuffle::PolyShuffle;
pub fn polyshuffle_new() -> *mut PolyShuffle {
    new_default_raw()
}
pub fn polyshuffle_free(ptr: *mut PolyShuffle) {
    drop_raw(ptr)
}
