use crate::module_config::{ModuleConfigInfo, RackInput, RackOutput, StaticModuleConfig};
use crate::rack::{InputPort, OutputPort, Port};
use crate::util::gate;

const THRESHOLD: f32 = 12.0;

impl StaticModuleConfig for Breaker {
    const INPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Left", c"Right", c"Reset trigger"];

    const OUTPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Tripped gate", c"Left", c"Right"];
}

struct BreakerInputs<'a> {
    left: InputPort<'a>,
    right: InputPort<'a>,
    reset_trigger: InputPort<'a>,
}
impl RackInput for BreakerInputs<'_> {
    const COUNT: usize = 3;

    fn from_raw_ptr(ports: *const Port) -> Self {
        let in_port = |ptr: *const Port, index: usize| {
            debug_assert!(index < Self::COUNT);
            InputPort::from_raw_port_index(ptr, index)
        };
        let left = in_port(ports, 0);
        let right = in_port(ports, 1);
        let reset_trigger = in_port(ports, 2);
        BreakerInputs {
            left,
            right,
            reset_trigger,
        }
    }
}

struct BreakerOutputs<'a> {
    tripped_gate: OutputPort<'a>,
    left: OutputPort<'a>,
    right: OutputPort<'a>,
}
impl RackOutput for BreakerOutputs<'_> {
    const COUNT: usize = 3;

    fn from_raw_ptr(ports: *mut Port) -> Self {
        let out_port = |ptr: *mut Port, index: usize| {
            debug_assert!(index < Self::COUNT);
            OutputPort::from_raw_port_index(ptr, index)
        };
        let tripped_gate = out_port(ports, 0);
        let left = out_port(ports, 1);
        let right = out_port(ports, 2);
        BreakerOutputs {
            tripped_gate,
            left,
            right,
        }
    }
}

#[derive(Default)]
enum BreakerState {
    #[default]
    Closed,
    Open,
}

#[derive(Default)]
pub struct Breaker {
    state: BreakerState,
    reset_trigger: crate::util::InputTrigger,
}

impl Breaker {
    fn process(
        &mut self,
        inputs: &BreakerInputs,
        outputs: &mut BreakerOutputs,
        tripped_status: &mut bool,
    ) {
        use BreakerState::*;

        // If we received a reset trigger, close the breaker.
        let reset_trigger_voltage = inputs.reset_trigger.get_voltage_monophonic().unwrap_or(0.0);
        if self.reset_trigger.process_voltage(reset_trigger_voltage) {
            self.state = Closed;
        }

        // If any of our input channels has a value out of range, trip the
        // breaker.
        if let Closed = self.state {
            let tripped = out_of_range(inputs.left.as_slice(), THRESHOLD)
                || out_of_range(inputs.right.as_slice(), THRESHOLD);
            if tripped {
                self.state = Open;
            }
        }

        outputs.left.set_polyphony_from(&inputs.left);
        outputs.right.set_polyphony_from(&inputs.right);
        let l_in = inputs.left.as_slice();
        let r_in = inputs.right.as_slice();
        let l_out = outputs.left.as_slice_mut();
        let r_out = outputs.right.as_slice_mut();

        match self.state {
            Closed => {
                copy_values(l_in, l_out);
                copy_values(r_in, r_out);
                outputs.tripped_gate.set_voltage_monophonic(gate::LOW);
                *tripped_status = false;
            }
            Open => {
                copy_muted(l_in, l_out);
                copy_muted(r_in, r_out);
                outputs.tripped_gate.set_voltage_monophonic(gate::HIGH);
                *tripped_status = true;
            }
        }
    }

    pub fn process_raw(
        &mut self,
        inputs: *const Port,
        outputs: *mut Port,
        tripped_status: &mut bool,
    ) {
        let inputs = BreakerInputs::from_raw_ptr(inputs);
        let mut outputs: BreakerOutputs = BreakerOutputs::from_raw_ptr(outputs);
        self.process(&inputs, &mut outputs, tripped_status)
    }

    pub fn get_module_config_info(&self) -> *mut ModuleConfigInfo {
        ModuleConfigInfo::from_module_instance(self).into_ptr()
    }
}

fn out_of_range(values: &[f32], threshold: f32) -> bool {
    // Note that depending on exactly how we do this comparison, NaNs may or may
    // not trigger it. Currently we opt to let NaNs through without triggering.
    values.iter().any(|n| n.abs() >= threshold)
}

fn copy_values(src: &[f32], dest: &mut [f32]) {
    let n = src.len();
    dest[..n].copy_from_slice(src);
}

fn copy_muted(src: &[f32], dest: &mut [f32]) {
    let n = src.len();
    dest[..n].fill(0.0);
}
