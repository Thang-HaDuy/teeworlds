// src/rust_core/character_core.rs

use crate::vec2::Vec2;
use crate::world_core::WorldCore;
use crate::collision::Collision;
use crate::netobj::{NetObjCharacterCore, PlayerInput};
pub const PHYS_SIZE: f32 = 28.0;

pub struct CharacterCore<'a> {
    pub world: Option<&'a WorldCore>,
    pub collision: Option<&'a Collision>,

    pub pos: Vec2,
    pub vel: Vec2,

    pub hook_drag_vel: Vec2,
    pub hook_pos: Vec2,
    pub hook_dir: Vec2,
    pub hook_tick: i32,
    pub hook_state: i32,
    pub hooked_player: i32,

    pub jumped: i32,
    pub direction: i32,
    pub angle: i32,
    pub death: bool,

    pub input: PlayerInput,
    pub triggered_events: i32,
}

impl<'a> CharacterCore<'a> {
    pub fn new() -> Self {
        Self {
            world: None,
            collision: None,

            pos: Vec2::zero(),
            vel: Vec2::zero(),

            hook_drag_vel: Vec2::zero(),
            hook_pos: Vec2::zero(),
            hook_dir: Vec2::zero(),
            hook_tick: 0,
            hook_state: 0,
            hooked_player: -1,

            jumped: 0,
            direction: 0,
            angle: 0,
            death: false,

            input: PlayerInput::default(),
            triggered_events: 0,
        }
    }

    pub fn init(&mut self, world: &'a WorldCore, collision: &'a Collision) {
        self.world = Some(world);
        self.collision = Some(collision);
    }

    pub fn reset(&mut self) {
        self.pos = Vec2::zero();
        self.vel = Vec2::zero();
        self.hook_drag_vel = Vec2::zero();
        self.hook_pos = Vec2::zero();
        self.hook_dir = Vec2::zero();
        self.hook_tick = 0;
        self.hook_state = 0;
        self.hooked_player = -1;
        self.jumped = 0;
        self.triggered_events = 0;
        self.death = false;
    }

    pub fn add_drag_velocity(&mut self) {
        if let Some(world) = self.world {
            let ds = world.tuning().hook_drag_speed;
            self.vel.x = saturated_add(-ds, ds, self.vel.x, self.hook_drag_vel.x);
            self.vel.y = saturated_add(-ds, ds, self.vel.y, self.hook_drag_vel.y);
        }
    }

    pub fn reset_drag_velocity(&mut self) {
        self.hook_drag_vel = Vec2::zero();
    }

    pub fn write_netobj(&self, out: &mut NetObjCharacterCore) {
        out.m_x = round_to_int(self.pos.x);
        out.m_y = round_to_int(self.pos.y);

        // velocity is scaled by 256 like in C++
        out.m_vel_x = round_to_int(self.vel.x * 256.0);
        out.m_vel_y = round_to_int(self.vel.y * 256.0);

        out.m_hook_state = self.hook_state;
        out.m_hook_tick = self.hook_tick;
        out.m_hook_x = round_to_int(self.hook_pos.x);
        out.m_hook_y = round_to_int(self.hook_pos.y);

        out.m_hook_dx = round_to_int(self.hook_dir.x * 256.0);
        out.m_hook_dy = round_to_int(self.hook_dir.y * 256.0);

        out.m_hooked_player = self.hooked_player;
        out.m_jumped = self.jumped;
        out.m_direction = self.direction;
        out.m_angle = self.angle;
    }

    pub fn read_netobj(&mut self, obj: &NetObjCharacterCore) {
        // position
        self.pos.x = obj.m_x as f32;
        self.pos.y = obj.m_y as f32;

        // velocity scaled back down
        self.vel.x = (obj.m_vel_x as f32) / 256.0;
        self.vel.y = (obj.m_vel_y as f32) / 256.0;

        self.hook_state = obj.m_hook_state;
        self.hook_tick = obj.m_hook_tick;

        self.hook_pos.x = obj.m_hook_x as f32;
        self.hook_pos.y = obj.m_hook_y as f32;

        // hook dir scaled
        self.hook_dir.x = (obj.m_hook_dx as f32) / 256.0;
        self.hook_dir.y = (obj.m_hook_dy as f32) / 256.0;

        self.hooked_player = obj.m_hooked_player;
        self.jumped = obj.m_jumped;
        self.direction = obj.m_direction;
        self.angle = obj.m_angle;
    }

