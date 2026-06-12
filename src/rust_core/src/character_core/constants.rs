pub const PHYS_SIZE: f32 = 28.0;
pub const MAX_CLIENTS: usize = 64;
pub const SERVER_TICK_SPEED: i32 = 50;

// Hook states — mirrors C++ enum in gamecore.h
pub const HOOK_RETRACTED: i32 = -1;
pub const HOOK_IDLE: i32 = 0;
pub const HOOK_RETRACT_START: i32 = 1;
pub const HOOK_RETRACT_END: i32 = 3;
pub const HOOK_FLYING: i32 = 4;
pub const HOOK_GRABBED: i32 = 5;

// Core event flags — from datasrc/network.py CoreEventFlags
pub const COREEVENTFLAG_GROUND_JUMP: i32 = 1 << 0;
pub const COREEVENTFLAG_AIR_JUMP: i32 = 1 << 1;
pub const COREEVENTFLAG_HOOK_ATTACH_PLAYER: i32 = 1 << 2;
pub const COREEVENTFLAG_HOOK_ATTACH_GROUND: i32 = 1 << 3;
pub const COREEVENTFLAG_HOOK_HIT_NOHOOK: i32 = 1 << 4;
