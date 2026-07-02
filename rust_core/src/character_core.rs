use crate::net_obj::{CoreEventFlags, PlayerInput};
use crate::vec2::Vec2;

pub const PHYS_SIZE: f32 = 28.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(i32)]
pub enum HookState {
    Retracted = -1,
    #[default]
    Idle = 0,
    RetractStart = 1,
    RetractEnd = 3,
    Flying = 4,
    Grabbed = 5,
}

impl HookState {
    pub fn from_i32(v: i32) -> Option<Self> {
        match v {
            -1 => Some(Self::Retracted),
            0 => Some(Self::Idle),
            1 => Some(Self::RetractStart),
            3 => Some(Self::RetractEnd),
            4 => Some(Self::Flying),
            5 => Some(Self::Grabbed),
            _ => None,
        }
    }
}

pub struct CharacterCore {
    pub pos: Vec2,
    pub vel: Vec2,
    pub hook_drag_vel: Vec2,
    pub hook_pos: Vec2,
    pub hook_dir: Vec2,
    pub hook_tick: i32,
    pub hook_state: HookState,
    pub hooked_player: i32,
    pub jumped: i32,
    pub direction: i32,
    pub angle: i32,
    pub death: bool,
    pub input: PlayerInput,
    pub triggered_events: CoreEventFlags,
}

impl Default for CharacterCore {
    fn default() -> Self {
        Self {
            pos: Vec2::zero(),
            vel: Vec2::zero(),
            hook_drag_vel: Vec2::zero(),
            hook_pos: Vec2::zero(),
            hook_dir: Vec2::zero(),
            hook_tick: 0,
            hook_state: HookState::default(),
            hooked_player: -1,
            jumped: 0,
            direction: 0,
            angle: 0,
            death: false,
            input: PlayerInput::default(),
            triggered_events: CoreEventFlags::empty(),
        }
    }
}

impl CharacterCore {
    pub fn new() -> Self {
        Self::default()
    }
    /// Mirrors C++ CCharacterCore::Reset()
    /// Chỉ reset physics state — giữ nguyên direction, angle, input
    pub fn reset(&mut self) {
        self.pos = Vec2::zero();
        self.vel = Vec2::zero();
        self.hook_drag_vel = Vec2::zero();
        self.hook_pos = Vec2::zero();
        self.hook_dir = Vec2::zero();
        self.hook_tick = 0;
        self.hook_state = HookState::Idle;
        self.hooked_player = -1;
        self.jumped = 0;
        self.triggered_events = CoreEventFlags::empty();
        self.death = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phys_size_is_28() {
        assert_eq!(PHYS_SIZE, 28.0);
    }

    #[test]
    fn default_hook_state_is_idle() {
        assert_eq!(CharacterCore::new().hook_state, HookState::Idle);
    }

    #[test]
    fn default_hooked_player_is_sentinel() {
        assert_eq!(CharacterCore::new().hooked_player, -1);
    }

    #[test]
    fn default_triggered_events_is_empty() {
        assert!(CharacterCore::new().triggered_events.is_empty());
    }

    #[test]
    fn hook_state_from_i32_roundtrip() {
        assert_eq!(HookState::from_i32(-1), Some(HookState::Retracted));
        assert_eq!(HookState::from_i32(0), Some(HookState::Idle));
        assert_eq!(HookState::from_i32(1), Some(HookState::RetractStart));
        assert_eq!(HookState::from_i32(3), Some(HookState::RetractEnd));
        assert_eq!(HookState::from_i32(4), Some(HookState::Flying));
        assert_eq!(HookState::from_i32(5), Some(HookState::Grabbed));
    }

    #[test]
    fn hook_state_gap_and_invalid_return_none() {
        assert_eq!(HookState::from_i32(2), None);
        assert_eq!(HookState::from_i32(99), None);
    }

    #[test]
    fn reset_clears_physics_state() {
        let mut c = CharacterCore::new();
        c.pos = Vec2::new(100.0, 200.0);
        c.vel = Vec2::new(5.0, -3.0);
        c.hook_state = HookState::Flying;
        c.hooked_player = 3;
        c.jumped = 2;
        c.death = true;

        c.reset();

        assert_eq!(c.pos, Vec2::zero());
        assert_eq!(c.vel, Vec2::zero());
        assert_eq!(c.hook_state, HookState::Idle);
        assert_eq!(c.hooked_player, -1);
        assert_eq!(c.jumped, 0);
        assert_eq!(c.death, false);
        assert!(c.triggered_events.is_empty());
    }

    #[test]
    fn reset_preserves_direction_and_input() {
        let mut c = CharacterCore::new();
        c.direction = 1;
        c.angle = 42;
        c.input.jump = 1;

        c.reset();

        assert_eq!(c.direction, 1);
        assert_eq!(c.angle, 42);
        assert_eq!(c.input.jump, 1);
    }
}
