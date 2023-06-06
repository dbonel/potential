use crate::module_config::{RackInput, RackOutput};
use crate::rack::{InputPort, OutputPort, Port};

fn mag_sign_process(inputs: &MagSignInput, outputs: &mut MagSignOutput) {
    // Upper half processing: decomposition
    {
        outputs.magnitude.set_polyphony_from(&inputs.bipolar);
        outputs.sign.set_polyphony_from(&inputs.bipolar);
        let bipolar = inputs.bipolar.as_slice();
        let magnitude = outputs.magnitude.as_slice_mut();
        let sign = outputs.sign.as_slice_mut();
        bipolar
            .iter()
            .zip(magnitude.iter_mut().zip(sign.iter_mut()))
            .for_each(|(bipolar_in, (magnitude_out, sign_out))| {
                (*magnitude_out, *sign_out) = {
                    if !bipolar_in.is_nan() {
                        (bipolar_in.abs(), bipolar_in.signum())
                    } else {
                        // Map NaNs to positive 0.
                        (0.0, 1.0)
                    }
                };
            });
    }

    // Lower half processing: recomposition
    {
        // We only produce as many output channels as we have input signs. We
        // can pad the input magnitudes with 0.0, but the signs are impossible
        // to default to either positive or negative without biasing the output.
        outputs.bipolar.set_polyphony_from(&inputs.sign);
        let magnitude = inputs.magnitude.as_slice();
        let sign = inputs.sign.as_slice();
        let bipolar = outputs.bipolar.as_slice_mut();

        // We could have fewer magnitudes than signs, so we extend the magnitudes with
        // an infinite iterator of 0.0. This will be zipped with the signs, so the
        // resulting zip iterator will be the length of the signs.
        let zeros = Some(0.0).iter().cycle();
        let mag_sign_pairs = magnitude.iter().chain(zeros).zip(sign.iter());
        bipolar
            .iter_mut()
            .zip(mag_sign_pairs)
            .for_each(|(bipolar_out, (magnitude_in, sign_in))| {
                *bipolar_out = {
                    let result = magnitude_in.copysign(*sign_in);
                    if !result.is_nan() {
                        result
                    } else {
                        0.0
                    }
                };
            })
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

    fn get_name(index: usize) -> &'static str {
        assert!(index < Self::COUNT);
        match index {
            0 => "Bipolar",
            1 => "Magnitude",
            2 => "Sign",
            _ => unreachable!(),
        }
    }

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

    fn get_name(index: usize) -> &'static str {
        assert!(index < Self::COUNT);
        match index {
            0 => "Magnitude",
            1 => "Sign",
            2 => "Bipolar",
            _ => unreachable!(),
        }
    }

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
