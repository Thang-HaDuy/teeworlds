use crate::vec2::Vec2;

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
    pub target_x: i32,
    pub target_y: i32,
    pub direction: i32,
    pub jump: bool,
    pub hook: bool,
}

impl PlayerInput {
    pub fn get_target_direction(&self) -> Vec2 {
        let v = Vec2 { x: self.target_x as f32, y: self.target_y as f32 };
        let len = v.length();
        if len > 0.0001 { v * (1.0 / len) } else { Vec2 { x: 0.0, y: -1.0 } }
    }
}
