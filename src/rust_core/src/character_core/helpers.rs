pub fn round_to_int(f: f32) -> i32 {
    f.round() as i32
}

pub fn velocity_ramp(value: f32, start: f32, range: f32, curvature: f32) -> f32 {
    if value < start {
        1.0
    } else if value > start + range {
        1.0 + curvature
    } else {
        1.0 + curvature * ((value - start) / range)
    }
}
