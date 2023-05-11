const THRESHOLD: f32 = 12.0;

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
    pub fn process(
        &mut self,
        left_in: &[f32],
        right_in: &[f32],
        reset_in: f32,
        tripped_status: &mut bool,
        tripped_gate: &mut f32,
        left_out: &mut [f32],
        right_out: &mut [f32],
    ) {
        use BreakerState::*;
        debug_assert!(left_in.len() <= left_out.len());
        debug_assert!(right_in.len() <= right_out.len());

        // If we received a reset trigger, close the breaker.
        if self.reset_trigger.process_voltage(reset_in) {
            self.state = Closed;
        }

        // If any of our input channels has a value out of range, trip the
        // breaker.
        match self.state {
            Closed => {
                let tripped = out_of_range(left_in, THRESHOLD) || out_of_range(right_in, THRESHOLD);
                if tripped {
                    self.state = Open;
                }
            }
            _ => {}
        }

        match self.state {
            Closed => {
                copy_values(left_in, left_out);
                copy_values(right_in, right_out);
                *tripped_gate = 0.0;
                *tripped_status = false;
            }
            Open => {
                copy_muted(left_in, left_out);
                copy_muted(right_in, right_out);
                *tripped_gate = 10.0;
                *tripped_status = true;
            }
        }
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
