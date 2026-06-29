use bitflags::bitflags;

pub const NET_SCALE: i32 = 100;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PlayerInput {
    pub direction: i32,
    pub target_x: i32,
    pub target_y: i32,
    pub jump: i32,
    pub fire: i32,
    pub hook: i32,
    pub player_flags: i32,
    pub wanted_weapon: i32,
    pub next_weapon: i32,
    pub prev_weapon: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct CharacterCoreNet {
    pub tick: i32,
    pub x: i32,
    pub y: i32,
    pub vel_x: i32,
    pub vel_y: i32,
    pub angle: i32,
    pub direction: i32,
    pub jumped: i32,
    pub hooked_player: i32,
    pub hook_state: i32,
    pub hook_tick: i32,
    pub hook_x: i32,
    pub hook_y: i32,
    pub hook_dx: i32,
    pub hook_dy: i32,
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct PlayerFlags: i32 {
        const ADMIN      = 1 << 0;
        const CHATTING   = 1 << 1;
        const SCOREBOARD = 1 << 2;
        const READY      = 1 << 3;
        const DEAD       = 1 << 4;
        const WATCHING   = 1 << 5;
        const BOT        = 1 << 6;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
    pub struct CoreEventFlags: i32 {
        const GROUND_JUMP        = 1 << 0;
        const AIR_JUMP           = 1 << 1;
        const HOOK_ATTACH_PLAYER = 1 << 2;
        const HOOK_ATTACH_GROUND = 1 << 3;
        const HOOK_HIT_NOHOOK    = 1 << 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_input_default_is_zero() {
        let input = PlayerInput::default();
        assert_eq!(input.direction, 0);
        assert_eq!(input.jump, 0);
    }

    #[test]
    fn character_core_net_default_is_zero() {
        let core = CharacterCoreNet::default();
        assert_eq!(core.x, 0);
        assert_eq!(core.hook_state, 0);
    }

    #[test]
    fn player_flags_no_overlap() {
        assert!((PlayerFlags::ADMIN & PlayerFlags::CHATTING).is_empty());
        assert!((PlayerFlags::DEAD & PlayerFlags::BOT).is_empty());
    }

    #[test]
    fn player_flags_insert_and_contains() {
        let mut flags = PlayerFlags::default();
        flags.insert(PlayerFlags::CHATTING);
        assert!(flags.contains(PlayerFlags::CHATTING));
        assert!(!flags.contains(PlayerFlags::DEAD));
    }

    #[test]
    fn core_event_flags_no_overlap() {
        assert!((CoreEventFlags::GROUND_JUMP & CoreEventFlags::AIR_JUMP).is_empty());
        assert!((CoreEventFlags::HOOK_ATTACH_PLAYER & CoreEventFlags::HOOK_HIT_NOHOOK).is_empty());
    }
}
