use crate::rack::Port;

pub trait RackInput: Sized {
    const COUNT: usize;
    fn get_name(index: usize) -> &'static str;
    fn from_raw_ptr(ports: *const Port) -> Self;
}

pub trait RackOutput: Sized {
    const COUNT: usize;
    fn get_name(index: usize) -> &'static str;
    fn from_raw_ptr(ports: *mut Port) -> Self;
}

// A default type for a module with no input ports.
pub struct NoInputs {}
impl RackInput for NoInputs {
    const COUNT: usize = 0;

    fn get_name(_index: usize) -> &'static str {
        "none"
    }

    fn from_raw_ptr(_ports: *const Port) -> Self {
        NoInputs {}
    }
}

// A default type for a module with no output ports.
pub struct NoOutputs {}
impl RackOutput for NoOutputs {
    const COUNT: usize = 0;

    fn get_name(_index: usize) -> &'static str {
        "none"
    }

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

        fn get_name(index: usize) -> &'static str {
            match index {
                0 => "One",
                _ => unreachable!(),
            }
        }

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
        let voltages = inputs.one.as_slice();
        let voltages = Vec::from(voltages);
        voltages
    }
}
