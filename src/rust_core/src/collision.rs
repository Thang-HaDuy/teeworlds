use crate::vec2::Vec2;

pub const COLFLAG_SOLID: i32 = 1;
pub const COLFLAG_DEATH: i32 = 2;
pub const COLFLAG_NOHOOK: i32 = 4;

#[derive(Debug)]
pub struct Collision {
    // tile map data will go here
}

impl Collision {
    pub fn new() -> Self {
        Self {}
    }

    pub fn check_point(&self, _x: f32, _y: f32) -> bool {
        false
    }

    pub fn check_point_flag(&self, _x: f32, _y: f32, _flag: i32) -> bool {
        false
    }

    /// Returns (hit_ground, hit_nohook, new_end_pos).
    /// Stub: always returns no hit.
    pub fn intersect_line(&self, _p0: Vec2, p1: Vec2) -> (bool, bool, Vec2) {
        (false, false, p1)
    }

    pub fn test_box(&self, _pos: Vec2, _size: Vec2, _col_flags: i32) -> bool {
        false
    }

    pub fn move_box(
        &self,
        inout_pos: &mut Vec2,
        inout_vel: &mut Vec2,
        size: Vec2,
        elasticity: f32,
        mut death: Option<&mut bool>,
    ) {
        let mut pos = *inout_pos;
        let mut vel = *inout_vel;

        let distance = vel.length();
        let max_steps = distance as i32;

        if let Some(d) = death.as_deref_mut() {
            *d = false;
        }

        if distance > 0.00001 {
            let fraction = 1.0 / (max_steps as f32 + 1.0);
            for _ in 0..=max_steps {
                let mut new_pos = Vec2 {
                    x: pos.x + vel.x * fraction,
                    y: pos.y + vel.y * fraction,
                };

                if let Some(d) = death.as_deref_mut() {
                    if self.test_box(new_pos, size * (2.0 / 3.0), COLFLAG_DEATH) {
                        *d = true;
                    }
                }

                if self.test_box(new_pos, size, 0) {
                    let mut hits = 0;

                    if self.test_box(Vec2 { x: pos.x, y: new_pos.y }, size, 0) {
                        new_pos.y = pos.y;
                        vel.y *= -elasticity;
                        hits += 1;
                    }

                    if self.test_box(Vec2 { x: new_pos.x, y: pos.y }, size, 0) {
                        new_pos.x = pos.x;
                        vel.x *= -elasticity;
                        hits += 1;
                    }

                    if hits == 0 {
                        new_pos.y = pos.y;
                        vel.y *= -elasticity;
                        new_pos.x = pos.x;
                        vel.x *= -elasticity;
                    }
                }

                pos = new_pos;
            }
        }

        *inout_pos = pos;
        *inout_vel = vel;
    }
}
