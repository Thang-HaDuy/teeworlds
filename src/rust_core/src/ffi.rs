/// FFI layer — C++ calls these functions to use Rust CharacterCore.
///
/// Usage from C++:
///   1. Create one WorldCore_Opaque per game world via world_core_new()
///   2. Build a CollisionVTable with pointers to your CCollision methods
///   3. For each player: character_core_new(), character_core_init(core, world, &vtable)
///   4. Register each core: world_core_set_character(world, cid, core)
///   5. Before each tick: world_core_sync_tuning(world, &m_Tuning)
///   6. Per tick: character_core_set_input, character_core_tick, character_core_move
///   7. character_core_write / character_core_read / character_core_quantize for net sync

use crate::character_core::CharacterCore;
use crate::world_core::WorldCore;
use crate::collision::CollisionVTable;
use crate::netobj::NetObjCharacterCore;

// ─────────────────────────────────────────────────────
// C-compatible player input (matches CNetObj_PlayerInput)
// ─────────────────────────────────────────────────────

#[repr(C)]
pub struct CNetObjPlayerInput {
    pub m_direction:      i32,
    pub m_target_x:       i32,
    pub m_target_y:       i32,
    pub m_jump:           i32,
    pub m_fire:           i32,
    pub m_hook:           i32,
    pub m_player_flags:   i32,
    pub m_wanted_weapon:  i32,
    pub m_next_weapon:    i32,
    pub m_prev_weapon:    i32,
}

// ─────────────────────────────────────────────────────
// CTuningParams_C — mirrors C++ CTuningParams layout.
// Each field is stored as int (value * 100) in C++.
// Field order must match tuning.h MACRO_TUNING_PARAM order exactly.
// ─────────────────────────────────────────────────────

#[repr(C)]
pub struct CTuningParams_C {
    pub ground_control_speed: i32,
    pub ground_control_accel: i32,
    pub ground_friction:      i32,
    pub ground_jump_impulse:  i32,
    pub air_jump_impulse:     i32,
    pub air_control_speed:    i32,
    pub air_control_accel:    i32,
    pub air_friction:         i32,
    pub hook_length:          i32,
    pub hook_fire_speed:      i32,
    pub hook_drag_accel:      i32,
    pub hook_drag_speed:      i32,
    pub gravity:              i32,
    pub velramp_start:        i32,
    pub velramp_range:        i32,
    pub velramp_curvature:    i32,
    // Weapon params — not used for physics but must be present to match C++ struct layout.
    pub gun_curvature:        i32,
    pub gun_speed:            i32,
    pub gun_lifetime:         i32,
    pub shotgun_curvature:    i32,
    pub shotgun_speed:        i32,
    pub shotgun_speeddiff:    i32,
    pub shotgun_lifetime:     i32,
    pub grenade_curvature:    i32,
    pub grenade_speed:        i32,
    pub grenade_lifetime:     i32,
    pub laser_reach:          i32,
    pub laser_bounce_delay:   i32,
    pub laser_bounce_num:     i32,
    pub laser_bounce_cost:    i32,
    pub player_collision:     i32,
    pub player_hooking:       i32,
}

