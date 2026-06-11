// src/world_core.rs


#[derive(Debug, Clone, Copy)]
pub struct Tuning {
    pub gravity: f32,
    pub ground_control_speed: f32,
    pub ground_control_accel: f32,
    pub ground_friction: f32,
    pub air_control_speed: f32,
    pub air_control_accel: f32,
    pub air_friction: f32,
    pub ground_jump_impulse: f32,
    pub air_jump_impulse: f32,
    pub hook_fire_speed: f32,
    pub hook_drag_accel: f32,
    pub hook_drag_speed: f32,
    pub hook_length: f32,
    pub player_hooking: bool,
    pub velramp_start: f32,
    pub velramp_range: f32,
    pub velramp_curvature: f32,
    pub shotgun_speeddiff: f32,
    pub shotgun_lifetime: f32,
    pub laser_reach: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            gravity: 0.5,
            ground_control_speed: 1.0,
            ground_control_accel: 0.8,
            ground_friction: 0.7,
            air_control_speed: 0.6,
            air_control_accel: 0.5,
            air_friction: 0.98,
            ground_jump_impulse: 10.0,
            air_jump_impulse: 8.0,
            hook_fire_speed: 20.0,
            hook_drag_accel: 2.0,
            hook_drag_speed: 12.0,
            hook_length: 200.0,
            player_hooking: true,
            velramp_start: 100.0,
            velramp_range: 200.0,
            velramp_curvature: 1.2,
            shotgun_speeddiff: 0.9,
            shotgun_lifetime: 1.5,
            laser_reach: 300.0,
        }
    }
}

#[derive(Debug)]
pub struct WorldCore {
    pub tuning: Tuning,
    // Thêm các entity/character map nếu cần, ví dụ:
    pub ap_characters: [Option<*mut ()>; 64], // placeholder pointer, sau này thay bằng CharacterCore reference
}

impl WorldCore {
    pub fn new() -> Self {
        Self {
            tuning: Tuning::default(),
            ap_characters: [None; 64],
        }
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    pub fn tuning_mut(&mut self) -> &mut Tuning {
        &mut self.tuning
    }

    // thêm các function cần thiết về va chạm hay tìm entity ở đây
}