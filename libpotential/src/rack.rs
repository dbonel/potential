use std::num::NonZeroU8;

pub const PORT_MAX_CHANNELS: usize = 16;
const MONOPHONIC: Option<NonZeroU8> = NonZeroU8::new(1);

// This is our internal representation of rack::engine::Port. This allows us to
// use borrowed pointers to the Rack engine's Port data, as long as we follow
// the same rules as Rack's API functions.
#[repr(C)]
pub struct Port {
    voltages: [f32; PORT_MAX_CHANNELS],
    channels: Option<NonZeroU8>,
    // We don't use these but include them for size/alignment matching.
    _lights: [f32; 3],
}
impl Default for Port {
    fn default() -> Self {
        let voltages = [0.0; PORT_MAX_CHANNELS];
        let channels = MONOPHONIC;
        let _lights = [0.0; 3];
        Port {
            voltages,
            channels,
            _lights,
        }
    }
}

// FIXME: fill in methods
#[repr(C)]
#[derive(Default)]
pub struct Param {
    value: f32,
}

// FIXME: fill in methods
#[repr(C)]
#[derive(Default)]
pub struct Light {
    value: f32,
}

// A wrapper type over a reference to a Port inside the Rack engine.
pub struct InputPort<'a> {
    inner: &'a Port,
}

impl<'a> InputPort<'a> {
    // This is a safe interface, which is mostly for unit tests.
    pub fn wrap(port: &'a Port) -> Self {
        InputPort { inner: port }
    }

    // Construct an InputPort from a raw pointer from across the FFI.
    pub fn from_raw_port(port: *const Port) -> Self {
        let inner = unsafe { &*port };
        InputPort { inner }
    }

    // Construct an InputPort from a raw base pointer and an index. If we know
    // there's an array on the other side, this makes it easier to construct a
    // few of these in a row.
    pub fn from_raw_port_index(port: *const Port, index: usize) -> Self {
        let inner = unsafe {
            let port = port.add(index);
            &*port
        };
        InputPort { inner }
    }

    // Get a single voltage from the first channel. Returns None if the port
    // isn't connected (i.e. has 0 channels).
    pub fn get_monophonic_voltage(&self) -> Option<f32> {
        self.as_slice().map(|voltages| {
            // SAFETY: .as_slice() will never return a zero-length slice because
            // of the guarantee provided by get_polyphony_count().
            unsafe { *voltages.get_unchecked(0) }
        })
    }

    // Get a single voltage from the first channel, or 0.0 if the port is not
    // connected.
    pub fn get_zero_normaled_monophonic_voltage(&self) -> f32 {
        self.get_monophonic_voltage().unwrap_or(0.0)
    }

    // Get the number of polyphony channels on the port. An unconnected port
    // returns None. The polyphony count is guaranteed to be non-zero if the
    // port is connected.
    pub fn get_polyphony_count(&self) -> Option<usize> {
        self.inner.channels.map(|channels| {
            let channels = channels.get() as usize;
            debug_assert!(channels <= PORT_MAX_CHANNELS);
            channels
        })
    }

    // Get all of the voltages as a slice. As with get_polyphony_count, an
    // unconnected port returns None, and the slice length is guaranteed to be
    // >0 if the port is connected.
    pub fn as_slice(&self) -> Option<&[f32]> {
        self.get_polyphony_count()
            .map(|n| &self.inner.voltages[..n])
    }
}

pub struct OutputPort<'a> {
    inner: &'a mut Port,
}

impl<'a> OutputPort<'a> {
    pub fn wrap(port: &'a mut Port) -> Self {
        OutputPort { inner: port }
    }

    pub fn from_raw_port(port: *mut Port) -> Self {
        let inner = unsafe { &mut *port };
        OutputPort { inner }
    }

    pub fn from_raw_port_index(port: *mut Port, index: usize) -> Self {
        let inner = unsafe {
            let port = port.add(index);
            &mut *port
        };
        OutputPort { inner }
    }

    // Set the polyphony count to 1, and output a single voltage.
    pub fn set_monophonic_voltage(&mut self, voltage: f32) {
        self.set_polyphony_count(1);
        self.inner.voltages[0] = voltage;
    }

