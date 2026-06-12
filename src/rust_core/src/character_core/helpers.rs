use crate::vec2::Vec2;

pub fn round_to_int(f: f32) -> i32 {
    f.round() as i32
}

/// Mirrors C++ VelocityRamp: returns 1/curvature^((value-start)/range)
pub fn velocity_ramp(value: f32, start: f32, range: f32, curvature: f32) -> f32 {
    if value < start {
        1.0
    } else {
        1.0 / curvature.powf((value - start) / range)
    }
}

pub fn dot(a: Vec2, b: Vec2) -> f32 {
    a.x * b.x + a.y * b.y
}

pub fn mix(a: Vec2, b: Vec2, t: f32) -> Vec2 {
    a + (b - a) * t
}

/// Closest point on infinite line through AB to target point P.
/// Mirrors C++ closest_point_on_line from math.h.
pub fn closest_point_on_line(line_a: Vec2, line_b: Vec2, target: Vec2) -> Vec2 {
    let ab = line_b - line_a;
    let ab_len_sq = ab.x * ab.x + ab.y * ab.y;
    if ab_len_sq < 0.0001 {
        return line_a;
    }
    let ap = target - line_a;
    let t = (ap.x * ab.x + ap.y * ab.y) / ab_len_sq;
    line_a + ab * t
}
