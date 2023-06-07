pub const PORT_MAX_CHANNELS: usize = 16;

// This is our internal representation of rack::engine::Port. This allows us to
// use borrowed pointers to the Rack engine's Port data, as long as we follow
// the same rules as Rack's API functions.
#[repr(C)]
pub struct Port {
    voltages: [f32; PORT_MAX_CHANNELS],
    channels: u8,
    // We don't use these but include them for size/alignment matching.
    _lights: [f32; 3],
}
impl Default for Port {
    fn default() -> Self {
        let voltages = [0.0; PORT_MAX_CHANNELS];
        let channels = 1;
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
    pub fn get_voltage_monophonic(&self) -> Option<f32> {
        self.as_slice().first().copied()
    }

    // Get the number of polyphony channels on the port. An unconnected port has
    // 0 channels.
    pub fn get_polyphony_count(&self) -> usize {
        debug_assert!(self.inner.channels <= PORT_MAX_CHANNELS as u8);
        self.inner.channels as usize
    }

    // Get all of the voltages as a slice.
    pub fn as_slice(&self) -> &[f32] {
        let n = self.get_polyphony_count();
        &self.inner.voltages[..n]
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
    pub fn set_voltage_monophonic(&mut self, voltage: f32) {
        self.set_polyphony_count(1);
        self.inner.voltages[0] = voltage;
    }

    // Set the polyphony count, and zero any unused positions in the voltages
    // array. The Rack API does this same zeroing in setChannels().
    pub fn set_polyphony_count(&mut self, n: usize) {
        // Rack uses channels==0 to mean the port has no cable connected. Don't
        // allow changes away from 0.
        if self.inner.channels == 0 {
            return;
        }

        // We keep "u" as the caller-requested number of channels (which could be 0).
        let u = n.min(PORT_MAX_CHANNELS);

        // Don't let the caller set channels==0 either, or exceed the Rack polyphony max.
        let n = u.max(1);

        // Let's not do a lot of needless memsets if we're not changing anything.
        if n != self.inner.channels as usize {
            // We use `m` here and not `n` because the caller may not remember to
            // zero the first voltage if they tried to set channels to 0.
            self.inner.voltages[u..].fill(0.0);
            self.inner.channels = n as u8;
        }
    }

    // Set the polyphony channel count of this OutputPort to the same count as
    // a reference InputPort.
    pub fn set_polyphony_from(&mut self, other: &InputPort) {
        let n = other.get_polyphony_count();
        debug_assert!(n <= PORT_MAX_CHANNELS);
        self.set_polyphony_count(n);
    }

    // Get the output voltages as a mutable slice. The polyphony channel count
    // should already have been set before calling this.
    pub fn as_slice_mut(&mut self) -> &mut [f32] {
        let n = self.inner.channels as usize;
        &mut self.inner.voltages[..n]
    }

    pub fn set_voltages_from_slice(&mut self, voltages: &[f32]) {
        let n = voltages.len();
        assert!(n <= PORT_MAX_CHANNELS);
        self.set_polyphony_count(n);
        if n > 0 {
            self.as_slice_mut().copy_from_slice(voltages)
        }
    }
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
