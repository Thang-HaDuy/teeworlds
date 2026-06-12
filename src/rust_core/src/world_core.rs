use crate::character_core::CharacterCore;
use crate::character_core::MAX_CLIENTS;

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
    pub player_collision: bool,
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
            ground_control_speed: 10.0,
            ground_control_accel: 100.0 / 50.0,
            ground_friction: 0.5,
            air_control_speed: 8.0,
            air_control_accel: 250.0 / 50.0 / 50.0,
            air_friction: 0.95,
            ground_jump_impulse: 13.2,
            air_jump_impulse: 12.0,
            hook_fire_speed: 80.0,
            hook_drag_accel: 3.0,
            hook_drag_speed: 15.0,
            hook_length: 380.0,
            player_collision: true,
            player_hooking: true,
            velramp_start: 550.0,
            velramp_range: 2000.0,
            velramp_curvature: 1.4,
            shotgun_speeddiff: 0.8,
            shotgun_lifetime: 2.0,
            laser_reach: 800.0,
        }
    }
}

pub struct WorldCore {
    pub tuning: Tuning,
    /// Pointers to all active CharacterCores, indexed by client ID.
    /// Null pointer means slot is empty. Mirrors C++ CWorldCore::m_apCharacters.
    pub ap_characters: [*mut CharacterCore; MAX_CLIENTS],
}

// SAFETY: WorldCore is used single-threaded in the game loop.
unsafe impl Send for WorldCore {}
unsafe impl Sync for WorldCore {}

impl WorldCore {
    pub fn new() -> Self {
        Self {
            tuning: Tuning::default(),
            ap_characters: [std::ptr::null_mut(); MAX_CLIENTS],
        }
    }

    pub fn tuning(&self) -> &Tuning {
        &self.tuning
    }

    pub fn tuning_mut(&mut self) -> &mut Tuning {
        &mut self.tuning
    }
}
