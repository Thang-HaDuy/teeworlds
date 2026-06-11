use crate::vec2::Vec2;

// src/collision.rs
#[derive(Debug)]
pub struct Collision {
    // các field tạm
}

impl Collision {
    pub fn move_box(
        &self,
        inout_pos: &mut Vec2,
        inout_vel: &mut Vec2,
        size: Vec2,
        elasticity: f32,
        mut death: Option<&mut bool>, // <- mutable ở đây
    ) {
        let mut pos = *inout_pos;
        let mut vel = *inout_vel;

        let distance = vel.length();
        let max_steps = distance as i32;

        if let Some(d) = death.as_mut() {
            *d = false;
        }

        if distance > 0.00001 {
            let fraction = 1.0 / (max_steps as f32 + 1.0);
            for _ in 0..=max_steps {
                let mut new_pos = Vec2 {
                    x: pos.x + vel.x * fraction,
                    y: pos.y + vel.y * fraction,
                };

                if let Some(d) = death.as_mut() {
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

    // Thêm stub cho test_box
    fn test_box(&self, _pos: Vec2, _size: Vec2, _col_flags: i32) -> bool {
        // logic collision thực tế sẽ được implement sau
        false
    }
}

// Constants tương ứng với C++ code
const COLFLAG_DEATH: i32 = 1;