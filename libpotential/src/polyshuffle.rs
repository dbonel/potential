use crate::module_config::{ModuleConfigInfo, RackInput, RackOutput, StaticModuleConfig};
use crate::rack::{InputPort, OutputPort, Port, PORT_MAX_CHANNELS};
use crate::util::InputTrigger;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

impl StaticModuleConfig for PolyShuffle {
    const INPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Polyphonic", c"Shuffle trigger"];

    const OUTPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Shuffled polyphonic"];
}

pub struct PolyShuffle {
    rng: SmallRng,
    shuffle_trigger: InputTrigger,
    // This is always 0..16 in some permutation. The current polyphony count
    // is probably smaller than this, so this is only used as a base for the
    // channel_destinations map.
    channel_destinations_full: [usize; PORT_MAX_CHANNELS],
    // Together, channel_count and channel_destinations are a cached subset
    // of channel_destinations_full. If channel_count is Some(n), then n is
    // the polyphony count, and channel_destinations[..n] will contain (0..n)
    // in some permutation. The elements of channel_destinations[n..] have no
    // guarantee of contents and should not be used.
    //
    // Setting channel_count to None marks the contents of channel_destinations
    // as invalid, to be lazily recomputed when it's next used. Cache
    // invalidations should be done when we get a shuffle trigger event, or when
    // the polyphony count changes.
    channel_count: Option<usize>,
    channel_destinations: [usize; PORT_MAX_CHANNELS],
}

impl PolyShuffle {
    pub fn new() -> Self {
        // This seed is just 8 bytes sampled from /dev/urandom.
        let rng = SmallRng::seed_from_u64(0xeafcf19c4c7cd3ac);
        let channel_destinations_full: [usize; PORT_MAX_CHANNELS] = core::array::from_fn(|n| n);
        let channel_destinations = channel_destinations_full;
        PolyShuffle {
            rng,
            shuffle_trigger: InputTrigger::new(),
            channel_destinations_full,
            channel_destinations,
            channel_count: None,
        }
    }

    fn process(&mut self, inputs: &PolyShuffleInput, outputs: &mut PolyShuffleOutput) {
        let trigger_voltage = inputs
            .shuffle_trigger
            .get_zero_normaled_monophonic_voltage();
        if self.shuffle_trigger.process_voltage(trigger_voltage) {
            self.shuffle();
        }

        if let Some(input_voltages) = inputs.poly.as_slice() {
            let channel_count = input_voltages.len();
            let destinations = self.get_channel_destinations(channel_count);
            let mut output_buffer = [0.0; PORT_MAX_CHANNELS];
            // Copy inputs to output buffer according to their mapped destinations.
            input_voltages.iter().enumerate().for_each(|(i, value)| {
                let destination_index = destinations[i];
                output_buffer[destination_index] = *value;
            });
            outputs
                .shuffled_poly
                .set_voltages_from_slice(&output_buffer[..channel_count]);
        } else {
            outputs.shuffled_poly.set_polyphony_count(0);
        }
    }

    pub fn process_raw(&mut self, inputs: *const Port, outputs: *mut Port) {
        let inputs = PolyShuffleInput::from_raw_ptr(inputs);
        let mut outputs = PolyShuffleOutput::from_raw_ptr(outputs);
        self.process(&inputs, &mut outputs)
    }

    // Shuffle the channel_destinations_full array.
    fn shuffle(&mut self) {
        self.channel_destinations_full
            .as_mut_slice()
            .shuffle(&mut self.rng);
        self.invalidate_channel_destinations();
    }

    // Mark the channel_destinations cache as invalid, forcing it to be
    // regenerated next time it's needed.
    fn invalidate_channel_destinations(&mut self) {
        self.channel_count = None;
    }

    // Get the cached channel_destinations array as a slice, regenerating it
    // first if needed.
    fn get_channel_destinations(&mut self, channel_count: usize) -> &[usize] {
        assert!(channel_count <= self.channel_destinations_full.len());
        match self.channel_count {
            Some(c) if c == channel_count => &self.channel_destinations[..channel_count],
            _ => {
                let n = channel_count;
                // Select just the output destinations that fit inside our channel count.
                let subset = self
                    .channel_destinations_full
                    .as_slice()
                    .iter()
                    .copied()
                    .filter(|v| *v < n);
                let channel_destinations = &mut self.channel_destinations[..n];
                channel_destinations
                    .iter_mut()
                    .zip(subset)
                    .for_each(|(o, i)| {
                        *o = i;
                    });
                self.channel_count = Some(channel_count);
                &self.channel_destinations[..channel_count]
            }
        }
    }

    pub fn get_module_config_info(&self) -> *mut ModuleConfigInfo {
        ModuleConfigInfo::from_module_instance(self).into_ptr()
    }

    // This is handy for debugging.
    #[allow(dead_code)]
    fn get_destinations(&self) -> &[usize] {
        &self.channel_destinations[..self.channel_count.unwrap_or_default()]
    }
}

