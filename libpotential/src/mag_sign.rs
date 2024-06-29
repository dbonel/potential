use crate::module_config::{ModuleConfigInfo, RackInput, RackOutput, StaticModuleConfig};
use crate::rack::{InputPort, OutputPort, Port};

impl StaticModuleConfig for MagSign {
    const INPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Bipolar", c"Magnitude", c"Sign"];

    const OUTPUT_PORTS: &'static [&'static std::ffi::CStr] = &[c"Magnitude", c"Sign", c"Bipolar"];
}

pub struct MagSign {}

impl Default for MagSign {
    fn default() -> Self {
        Self {}
    }
}

impl MagSign {
    pub fn get_module_config_info(&self) -> *mut ModuleConfigInfo {
        ModuleConfigInfo::from_module_instance(self).into_ptr()
    }
}

fn mag_sign_process(inputs: &MagSignInput, outputs: &mut MagSignOutput) {
    // Upper half processing: decomposition
    if let Some(bipolar_voltages) = inputs.bipolar.as_slice() {
        let n = bipolar_voltages.len();

        if let Some(magnitude_voltages) = outputs.magnitude.as_slice_mut_from_polyphony_count(n) {
            assert_eq!(n, magnitude_voltages.len());
            magnitude_voltages
                .iter_mut()
                .zip(bipolar_voltages)
                .for_each(|(m, b)| {
                    *m = b.abs();
                });
        }

        if let Some(sign_voltages) = outputs.sign.as_slice_mut_from_polyphony_count(n) {
            assert_eq!(n, sign_voltages.len());
            sign_voltages
                .iter_mut()
                .zip(bipolar_voltages)
                .for_each(|(s, b)| {
                    *s = b.signum();
                });
        }
    } else {
        outputs.magnitude.set_polyphony_count(0);
        outputs.sign.set_polyphony_count(0);
    }

    // Lower half processing: recomposition
    //
    // We only produce as many output channels as we have input signs. We
    // can pad the input magnitudes with 0.0, but the signs are impossible to
    // default to either positive or negative without biasing the output.
    if let Some(sign_voltages) = inputs.sign.as_slice() {
        let n = sign_voltages.len();

        if let Some(bipolar_outputs) = outputs.bipolar.as_slice_mut_from_polyphony_count(n) {
            assert_eq!(bipolar_outputs.len(), n);

            let zero_padded_magnitudes = inputs
                .magnitude
                .as_slice()
                .unwrap_or_default()
                .iter()
                .chain(Some(0.0).iter().cycle())
                .take(n);
            let mag_sign_pairs = zero_padded_magnitudes.zip(sign_voltages);

            bipolar_outputs.iter_mut().zip(mag_sign_pairs).for_each(
                |(bipolar_out, (magnitude_in, sign_in))| {
                    *bipolar_out = magnitude_in.copysign(*sign_in);
                },
            );
        }
    } else {
        outputs.bipolar.set_polyphony_count(0);
    }
}

pub fn mag_sign_process_raw(inputs: *const Port, outputs: *mut Port) {
    let inputs = MagSignInput::from_raw_ptr(inputs);
    let mut outputs = MagSignOutput::from_raw_ptr(outputs);
    mag_sign_process(&inputs, &mut outputs)
}

struct MagSignInput<'a> {
    bipolar: InputPort<'a>,
    magnitude: InputPort<'a>,
    sign: InputPort<'a>,
}
impl RackInput for MagSignInput<'_> {
    const COUNT: usize = 3;

    fn from_raw_ptr(ports: *const Port) -> Self {
        let bipolar = InputPort::from_raw_port_index(ports, 0);
        let magnitude = InputPort::from_raw_port_index(ports, 1);
        let sign = InputPort::from_raw_port_index(ports, 2);
        MagSignInput {
            bipolar,
            magnitude,
            sign,
        }
    }
}

struct MagSignOutput<'a> {
    magnitude: OutputPort<'a>,
    sign: OutputPort<'a>,
    bipolar: OutputPort<'a>,
}
impl RackOutput for MagSignOutput<'_> {
    const COUNT: usize = 3;

    fn from_raw_ptr(ports: *mut Port) -> Self {
        let magnitude = OutputPort::from_raw_port_index(ports, 0);
        let sign = OutputPort::from_raw_port_index(ports, 1);
        let bipolar = OutputPort::from_raw_port_index(ports, 2);
        MagSignOutput {
            magnitude,
            sign,
            bipolar,
        }
    }
}
