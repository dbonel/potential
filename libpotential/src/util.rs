// Reference voltages for gate outputs
pub mod gate {
    pub const LOW: f32 = 0.0;
    pub const HIGH: f32 = 10.0;
}

#[derive(Debug, Default)]
enum InputTriggerState {
    #[default]
    Low,
    High,
}

// Generates trigger events from an input port's voltage.
#[derive(Debug, Default)]
pub struct InputTrigger {
    state: InputTriggerState,
}

impl InputTrigger {
    const LOW_THRESHOLD: f32 = 0.1;
    const HIGH_THRESHOLD: f32 = 1.0;

    pub fn new() -> Self {
        InputTrigger {
            state: InputTriggerState::Low,
        }
    }

    pub fn process_voltage(&mut self, value: f32) -> bool {
        use InputTriggerState::*;
        match self.state {
            Low => {
                let triggered = value >= Self::HIGH_THRESHOLD;
                if triggered {
                    self.state = High;
                }
                triggered
            }
            High => {
                if value <= Self::LOW_THRESHOLD {
                    self.state = Low;
                }
                false
            }
        }
    }

    pub fn reset(&mut self) {
        self.state = InputTriggerState::Low;
    }
}