// ─────────────────────────────────────────────────────
// WorldCore FFI
// ─────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn world_core_new() -> *mut WorldCore {
    Box::into_raw(Box::new(WorldCore::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn world_core_free(ptr: *mut WorldCore) {
    unsafe {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Sync tuning values from C++ CTuningParams into the Rust WorldCore.
/// Must be called once before each simulation tick when tuning may have changed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn world_core_sync_tuning(
    world: *mut WorldCore,
    tuning: *const CTuningParams_C,
) {
    unsafe {
        let w = &mut *world;
        let t = &*tuning;
        // CTuneParam stores value as int(value * 100) — divide to get float.
        w.tuning.ground_control_speed = t.ground_control_speed as f32 / 100.0;
        w.tuning.ground_control_accel = t.ground_control_accel as f32 / 100.0;
        w.tuning.ground_friction      = t.ground_friction      as f32 / 100.0;
        w.tuning.ground_jump_impulse  = t.ground_jump_impulse  as f32 / 100.0;
        w.tuning.air_jump_impulse     = t.air_jump_impulse     as f32 / 100.0;
        w.tuning.air_control_speed    = t.air_control_speed    as f32 / 100.0;
        w.tuning.air_control_accel    = t.air_control_accel    as f32 / 100.0;
        w.tuning.air_friction         = t.air_friction         as f32 / 100.0;
        w.tuning.hook_length          = t.hook_length          as f32 / 100.0;
        w.tuning.hook_fire_speed      = t.hook_fire_speed      as f32 / 100.0;
        w.tuning.hook_drag_accel      = t.hook_drag_accel      as f32 / 100.0;
        w.tuning.hook_drag_speed      = t.hook_drag_speed      as f32 / 100.0;
        w.tuning.gravity              = t.gravity              as f32 / 100.0;
        w.tuning.velramp_start        = t.velramp_start        as f32 / 100.0;
        w.tuning.velramp_range        = t.velramp_range        as f32 / 100.0;
        w.tuning.velramp_curvature    = t.velramp_curvature    as f32 / 100.0;
        w.tuning.player_collision     = t.player_collision != 0;
        w.tuning.player_hooking       = t.player_hooking  != 0;
    }
}

/// Register (or unregister with null) a CharacterCore at a client slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn world_core_set_character(
    world: *mut WorldCore,
    id: usize,
    core: *mut CharacterCore,
) {
    unsafe {
        if id < crate::character_core::MAX_CLIENTS {
            (*world).ap_characters[id] = core;
        }
    }
}

// ─────────────────────────────────────────────────────
// CharacterCore FFI
// ─────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub extern "C" fn character_core_new() -> *mut CharacterCore {
    Box::into_raw(Box::new(CharacterCore::new()))
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_free(ptr: *mut CharacterCore) {
    unsafe {
        if !ptr.is_null() {
            drop(Box::from_raw(ptr));
        }
    }
}

/// Initialize the core with a shared WorldCore pointer and a C++ CCollision vtable.
/// `vtable` may be null (testing only — collision will be no-op).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_init(
    core: *mut CharacterCore,
    world: *mut WorldCore,
    vtable: *const CollisionVTable,
) {
    unsafe {
        let c = &mut *core;
        let vt = if vtable.is_null() { None } else { Some(*vtable) };
        c.init(world, vt);
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_reset(core: *mut CharacterCore) {
    unsafe { (*core).reset(); }
}

/// Copy input from a CNetObj_PlayerInput into the core.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_set_input(
    core: *mut CharacterCore,
    input: *const CNetObjPlayerInput,
) {
    unsafe {
        let c = &mut *core;
        let i = &*input;
        c.input.direction = i.m_direction;
        c.input.target_x  = i.m_target_x;
        c.input.target_y  = i.m_target_y;
        c.input.jump      = i.m_jump != 0;
        c.input.hook      = i.m_hook != 0;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_tick(core: *mut CharacterCore, use_input: bool) {
    unsafe { (*core).tick(use_input); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_add_drag_velocity(core: *mut CharacterCore) {
    unsafe { (*core).add_drag_velocity(); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_reset_drag_velocity(core: *mut CharacterCore) {
    unsafe { (*core).reset_drag_velocity(); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_move(core: *mut CharacterCore) {
    unsafe { (*core).move_core(); }
}

/// Write physics state to a CNetObj_CharacterCore (same binary layout as NetObjCharacterCore).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_write(
    core: *const CharacterCore,
    out: *mut NetObjCharacterCore,
) {
    unsafe { (*core).write_netobj(&mut *out); }
}

/// Read physics state from a CNetObj_CharacterCore.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_read(
    core: *mut CharacterCore,
    obj: *const NetObjCharacterCore,
) {
    unsafe { (*core).read_netobj(&*obj); }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_quantize(core: *mut CharacterCore) {
    unsafe { (*core).quantize(); }
}

// ─────────────────────────────────────────────────────
// Field accessors (mirrors direct field access in C++ CCharacterCore)
// ─────────────────────────────────────────────────────

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_get_pos(
    core: *const CharacterCore, x: *mut f32, y: *mut f32,
) {
    unsafe {
        *x = (*core).pos.x;
        *y = (*core).pos.y;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_set_pos(core: *mut CharacterCore, x: f32, y: f32) {
    unsafe {
        (*core).pos.x = x;
        (*core).pos.y = y;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_get_vel(
    core: *const CharacterCore, x: *mut f32, y: *mut f32,
) {
    unsafe {
        *x = (*core).vel.x;
        *y = (*core).vel.y;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_set_vel(core: *mut CharacterCore, x: f32, y: f32) {
    unsafe {
        (*core).vel.x = x;
        (*core).vel.y = y;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_add_vel(core: *mut CharacterCore, dx: f32, dy: f32) {
    unsafe {
        (*core).vel.x += dx;
        (*core).vel.y += dy;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_get_triggered_events(core: *const CharacterCore) -> i32 {
    unsafe { (*core).triggered_events }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn character_core_get_death(core: *const CharacterCore) -> bool {
    unsafe { (*core).death }
}
