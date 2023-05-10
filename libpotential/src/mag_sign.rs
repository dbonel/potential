// Decompose a slice of floats into their magnitudes (the absolute values)
// and their signs (a value that is either -1.0 or +1.0).
pub fn decompose(in_values: &[f32], out_magnitudes: &mut [f32], out_signs: &mut [f32]) {
    debug_assert!(out_signs.len() >= in_values.len());
    debug_assert!(out_magnitudes.len() >= in_values.len());
    in_values
        .iter()
        .zip(out_magnitudes.iter_mut().zip(out_signs.iter_mut()))
        .for_each(|(i, (o_mag, o_sign))| {
            (*o_mag, *o_sign) = {
                if i.is_finite() {
                    (i.abs(), i.signum())
                } else {
                    // Map infinities and NaNs to positive 0.
                    (0.0, 1.0)
                }
            };
        });
}

pub fn recompose(in_magnitudes: &[f32], in_signs: &[f32], out_values: &mut [f32]) {
    debug_assert!(out_values.len() >= in_signs.len());
    debug_assert!(out_values.len() >= in_magnitudes.len());
    out_values
        .iter_mut()
        .zip(in_magnitudes.iter().zip(in_signs.iter()))
        .for_each(|(o, (i_mag, i_sign))| {
            *o = {
                if i_mag.is_finite() && i_sign.is_finite() {
                    i_mag.copysign(*i_sign)
                } else {
                    0.0
                }
            };
        })
}
