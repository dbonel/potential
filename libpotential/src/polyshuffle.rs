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
    channel_destinations: [usize; PORT_MAX_CHANNELS],
    channel_count: usize,
}

impl PolyShuffle {
    pub fn new() -> Self {
        // This seed is just 8 bytes from /dev/urandom.
        let rng = SmallRng::seed_from_u64(0xeafcf19c4c7cd3ac);
        let channel_destinations_full: [usize; PORT_MAX_CHANNELS] = core::array::from_fn(|n| n);
        let channel_destinations = channel_destinations_full.clone();
        let channel_count = 0;
        PolyShuffle {
            rng,
            shuffle_trigger: InputTrigger::new(),
            channel_destinations_full,
            channel_destinations,
            channel_count,
        }
    }

    fn process(&mut self, inputs: &PolyShuffleInput, outputs: &mut PolyShuffleOutput) {
        let channel_count = inputs.poly.get_polyphony_count();

        let trigger_voltage = inputs
            .shuffle_trigger
            .get_voltage_monophonic()
            .unwrap_or(0.0);
        if self.shuffle_trigger.process_voltage(trigger_voltage) {
            // .shuffle() will call .resize(), so channel_count should be
            // updated before calling it so we don't waste work.
            self.channel_count = channel_count;
            self.shuffle();
        }

        // Handle input polyphony count changing.
        if channel_count != self.channel_count {
            self.channel_count = channel_count;
            self.resize();
        }

        outputs.shuffled_poly.set_polyphony_from(&inputs.poly);

        // Copy inputs to outputs according to their mapped destinations.
        let orig = inputs.poly.as_slice();
        let shuffled = outputs.shuffled_poly.as_slice_mut();
        debug_assert_eq!(orig.len(), shuffled.len());
        orig.iter().enumerate().for_each(|(i, value)| {
            let destination = self.channel_destinations[i];
            shuffled[destination] = *value;
        });
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
        // The channel_destinations array will be stale now, regenerate it.
        self.resize()
    }

    // Regenerate the channel destinations for the current polyphony count.
    fn resize(&mut self) {
        let n = self.channel_count;

        // Select just the output destinations that fit inside our channel count.
        let subset = self
            .channel_destinations_full
            .iter()
            .cloned()
            .filter(|v| *v < n);

        let channel_destinations = &mut self.channel_destinations[..n];
        channel_destinations
            .iter_mut()
            .zip(subset)
            .for_each(|(o, i)| {
                *o = i;
            });
    }

    pub fn get_module_config_info(&self) -> *mut ModuleConfigInfo {
        ModuleConfigInfo::from_module_instance(self).into_ptr()
    }

    // This is handy for debugging.
    #[allow(dead_code)]
    fn get_destinations(&self) -> &[usize] {
        &self.channel_destinations[..self.channel_count]
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

    fn get_name(index: usize) -> &'static str {
        assert!(index < Self::COUNT);
        match index {
            0 => "Polyphonic",
            1 => "Shuffle trigger",
            _ => unreachable!(),
        }
    }

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

    fn get_name(index: usize) -> &'static str {
        assert!(index < Self::COUNT);
        "Shuffled polyphonic"
    }

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
            t_low.set_voltage_monophonic(gate::LOW);
            t_high.set_voltage_monophonic(gate::HIGH);
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
                    // Whenever .sort_floats() stabilizes, use that instead.
                    // https://github.com/rust-lang/rust/issues/93396
                    .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
                assert_eq!(i1.as_slice(), o1.as_slice_mut());
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
            assert_eq!(i1.as_slice().len(), o1.as_slice_mut().len());
            assert_ne!(i1.as_slice(), o1.as_slice_mut());

            // This should also pass the re-sorting test.
            o1.as_slice_mut()
                .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            assert_eq!(i1.as_slice(), o1.as_slice_mut());
        }
    }
}
