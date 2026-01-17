// utils.rs
#[inline]
pub fn db_to_gain(db: f32) -> f32 {
    if db <= -80.0 {
        0.0
    } else {
        10f32.powf(db / 20.0)
    }
}

#[inline]
pub fn knob_gain(knob_val: f32) -> f32 {
    knob_val.signum() * db_to_gain(80.0 * (knob_val.abs() - 1.0))
}


#[inline]
pub fn convex(a: f32, b: f32, ratio: f32) -> f32 {
    (a - b) * ratio + b // a*ratio + b*(1 - ratio)
}
