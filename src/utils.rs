// utils.rs
#[inline]
pub fn balance_ratio_to_stereo_coefficients(ratio: f32) -> (f32, f32) {
    use std::f32::consts::FRAC_PI_2;
    ((FRAC_PI_2 * ratio).cos(), (FRAC_PI_2 * ratio).sin())
}

/// Converts % to normalized value
#[inline]
pub fn knob_gain(knob_val: f32) -> f32 {
    knob_val / 100.0
}

/// Convex combination of points *a* and *b*. *ratio* must be in interval from 0 to 1.
#[inline]
pub fn convex(a: f32, b: f32, ratio: f32) -> f32 {
    (a - b) * ratio + b // <=> a * ratio + b * (1.0 - ratio)
}