impl Default for PolyShuffle {
    fn default() -> Self {
        PolyShuffle::new()
    }
}

struct PolyShuffleInput<'a> {
    poly: InputPort<'a>,
    shuffle_trigger: InputPort<'a>,
}
impl RackInput for PolyShuffleInput<'_> {
    const COUNT: usize = 2;

    fn from_raw_ptr(ports: *const Port) -> Self {
        let poly = InputPort::from_raw_port_index(ports, 0);
        let shuffle_trigger = InputPort::from_raw_port_index(ports, 1);
        PolyShuffleInput {
            poly,
            shuffle_trigger,
        }
    }
}

struct PolyShuffleOutput<'a> {
    shuffled_poly: OutputPort<'a>,
}
impl RackOutput for PolyShuffleOutput<'_> {
    const COUNT: usize = 1;

    fn from_raw_ptr(ports: *mut Port) -> Self {
        let shuffled_poly = OutputPort::from_raw_port_index(ports, 0);
        PolyShuffleOutput { shuffled_poly }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::util::gate;

    #[test]
    fn test_shuffle() {
        let mut p = PolyShuffle::new();
        let mut i1 = Port::default();
        let mut o1 = Port::default();
        let mut t_low = Port::default();
        let mut t_high = Port::default();
        let initial_voltages = vec![
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5,
        ];

        // Q: What's with all the scopes? A: There are a bunch of data types
        // that borrow our longer-lived ports, and it's convenient to be able to
        // throw them all away once they've done what we need.

        {
            // use the input as an output so we can shove some known voltages in there
            let mut temp_out = OutputPort::wrap(&mut i1);
            temp_out.set_voltages_from_slice(initial_voltages.as_slice());
            // set the voltages for low and high trigger states
            let mut t_low = OutputPort::wrap(&mut t_low);
            let mut t_high = OutputPort::wrap(&mut t_high);
            t_low.set_monophonic_voltage(gate::LOW);
            t_high.set_monophonic_voltage(gate::HIGH);
        }

        // Test the initial state (channel permutation is unmodified).
        {
            {
                let inputs = PolyShuffleInput {
                    poly: InputPort::wrap(&i1),
                    shuffle_trigger: InputPort::wrap(&t_low),
                };
                let mut outputs = PolyShuffleOutput {
                    shuffled_poly: OutputPort::wrap(&mut o1),
                };
                p.process(&inputs, &mut outputs);
            }
            let i1 = InputPort::wrap(&i1);
            let o1 = InputPort::wrap(&o1);
            assert_eq!(i1.as_slice(), o1.as_slice());
        }

        // Again, with the trigger input high this time.
        {
            {
                let inputs = PolyShuffleInput {
                    poly: InputPort::wrap(&i1),
                    shuffle_trigger: InputPort::wrap(&t_high),
                };
                let mut outputs = PolyShuffleOutput {
                    shuffled_poly: OutputPort::wrap(&mut o1),
                };
                p.process(&inputs, &mut outputs);
            }

            {
                // Test the shuffle result; the output order should be different from
                // before (unless we got extremely unlucky with the permutation).
                let i1 = InputPort::wrap(&i1);
                let o1 = InputPort::wrap(&o1);
                assert_ne!(i1.as_slice(), o1.as_slice());
            }

            {
                let i1 = InputPort::wrap(&i1);
                let mut o1 = OutputPort::wrap(&mut o1);
                // However, we should still be able to sort our voltages back to their original order without
                // any going missing.
                o1.as_slice_mut()
                    .unwrap()
                    // Whenever .sort_floats() stabilizes, use that instead.
                    // https://github.com/rust-lang/rust/issues/93396
                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                assert_eq!(i1.as_slice().unwrap(), o1.as_slice_mut().unwrap());
            }
        }

        // Test the polyphony count changing. The shuffle trigger is low again.
        {
            {
                let n: usize = 8;
                let mut temp_out = OutputPort::wrap(&mut i1);
                temp_out.set_polyphony_count(n);
            }
            {
                let inputs = PolyShuffleInput {
                    poly: InputPort::wrap(&i1),
                    shuffle_trigger: InputPort::wrap(&t_low),
                };
                let mut outputs = PolyShuffleOutput {
                    shuffled_poly: OutputPort::wrap(&mut o1),
                };
                p.process(&inputs, &mut outputs);
            }

            let i1 = InputPort::wrap(&i1);
            let mut o1 = OutputPort::wrap(&mut o1);
            assert_eq!(
                i1.as_slice().unwrap().len(),
                o1.as_slice_mut().unwrap().len()
            );
            assert_ne!(i1.as_slice().unwrap(), o1.as_slice_mut().unwrap());

            // This should also pass the re-sorting test.
            o1.as_slice_mut()
                .unwrap()
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(i1.as_slice().unwrap(), o1.as_slice_mut().unwrap());
        }
    }
}
