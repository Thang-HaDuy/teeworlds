// src/character_core.rs

use crate::world_core::WorldCore;
use crate::collision::Collision;
use crate::netobj::PlayerInput;

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug)]
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