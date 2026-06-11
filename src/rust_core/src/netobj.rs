// src/netobj.rs
#[derive(Debug, Default, Clone, Copy)]
pub struct PlayerInput {
    pub m_TargetX: i32,
    pub m_TargetY: i32,
    pub m_Direction: i32,
    pub m_Jump: bool,
    pub m_Hook: bool,
}