    pub fn tick(&mut self, use_input: bool) {
        self.triggered_events = 0;

        // grounded check
        let grounded = if let Some(collision) = self.collision {
            collision.check_point(self.pos.x + PHYS_SIZE / 2.0, self.pos.y + PHYS_SIZE / 2.0 + 5.0) ||
                collision.check_point(self.pos.x - PHYS_SIZE / 2.0, self.pos.y + PHYS_SIZE / 2.0 + 5.0)
        } else {
            false
        };

        let target_dir = self.input.get_target_direction();

        // gravity
        if let Some(world) = self.world {
            self.vel.y += world.tuning.gravity;
        }

        let (max_speed, accel, friction) = if grounded {
            if let Some(world) = self.world {
                (world.tuning.ground_control_speed, world.tuning.ground_control_accel, world.tuning.ground_friction)
            } else {
                (1.0, 1.0, 1.0) // fallback
            }
        } else {
            if let Some(world) = self.world {
                (world.tuning.air_control_speed, world.tuning.air_control_accel, world.tuning.air_friction)
            } else {
                (1.0, 1.0, 1.0)
            }
        };

        // handle input
        if use_input {
            self.direction = self.input.direction;
            self.angle = (target_dir.angle() * 256.0) as i32;

            // jumping
            if self.input.jump {
                if self.jumped & 1 == 0 {
                    if grounded {
                        self.triggered_events |= COREEVENTFLAG_GROUND_JUMP;
                        if let Some(world) = self.world {
                            self.vel.y = -world.tuning.ground_jump_impulse;
                        }
                        self.jumped |= 1;
                    } else if self.jumped & 2 == 0 {
                        self.triggered_events |= COREEVENTFLAG_AIR_JUMP;
                        if let Some(world) = self.world {
                            self.vel.y = -world.tuning.air_jump_impulse;
                        }
                        self.jumped |= 3;
                    }
                }
            } else {
                self.jumped &= !1;
            }

            // hook input
            if self.input.hook {
                if self.hook_state == HOOK_IDLE {
                    self.hook_state = HOOK_FLYING;
                    self.hook_pos = self.pos + target_dir * PHYS_SIZE * 1.5;
                    self.hook_dir = target_dir;
                    self.hooked_player = -1;
                    self.hook_tick = 0;
                }
            } else {
                self.hooked_player = -1;
                self.hook_state = HOOK_IDLE;
                self.hook_pos = self.pos;
            }
        }

        // movement
        if self.direction < 0 {
            self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, -accel);
        }
        if self.direction > 0 {
            self.vel.x = saturated_add(-max_speed, max_speed, self.vel.x, accel);
        }
        if self.direction == 0 {
            self.vel.x *= friction;
        }

        if grounded {
            self.jumped &= !2;
        }

        // handle hook
        self.update_hook();

        // clamp velocity
        if self.vel.length() > 6000.0 {
            self.vel = self.vel.normalize() * 6000.0;
        }
    }

    pub fn move_core(&mut self) {
        if self.world.is_none() {
            return;
        }

        // velocity ramp like C++ version
        if let Some(world) = self.world {
            let ramp = velocity_ramp(self.vel.length() * 50.0,
                                     world.tuning.velramp_start,
                                     world.tuning.velramp_range,
                                     world.tuning.velramp_curvature);
            self.vel.x *= ramp;
        }

        // move using collision system
        if let Some(collision) = self.collision {
            let mut new_pos = self.pos;
            collision.move_box(&mut new_pos, &mut self.vel, Vec2 { x: PHYS_SIZE, y: PHYS_SIZE }, 0, self.death);
            self.pos = new_pos;
        }

        // restore velocity x after ramp
        if let Some(world) = self.world {
            let ramp = velocity_ramp(self.vel.length() * 50.0,
                                     world.tuning.velramp_start,
                                     world.tuning.velramp_range,
                                     world.tuning.velramp_curvature);
            self.vel.x *= 1.0 / ramp;
        }
    }
    /// Same as CCharacterCore::Quantize — write then read on local object
    pub fn quantize(&mut self) {
        // create temporary network object
        let mut tmp = NetObjCharacterCore {
            m_x: 0,
            m_y: 0,
            m_vel_x: 0,
            m_vel_y: 0,
            m_hook_state: 0,
            m_hook_tick: 0,
            m_hook_x: 0,
            m_hook_y: 0,
            m_hook_dx: 0,
            m_hook_dy: 0,
            m_hooked_player: 0,
            m_jumped: 0,
            m_direction: 0,
            m_angle: 0,
        };

        // serialize current state
        self.write_netobj(&mut tmp);
        // read it back to truncate/quantize floats
        self.read_netobj(&tmp);
    }
}

pub fn saturated_add(min: f32, max: f32, current: f32, add: f32) -> f32 {
    (current + add).clamp(min, max)
}

// convert float -> rounded int
fn round_to_int(f: f32) -> i32 {
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