use crate::module_config::{ModuleConfigInfo, RackInput, RackOutput, StaticModuleConfig};
use crate::rack::{InputPort, OutputPort, Port, PORT_MAX_CHANNELS};
use crate::util::gate;

const THRESHOLD: f32 = 12.0;

const ZEROES: [f32; PORT_MAX_CHANNELS] = [0.0; PORT_MAX_CHANNELS];

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
        let reset_trigger_voltage = inputs.reset_trigger.get_zero_normaled_monophonic_voltage();
        if self.reset_trigger.process_voltage(reset_trigger_voltage) {
            self.state = Closed;
        }

        // If any of our input channels has a value out of range, trip the
        // breaker.
        if matches!(self.state, Closed) {
            let left_in = inputs.left.as_slice();
            let right_in = inputs.right.as_slice();
            let tripped = left_in
                .map(|left| out_of_range(left, THRESHOLD))
                .unwrap_or(false)
                || right_in
                    .map(|right| out_of_range(right, THRESHOLD))
                    .unwrap_or(false);
            if tripped {
                self.state = Open;
            }
        }

        let mute = match self.state {
            Closed => {
                outputs.tripped_gate.set_monophonic_voltage(gate::LOW);
                *tripped_status = false;
                false
            }
            Open => {
                outputs.tripped_gate.set_monophonic_voltage(gate::HIGH);
                *tripped_status = true;
                true
            }
        };
        copy_or_mute(&inputs.left, &mut outputs.left, mute);
        copy_or_mute(&inputs.right, &mut outputs.right, mute);
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

fn copy_or_mute(src: &InputPort, dest: &mut OutputPort, mute: bool) {
    match src.as_slice() {
        Some(voltages) => {
            if !mute {
                dest.set_voltages_from_slice(voltages);
            } else {
                let zeroes = &ZEROES[..voltages.len()];
                dest.set_voltages_from_slice(zeroes);
            }
        }
        None => {
            dest.set_polyphony_count(0);
        }
    }
}