    // Set the polyphony count, and zero any unused positions in the voltages
    // array. The Rack API does this same zeroing in setChannels().
    //
    // If the port is currently disconnected, we will not change the polyphony
    // count, and a None will be returned.
    //
    // If the requested polyphony count `n` is 0, we will set a single voltage of 0.0, and
    // a None will be returned.
    //
    // The requested polyphony size `n` will be truncated to PORT_MAX_CHANNELS
    // before applying the change.
    pub fn set_polyphony_count(&mut self, n: usize) -> Option<usize> {
        let n = clamp_polyphony_count(n) as u8;
        let new_channels = NonZeroU8::new(n);
        self.set_port_polyphony(new_channels)
    }

    // Set the polyphony channel count of this OutputPort to the same count as
    // a reference InputPort.
    pub fn set_polyphony_from(&mut self, other: &InputPort) {
        let reference_polyphony = other.get_polyphony_count();
        let new_channels = reference_polyphony.map(|n| {
            debug_assert!(n <= PORT_MAX_CHANNELS);
            let n = clamp_polyphony_count(n) as u8;
            // SAFETY: We know n is at least 1 because 0 is mapped to None.
            unsafe { NonZeroU8::new_unchecked(n) }
        });
        self.set_port_polyphony(new_channels);
    }

    // This is the private inner implementation used by set_polyphony_count()
    // and set_polyphony_from().
    //
    // SAFETY: The caller is responsible for upholding the new_channels <=
    // PORT_MAX_CHANNELS invariant.
    fn set_port_polyphony(&mut self, new_channels: Option<NonZeroU8>) -> Option<usize> {
        let old_channels = self.inner.channels;
        let r = match (old_channels, new_channels) {
            // No change, no-op.
            (c1, c2) if c1 == c2 => old_channels,

            // Cable is connected, caller asked for 0.
            (Some(_), None) => {
                self.inner.channels = MONOPHONIC;
                self.inner.voltages.fill(0.0);
                new_channels
            }

            // Cable is connected, caller has changed polyphony count.
            (Some(_), Some(c2)) => {
                let c2 = c2.get() as usize;
                self.inner.channels = new_channels;
                self.inner.voltages[(c2)..].fill(0.0);
                new_channels
            }

            // Cable is disconnected, no-op.
            (None, _) => None,
        };
        r.map(|n| n.get() as usize)
    }

    // Get the output voltages as a mutable slice. The polyphony channel count
    // should already have been set before calling this.
    pub fn as_slice_mut(&mut self) -> Option<&mut [f32]> {
        self.inner.channels.map(|n| {
            let n = n.get() as usize;
            &mut self.inner.voltages[..n]
        })
    }

    pub fn as_slice_mut_from_polyphony_count(&mut self, n: usize) -> Option<&mut [f32]> {
        self.set_polyphony_count(n)
            .map(|n| &mut self.inner.voltages[..n])
    }

    // Set polyphonic voltages from a slice. This uses the same semantics as
    // set_polyphony_count, so if the slice is longer than PORT_MAX_CHANNELS,
    // the excess values will be ignored.
    pub fn set_voltages_from_slice(&mut self, voltages: &[f32]) -> Option<usize> {
        let n = voltages.len();
        self.set_polyphony_count(n).and_then(|n1| {
            self.as_slice_mut().map(|values| {
                let n2 = values.len();
                debug_assert_eq!(n1, n2);
                values.copy_from_slice(&voltages[..n1]);
                values.len()
            })
        })
    }
}

// A helper function to clamp `count` to PORT_MAX_CHANNELS.
fn clamp_polyphony_count(count: usize) -> usize {
    PORT_MAX_CHANNELS.min(count)
}

pub struct ModuleLight<'a> {
    inner: &'a mut Light,
}

impl ModuleLight<'_> {
    pub fn set_brightness(&mut self, brightness: f32) {
        self.inner.value = brightness;
    }
}

pub struct ModuleParam<'a> {
    inner: &'a Param,
}

impl ModuleParam<'_> {
    pub fn get_value(&self) -> f32 {
        self.inner.value
    }
}
