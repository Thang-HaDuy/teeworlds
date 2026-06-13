use std::ffi::c_void;
use crate::vec2::Vec2;

pub const COLFLAG_SOLID: i32 = 1;
pub const COLFLAG_DEATH: i32 = 2;
pub const COLFLAG_NOHOOK: i32 = 4;

/// C-compatible vtable for calling back into C++ CCollision.
/// C++ fills this struct and passes it to character_core_init().
#[repr(C)]
#[derive(Clone, Copy)]
pub struct CollisionVTable {
    pub userdata: *mut c_void,
    /// bool CheckPoint(float x, float y)  — always checks COLFLAG_SOLID
    pub check_point: unsafe extern "C" fn(*mut c_void, f32, f32) -> bool,
    /// int IntersectLine(x0,y0, x1,y1, out_x,out_y) — returns COLFLAG_* bits
    pub intersect_line: unsafe extern "C" fn(*mut c_void, f32, f32, f32, f32, *mut f32, *mut f32) -> i32,
    /// void MoveBox(inout_pos_x,inout_pos_y, inout_vel_x,inout_vel_y, size_x,size_y, elasticity, death_out)
    pub move_box: unsafe extern "C" fn(*mut c_void, *mut f32, *mut f32, *mut f32, *mut f32, f32, f32, f32, *mut bool),
    /// bool TestBox(x, y, size_x, size_y, flag)
    pub test_box: unsafe extern "C" fn(*mut c_void, f32, f32, f32, f32, i32) -> bool,
}

pub struct Collision {
    vtable: Option<CollisionVTable>,
}

impl Collision {
    pub fn new() -> Self {
        Self { vtable: None }
    }

    pub fn set_vtable(&mut self, vt: CollisionVTable) {
        self.vtable = Some(vt);
    }

    pub fn check_point(&self, x: f32, y: f32) -> bool {
        match self.vtable {
            Some(vt) => unsafe { (vt.check_point)(vt.userdata, x, y) },
            None => false,
        }
    }

    pub fn check_point_flag(&self, x: f32, y: f32, flag: i32) -> bool {
        // Simplified: delegates to check_point (solid check only via vtable).
        // Flag-specific checks are only needed internally for death tiles (test_box).
        let _ = flag;
        self.check_point(x, y)
    }

    /// Returns (hit_solid, hit_nohook, collision_point).
    pub fn intersect_line(&self, p0: Vec2, p1: Vec2) -> (bool, bool, Vec2) {
        match self.vtable {
            Some(vt) => unsafe {
                let mut out_x = p1.x;
                let mut out_y = p1.y;
                let flags = (vt.intersect_line)(
                    vt.userdata,
                    p0.x, p0.y, p1.x, p1.y,
                    &mut out_x, &mut out_y,
                );
                let hit_solid = flags & COLFLAG_SOLID != 0;
                let hit_nohook = flags & COLFLAG_NOHOOK != 0;
                (hit_solid && !hit_nohook, hit_nohook, Vec2 { x: out_x, y: out_y })
            },
            None => (false, false, p1),
        }
    }

    pub fn test_box(&self, pos: Vec2, size: Vec2, col_flags: i32) -> bool {
        match self.vtable {
            Some(vt) => unsafe {
                (vt.test_box)(vt.userdata, pos.x, pos.y, size.x, size.y, col_flags)
            },
            None => false,
        }
    }

    pub fn move_box(
        &self,
        inout_pos: &mut Vec2,
        inout_vel: &mut Vec2,
        size: Vec2,
        elasticity: f32,
        death: Option<&mut bool>,
    ) {
        match self.vtable {
            Some(vt) => unsafe {
                let mut dead = false;
                let death_ptr = match death {
                    Some(d) => d as *mut bool,
                    None => &mut dead as *mut bool,
                };
                (vt.move_box)(
                    vt.userdata,
                    &mut inout_pos.x, &mut inout_pos.y,
                    &mut inout_vel.x, &mut inout_vel.y,
                    size.x, size.y,
                    elasticity,
                    death_ptr,
                );
            },
            None => {}
        }
    }
}
