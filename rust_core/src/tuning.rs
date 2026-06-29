// rust_core/src/tuning.rs

#[derive(Debug, Clone, Copy)]
pub struct Tuning {
    // physics
    pub ground_control_speed: f32,
    pub ground_control_accel: f32,
    pub ground_friction: f32,
    pub ground_jump_impulse: f32,
    pub air_jump_impulse: f32,
    pub air_control_speed: f32,
    pub air_control_accel: f32,
    pub air_friction: f32,
    pub hook_length: f32,
    pub hook_fire_speed: f32,
    pub hook_drag_accel: f32,
    pub hook_drag_speed: f32,
    pub gravity: f32,

    // velocity ramp
    pub velramp_start: f32,
    pub velramp_range: f32,
    pub velramp_curvature: f32,

    // weapons
    pub gun_curvature: f32,
    pub gun_speed: f32,
    pub gun_lifetime: f32,
    pub shotgun_curvature: f32,
    pub shotgun_speed: f32,
    pub shotgun_speeddiff: f32,
    pub shotgun_lifetime: f32,
    pub grenade_curvature: f32,
    pub grenade_speed: f32,
    pub grenade_lifetime: f32,
    pub laser_reach: f32,
    pub laser_bounce_delay: f32,
    pub laser_bounce_num: f32,
    pub laser_bounce_cost: f32,

    // player
    pub player_collision: f32,
    pub player_hooking: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            ground_control_speed: 10.0,
            ground_control_accel: 100.0 / 50.0, // TicksPerSecond = 50
            ground_friction: 0.5,
            ground_jump_impulse: 13.2,
            air_jump_impulse: 12.0,
            air_control_speed: 250.0 / 50.0, // TicksPerSecond = 50
            air_control_accel: 1.5,
            air_friction: 0.95,
            hook_length: 380.0,
            hook_fire_speed: 80.0,
            hook_drag_accel: 3.0,
            hook_drag_speed: 15.0,
            gravity: 0.5,
            velramp_start: 550.0,
            velramp_range: 2000.0,
            velramp_curvature: 1.4,
            gun_curvature: 1.25,
            gun_speed: 2200.0,
            gun_lifetime: 2.0,
            shotgun_curvature: 1.25,
            shotgun_speed: 2750.0,
            shotgun_speeddiff: 0.8,
            shotgun_lifetime: 0.20,
            grenade_curvature: 7.0,
            grenade_speed: 1000.0,
            grenade_lifetime: 2.0,
            laser_reach: 800.0,
            laser_bounce_delay: 150.0,
            laser_bounce_num: 1.0,
            laser_bounce_cost: 0.0,
            player_collision: 1.0,
            player_hooking: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_ground_control_speed() {
        let t = Tuning::default();
        assert_eq!(t.ground_control_speed, 10.0);
    }

    #[test]
    fn default_gravity() {
        let t = Tuning::default();
        assert_eq!(t.gravity, 0.5);
    }

    #[test]
    fn ticks_per_second_params() {
        let t = Tuning::default();
        // 100.0 / 50.0 = 2.0
        assert!((t.ground_control_accel - 2.0).abs() < f32::EPSILON);
        // 250.0 / 50.0 = 5.0
        assert!((t.air_control_speed - 5.0).abs() < f32::EPSILON);
    }
}
