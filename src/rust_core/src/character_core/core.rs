use crate::vec2::{Vec2, saturated_add};
use crate::world_core::WorldCore;
use crate::collision::Collision;
use crate::netobj::{NetObjCharacterCore, PlayerInput};
use super::constants::*;
use super::helpers::{round_to_int, velocity_ramp, dot, mix, closest_point_on_line};

pub struct CharacterCore {
    /// Raw pointer to the shared WorldCore. Must be set via init() before tick/move.
    pub world: *mut WorldCore,
    /// Raw pointer to the Collision instance. Must be set via init() before tick/move.
    pub collision: *mut Collision,

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

// SAFETY: CharacterCore is used single-threaded in the game loop.
unsafe impl Send for CharacterCore {}

impl CharacterCore {
    pub fn new() -> Self {
        Self {
            world: std::ptr::null_mut(),
            collision: std::ptr::null_mut(),
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

    pub fn init(&mut self, world: &mut WorldCore, collision: &mut Collision) {
        self.world = world as *mut _;
        self.collision = collision as *mut _;
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
        let world = match unsafe { self.world.as_ref() } {
            Some(w) => w,
            None => return,
        };
        let ds = world.tuning.hook_drag_speed;
        self.vel.x = saturated_add(-ds, ds, self.vel.x, self.hook_drag_vel.x);
        self.vel.y = saturated_add(-ds, ds, self.vel.y, self.hook_drag_vel.y);
    }

    pub fn reset_drag_velocity(&mut self) {
        self.hook_drag_vel = Vec2::zero();
    }

    pub fn write_netobj(&self, out: &mut NetObjCharacterCore) {
        out.m_x = round_to_int(self.pos.x);
        out.m_y = round_to_int(self.pos.y);
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
        self.pos.x = obj.m_x as f32;
        self.pos.y = obj.m_y as f32;
        self.vel.x = (obj.m_vel_x as f32) / 256.0;
        self.vel.y = (obj.m_vel_y as f32) / 256.0;
        self.hook_state = obj.m_hook_state;
        self.hook_tick = obj.m_hook_tick;
        self.hook_pos.x = obj.m_hook_x as f32;
        self.hook_pos.y = obj.m_hook_y as f32;
        self.hook_dir.x = (obj.m_hook_dx as f32) / 256.0;
        self.hook_dir.y = (obj.m_hook_dy as f32) / 256.0;
        self.hooked_player = obj.m_hooked_player;
        self.jumped = obj.m_jumped;
        self.direction = obj.m_direction;
        self.angle = obj.m_angle;
    }

    pub fn quantize(&mut self) {
        let mut tmp = NetObjCharacterCore {
            m_x: 0, m_y: 0, m_vel_x: 0, m_vel_y: 0,
            m_hook_state: 0, m_hook_tick: 0,
            m_hook_x: 0, m_hook_y: 0, m_hook_dx: 0, m_hook_dy: 0,
            m_hooked_player: 0, m_jumped: 0, m_direction: 0, m_angle: 0,
        };
        self.write_netobj(&mut tmp);
        self.read_netobj(&tmp);
    }

    /// Mirrors C++ CCharacterCore::Tick(bool UseInput).
    pub fn tick(&mut self, use_input: bool) {
        let world = match unsafe { self.world.as_ref() } {
            Some(w) => w,
            None => return,
        };
        let collision = match unsafe { self.collision.as_ref() } {
            Some(c) => c,
            None => return,
        };

        self.triggered_events = 0;

        let grounded =
            collision.check_point(self.pos.x + PHYS_SIZE / 2.0, self.pos.y + PHYS_SIZE / 2.0 + 5.0)
                || collision.check_point(self.pos.x - PHYS_SIZE / 2.0, self.pos.y + PHYS_SIZE / 2.0 + 5.0);

        let target_dir = self.input.get_target_direction();

        self.vel.y += world.tuning.gravity;

        let (max_speed, accel, friction) = if grounded {
            (world.tuning.ground_control_speed, world.tuning.ground_control_accel, world.tuning.ground_friction)
        } else {
            (world.tuning.air_control_speed, world.tuning.air_control_accel, world.tuning.air_friction)
        };

        if use_input {
            self.direction = self.input.direction;
            self.angle = (target_dir.angle() * 256.0) as i32;

            if self.input.jump {
                if self.jumped & 1 == 0 {
                    if grounded {
                        self.triggered_events |= COREEVENTFLAG_GROUND_JUMP;
                        self.vel.y = -world.tuning.ground_jump_impulse;
                        self.jumped |= 1;
                    } else if self.jumped & 2 == 0 {
                        self.triggered_events |= COREEVENTFLAG_AIR_JUMP;
                        self.vel.y = -world.tuning.air_jump_impulse;
                        self.jumped |= 3;
                    }
                }
            } else {
                self.jumped &= !1;
            }

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

        // ---- Hook state machine (mirrors C++ CCharacterCore::Tick inline) ----

        if self.hook_state == HOOK_IDLE {
            self.hooked_player = -1;
            self.hook_pos = self.pos;
        } else if self.hook_state >= HOOK_RETRACT_START && self.hook_state < HOOK_RETRACT_END {
            self.hook_state += 1;
        } else if self.hook_state == HOOK_RETRACT_END {
            self.hook_state = HOOK_RETRACTED;
        } else if self.hook_state == HOOK_FLYING {
            let mut new_pos = self.hook_pos + self.hook_dir * world.tuning.hook_fire_speed;

            if (self.pos - new_pos).length() > world.tuning.hook_length {
                self.hook_state = HOOK_RETRACT_START;
                new_pos = self.pos + (new_pos - self.pos).normalize() * world.tuning.hook_length;
            }

            // Tile collision check
            let mut going_to_hit_ground = false;
            let mut going_to_retract = false;
            let (hit_ground, hit_nohook, clipped_pos) = collision.intersect_line(self.hook_pos, new_pos);
            new_pos = clipped_pos;
            if hit_ground {
                going_to_hit_ground = true;
            } else if hit_nohook {
                going_to_retract = true;
            }

            // Player check has priority over tile collision
            if world.tuning.player_hooking {
                let mut best_dist = f32::MAX;
                unsafe {
                    for i in 0..MAX_CLIENTS {
                        let pchar = world.ap_characters[i];
                        if pchar.is_null() || pchar == self as *mut _ {
                            continue;
                        }
                        let pchar_ref = &*pchar;
                        let closest = closest_point_on_line(self.hook_pos, new_pos, pchar_ref.pos);
                        if (pchar_ref.pos - closest).length() < PHYS_SIZE + 2.0 {
                            let dist = (self.hook_pos - pchar_ref.pos).length();
                            if self.hooked_player == -1 || dist < best_dist {
                                self.triggered_events |= COREEVENTFLAG_HOOK_ATTACH_PLAYER;
                                self.hook_state = HOOK_GRABBED;
                                self.hooked_player = i as i32;
                                best_dist = dist;
                            }
                        }
                    }
                }
            }

            // Ground/nohook check only if still flying after player check
            if self.hook_state == HOOK_FLYING {
                if going_to_hit_ground {
                    self.triggered_events |= COREEVENTFLAG_HOOK_ATTACH_GROUND;
                    self.hook_state = HOOK_GRABBED;
                } else if going_to_retract {
                    self.triggered_events |= COREEVENTFLAG_HOOK_HIT_NOHOOK;
                    self.hook_state = HOOK_RETRACT_START;
                }
                self.hook_pos = new_pos;
            }
        }

        if self.hook_state == HOOK_GRABBED {
            if self.hooked_player != -1 {
                unsafe {
                    let pchar = world.ap_characters[self.hooked_player as usize];
                    if !pchar.is_null() {
                        // Track hooked player's position
                        self.hook_pos = (*pchar).pos;
                    } else {
                        // Hooked player disappeared — release
                        self.hooked_player = -1;
                        self.hook_state = HOOK_RETRACTED;
                        self.hook_pos = self.pos;
                    }
                }
            }

            // Ground hook drag — only when not hooked to a player
            if self.hooked_player == -1 && (self.hook_pos - self.pos).length() > 46.0 {
                let mut hook_vel = (self.hook_pos - self.pos).normalize() * world.tuning.hook_drag_accel;

                // Upward pull is stronger (easier to climb platforms)
                if hook_vel.y > 0.0 {
                    hook_vel.y *= 0.3;
                }
                // Boost horizontal pull when moving in that direction, dampen otherwise
                if (hook_vel.x < 0.0 && self.direction < 0) || (hook_vel.x > 0.0 && self.direction > 0) {
                    hook_vel.x *= 0.95;
                } else {
                    hook_vel.x *= 0.75;
                }

                let new_vel = self.vel + hook_vel;
                // Only apply if it doesn't exceed drag speed limit OR is decelerating
                if new_vel.length() < world.tuning.hook_drag_speed || new_vel.length() < self.vel.length() {
                    self.vel = new_vel;
                }
            }

            // Timeout for player hooks (1.25 seconds)
            self.hook_tick += 1;
            if self.hooked_player != -1 {
                let still_alive = !world.ap_characters[self.hooked_player as usize].is_null();
                if self.hook_tick > SERVER_TICK_SPEED + SERVER_TICK_SPEED / 5 || !still_alive {
                    self.hooked_player = -1;
                    self.hook_state = HOOK_RETRACTED;
                    self.hook_pos = self.pos;
                }
            }
        }

        // ---- Player-player collision and hook drag (mirrors C++ Tick loop) ----
        unsafe {
            for i in 0..MAX_CLIENTS {
                let pchar = world.ap_characters[i];
                if pchar.is_null() || pchar == self as *mut _ {
                    continue;
                }
                let pchar_ref = &mut *pchar;

                let dist = (self.pos - pchar_ref.pos).length();
                let dir = (self.pos - pchar_ref.pos).normalize();

                // Push characters apart when overlapping
                if world.tuning.player_collision && dist < PHYS_SIZE * 1.25 && dist > 0.0 {
                    let overlap = PHYS_SIZE * 1.45 - dist;
                    let velocity = if self.vel.length() > 0.0001 {
                        1.0 - (dot(self.vel.normalize(), dir) + 1.0) / 2.0
                    } else {
                        0.5
                    };
                    self.vel = self.vel + dir * overlap * (velocity * 0.75);
                    self.vel = self.vel * 0.85;
                }

                // Apply hook drag forces between hooker and hooked player
                if self.hooked_player == i as i32 && world.tuning.player_hooking {
                    if dist > PHYS_SIZE * 1.5 {
                        let accel = world.tuning.hook_drag_accel * (dist / world.tuning.hook_length);
                        // Pull hooked player toward hooker
                        pchar_ref.hook_drag_vel = pchar_ref.hook_drag_vel + dir * accel * 1.5;
                        // Small counter-force on the hooker
                        self.hook_drag_vel = self.hook_drag_vel - dir * accel * 0.25;
                    }
                }
            }
        }

        // Clamp velocity
        if self.vel.length() > 6000.0 {
            self.vel = self.vel.normalize() * 6000.0;
        }
    }

    /// Mirrors C++ CCharacterCore::Move().
    pub fn move_core(&mut self) {
        let world = match unsafe { self.world.as_ref() } {
            Some(w) => w,
            None => return,
        };
        let collision = match unsafe { self.collision.as_ref() } {
            Some(c) => c,
            None => return,
        };

        let ramp = velocity_ramp(
            self.vel.length() * 50.0,
            world.tuning.velramp_start,
            world.tuning.velramp_range,
            world.tuning.velramp_curvature,
        );
        self.vel.x *= ramp;

        let mut new_pos = self.pos;
        collision.move_box(
            &mut new_pos,
            &mut self.vel,
            Vec2 { x: PHYS_SIZE, y: PHYS_SIZE },
            0.0,
            Some(&mut self.death),
        );

        if ramp != 0.0 {
            self.vel.x *= 1.0 / ramp;
        }

        if world.tuning.player_collision {
            let dist = (self.pos - new_pos).length();
            let end = dist as i32 + 1;
            let mut last_pos = self.pos;
            let mut collision_found = false;
            let mut collision_pos = self.pos; // default: stay in place on early return

            'outer: for i in 0..end {
                let a = if dist > 0.0 { i as f32 / dist } else { 0.0 };
                let pos = mix(self.pos, new_pos, a);

                unsafe {
                    for p in 0..MAX_CLIENTS {
                        let pchar = world.ap_characters[p];
                        if pchar.is_null() || pchar == self as *mut _ {
                            continue;
                        }
                        let pchar_ref = &*pchar;
                        let d = (pos - pchar_ref.pos).length();
                        if d < PHYS_SIZE && d >= 0.0 {
                            collision_found = true;
                            if a > 0.0 {
                                collision_pos = last_pos;
                            } else if (new_pos - pchar_ref.pos).length() > d {
                                collision_pos = new_pos;
                            }
                            // else: neither condition — pos stays as self.pos (no move)
                            break 'outer;
                        }
                    }
                }
                last_pos = pos;
            }

            if collision_found {
                self.pos = collision_pos;
                return;
            }
        }

        self.pos = new_pos;
    }
}
