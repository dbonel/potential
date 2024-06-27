use std::ffi::{c_char, CStr};

use crate::rack::Port;

// This trait allows a module to describe its Rack IO configuration (inputs,
// outputs, etc.) at runtime. The return types are intended to be compatible-ish
// with the C++ side.
pub trait ModuleConfig {
    fn get_input_port_count(&self) -> usize;
    fn get_input_port_name(&self, index: usize) -> &'static CStr;
    fn get_output_port_count(&self) -> usize;
    fn get_output_port_name(&self, index: usize) -> &'static CStr;
}

// Modules will probably want to implement this instead of manually implementing
// the ModuleConfig trait.
pub trait StaticModuleConfig {
    const INPUT_PORTS: &'static [&'static CStr] = &[];
    const OUTPUT_PORTS: &'static [&'static CStr] = &[];
}

impl<T> ModuleConfig for T
where
    T: StaticModuleConfig,
{
    fn get_input_port_count(&self) -> usize {
        Self::INPUT_PORTS.len()
    }

    fn get_input_port_name(&self, index: usize) -> &'static CStr {
        assert!(index < Self::INPUT_PORTS.len());
        Self::INPUT_PORTS[index]
    }

    fn get_output_port_count(&self) -> usize {
        Self::OUTPUT_PORTS.len()
    }

    fn get_output_port_name(&self, index: usize) -> &'static CStr {
        assert!(index < Self::OUTPUT_PORTS.len());
        Self::OUTPUT_PORTS[index]
    }
}

// This type carries all of the same information we could get through a module's
// ModuleConfig interface, but as a standalone version optimized for use through
// the FFI.
pub struct ModuleConfigInfo {
    input_port_names: Vec<&'static CStr>,
    output_port_names: Vec<&'static CStr>,
}

impl ModuleConfigInfo {
    // Construct a ModuleConfigInfo from anything that implements ModuleConfig,
    // making a copy of all the module's config information.
    pub fn from_module_instance<T: ModuleConfig>(module: &T) -> Self {
        let input_port_count = module.get_input_port_count();
        let input_port_names = (0..input_port_count)
            .map(|index| module.get_input_port_name(index))
            .collect();
        let output_port_count = module.get_output_port_count();
        let output_port_names = (0..output_port_count)
            .map(|index| module.get_output_port_name(index))
            .collect();
        Self {
            input_port_names,
            output_port_names,
        }
    }

    // We use the return value of this to pass through the FFI.
    pub fn into_ptr(self) -> *mut Self {
        Box::into_raw(Box::new(self))
    }

    pub fn get_input_port_count(&self) -> usize {
        self.input_port_names.len()
    }

    pub fn get_output_port_count(&self) -> usize {
        self.output_port_names.len()
    }

    pub fn get_input_port_name(&self, index: usize) -> *const c_char {
        self.input_port_names[index].as_ptr()
    }

    pub fn get_output_port_name(&self, index: usize) -> *const c_char {
        self.output_port_names[index].as_ptr()
    }
}

pub trait RackInput: Sized {
    const COUNT: usize;
    fn from_raw_ptr(ports: *const Port) -> Self;
}

pub trait RackOutput: Sized {
    const COUNT: usize;
    fn from_raw_ptr(ports: *mut Port) -> Self;
}

// A default type for a module with no input ports.
pub struct NoInputs {}
impl RackInput for NoInputs {
    const COUNT: usize = 0;

    fn from_raw_ptr(_ports: *const Port) -> Self {
        NoInputs {}
    }
}

// A default type for a module with no output ports.
pub struct NoOutputs {}
impl RackOutput for NoOutputs {
    const COUNT: usize = 0;

    fn from_raw_ptr(_ports: *mut Port) -> Self {
        NoOutputs {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rack::{InputPort, OutputPort};

    pub struct TestInput<'a> {
        one: InputPort<'a>,
    }

    impl RackInput for TestInput<'_> {
        const COUNT: usize = 1;

        fn from_raw_ptr(ports: *const Port) -> Self {
            let one = InputPort::from_raw_port_index(ports, 0);
            TestInput { one }
        }
    }

    #[test]
    fn test_raw_roundtrip() {
        let mut p = vec![Port::default()];
        let test_voltages = [10.0f32];
        {
            let mut p = OutputPort::wrap(&mut p[0]);
            p.set_voltages_from_slice(test_voltages.as_slice());
        }
        let ptr: *mut Port = &mut p[0];
        let out_voltages = receiver(ptr);
        assert_eq!(test_voltages.as_slice(), out_voltages.as_slice());
    }

    fn receiver(inputs: *mut Port) -> Vec<f32> {
        let inputs = TestInput::from_raw_ptr(inputs);
        let voltages = inputs.one.as_slice().unwrap();
        let voltages = Vec::from(voltages);
        voltages
    }
}
