#pragma once
#ifdef __cplusplus
extern "C" {
#endif

#include <stdbool.h>
#include <stddef.h>

// ─────────────────────────────────────────────────────
// Opaque handle types
// ─────────────────────────────────────────────────────

typedef struct CharacterCore_s CharacterCore_Opaque;
typedef struct WorldCore_s     WorldCore_Opaque;

// ─────────────────────────────────────────────────────
// CollisionVTable — fill with pointers to CCollision methods.
// C++ example:
//
//   static bool cpp_check_point(void* ud, float x, float y) {
//       return static_cast<CCollision*>(ud)->CheckPoint(x, y);
//   }
//   static int cpp_intersect_line(void* ud, float x0, float y0, float x1, float y1,
//                                  float* ox, float* oy) {
//       vec2 out; int r = static_cast<CCollision*>(ud)->IntersectLine(
//           vec2(x0,y0), vec2(x1,y1), &out, 0);
//       if(ox) *ox=out.x; if(oy) *oy=out.y; return r;
//   }
//   static void cpp_move_box(void* ud, float* px, float* py,
//                             float* vx, float* vy,
//                             float sx, float sy, float e, bool* death) {
//       vec2 pos(*px,*py), vel(*vx,*vy);
//       static_cast<CCollision*>(ud)->MoveBox(&pos,&vel,vec2(sx,sy),e,death);
//       *px=pos.x; *py=pos.y; *vx=vel.x; *vy=vel.y;
//   }
//   static bool cpp_test_box(void* ud, float x, float y,
//                             float sx, float sy, int flag) {
//       return static_cast<CCollision*>(ud)->TestBox(vec2(x,y),vec2(sx,sy),flag);
//   }
// ─────────────────────────────────────────────────────

typedef struct {
    void* userdata;
    bool  (*check_point)   (void* ud, float x, float y);
    int   (*intersect_line)(void* ud, float x0, float y0, float x1, float y1,
                            float* out_x, float* out_y);
    void  (*move_box)      (void* ud, float* pos_x, float* pos_y,
                            float* vel_x, float* vel_y,
                            float size_x, float size_y,
                            float elasticity, bool* death);
    bool  (*test_box)      (void* ud, float x, float y,
                            float size_x, float size_y, int flag);
} CollisionVTable;

// ─────────────────────────────────────────────────────
// CTuningParams_C — must match C++ CTuningParams layout exactly.
// Each field is the raw int stored by CTuneParam (value * 100).
// Pass &GameWorld()->m_Core.m_Tuning cast to CTuningParams_C*.
// ─────────────────────────────────────────────────────

typedef struct {
    int ground_control_speed;
    int ground_control_accel;
    int ground_friction;
    int ground_jump_impulse;
    int air_jump_impulse;
    int air_control_speed;
    int air_control_accel;
    int air_friction;
    int hook_length;
    int hook_fire_speed;
    int hook_drag_accel;
    int hook_drag_speed;
    int gravity;
    int velramp_start;
    int velramp_range;
    int velramp_curvature;
    int gun_curvature;
    int gun_speed;
    int gun_lifetime;
    int shotgun_curvature;
    int shotgun_speed;
    int shotgun_speeddiff;
    int shotgun_lifetime;
    int grenade_curvature;
    int grenade_speed;
    int grenade_lifetime;
    int laser_reach;
    int laser_bounce_delay;
    int laser_bounce_num;
    int laser_bounce_cost;
    int player_collision;
    int player_hooking;
} CTuningParams_C;

// ─────────────────────────────────────────────────────
// NetObjCharacterCore — same layout as CNetObj_CharacterCore
// ─────────────────────────────────────────────────────

typedef struct {
    int m_x, m_y;
    int m_vel_x, m_vel_y;
    int m_hook_state, m_hook_tick;
    int m_hook_x, m_hook_y, m_hook_dx, m_hook_dy;
    int m_hooked_player;
    int m_jumped, m_direction, m_angle;
} NetObjCharacterCore_C;

// ─────────────────────────────────────────────────────
// WorldCore FFI
// ─────────────────────────────────────────────────────

WorldCore_Opaque* world_core_new(void);
void              world_core_free(WorldCore_Opaque* world);

/// Sync all tuning values from C++ CTuningParams into the Rust WorldCore.
/// Call once per tick (or when tuning changes).
void world_core_sync_tuning(WorldCore_Opaque* world, const CTuningParams_C* tuning);

/// Register a CharacterCore at client slot `id`. Pass NULL to unregister.
void world_core_set_character(WorldCore_Opaque* world, size_t id,
                               CharacterCore_Opaque* core);

// ─────────────────────────────────────────────────────
// CharacterCore FFI
// ─────────────────────────────────────────────────────

CharacterCore_Opaque* character_core_new(void);
void                  character_core_free(CharacterCore_Opaque* core);

/// Connect the core to a WorldCore and C++ collision.
/// vtable may be NULL only in unit tests (collision becomes no-op).
void character_core_init(CharacterCore_Opaque* core,
                          WorldCore_Opaque*     world,
                          const CollisionVTable* vtable);

void character_core_reset(CharacterCore_Opaque* core);

/// Copy a CNetObj_PlayerInput into the core before calling tick.
void character_core_set_input(CharacterCore_Opaque*     core,
                               const void*               input_ptr);

void character_core_tick              (CharacterCore_Opaque* core, bool use_input);
void character_core_add_drag_velocity (CharacterCore_Opaque* core);
void character_core_reset_drag_velocity(CharacterCore_Opaque* core);
void character_core_move              (CharacterCore_Opaque* core);

/// Serialize physics state to a CNetObj_CharacterCore-compatible buffer.
void character_core_write(const CharacterCore_Opaque* core, NetObjCharacterCore_C* out);
void character_core_read (CharacterCore_Opaque* core, const NetObjCharacterCore_C* obj);
void character_core_quantize(CharacterCore_Opaque* core);

// Field accessors
void  character_core_get_pos(const CharacterCore_Opaque* core, float* x, float* y);
void  character_core_set_pos(CharacterCore_Opaque* core, float x, float y);
void  character_core_get_vel(const CharacterCore_Opaque* core, float* x, float* y);
void  character_core_set_vel(CharacterCore_Opaque* core, float x, float y);
void  character_core_add_vel(CharacterCore_Opaque* core, float dx, float dy);
int   character_core_get_triggered_events(const CharacterCore_Opaque* core);
bool  character_core_get_death(const CharacterCore_Opaque* core);

#ifdef __cplusplus
}
#endif
