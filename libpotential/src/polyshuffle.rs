use crate::util::InputTrigger;

use rand::rngs::SmallRng;
use rand::seq::SliceRandom;
use rand::SeedableRng;

const PORT_MAX_CHANNELS: usize = 16;

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

    pub fn process(&mut self, inputs: &[f32], outputs: &mut [f32], shuffle_in: f32) -> usize {
        let channel_count = inputs.len();

        if self.shuffle_trigger.process_voltage(shuffle_in) {
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

        // Copy inputs to outputs according to their mapped destinations.
        inputs.iter().enumerate().for_each(|(i, value)| {
            let destination = self.channel_destinations[i];
            outputs[destination] = *value;
        });

        channel_count
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shuffle() {
        let mut p = PolyShuffle::new();
        let in_voltages = vec![
            0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 4.0, 4.5, 5.0, 5.5, 6.0, 6.5, 7.0, 7.5,
        ];
        let mut out_voltages: Vec<_> = (0..16).map(|_i| 0.0).collect();

        // Test the initial state (channel permutation is unmodified).
        p.process(in_voltages.as_slice(), out_voltages.as_mut_slice(), 0.0);
        assert_eq!(in_voltages.as_slice(), out_voltages.as_slice());
        dbg!(out_voltages.as_slice());

        // Test the shuffle trigger; the output order should be different from
        // before (unless we got extremely unlucky with the permutation).
        p.process(in_voltages.as_slice(), out_voltages.as_mut_slice(), 10.0);
        assert_ne!(in_voltages.as_slice(), out_voltages.as_slice());
        dbg!(out_voltages.as_slice());
        // However, we should still be able to sort our voltages back to their original order without
        // any going missing.
        out_voltages
            .as_mut_slice()
            // Whenever .sort_floats() stabilizes, use that instead.
            // https://github.com/rust-lang/rust/issues/93396
            .sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(in_voltages.as_slice(), out_voltages.as_slice());

        // Test the polyphony count changing.
        let n: usize = 8;
        let in_slice = &in_voltages[..n];
        let n = p.process(in_slice, out_voltages.as_mut_slice(), 0.0);
        let out_slice = &mut out_voltages[..n];
        assert_eq!(in_slice.len(), out_slice.len());
        assert_ne!(in_slice, out_slice);
        dbg!(&out_slice);
        // This should also pass the re-sorting test.
        out_slice.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        assert_eq!(&in_slice, &out_slice);
    }
}
