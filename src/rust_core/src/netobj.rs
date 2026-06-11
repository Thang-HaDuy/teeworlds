// src/netobj.rs

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NetObjCharacterCore {
    pub m_x: i32,
    pub m_y: i32,

    pub m_vel_x: i32,
    pub m_vel_y: i32,

    pub m_hook_state: i32,
    pub m_hook_tick: i32,
    pub m_hook_x: i32,
    pub m_hook_y: i32,
    pub m_hook_dx: i32,
    pub m_hook_dy: i32,
    pub m_hooked_player: i32,

    pub m_jumped: i32,
    pub m_direction: i32,
    pub m_angle: i32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerInput {
    pub m_TargetX: i32,
    pub m_TargetY: i32,
    pub m_Direction: i32,
    pub m_Jump: bool,
    pub m_Hook: bool,
}