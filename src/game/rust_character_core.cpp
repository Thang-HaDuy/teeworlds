#include "rust_character_core.h"

const float CCharacterCoreRust::PHYS_SIZE = 28.0f;

// ─────────────────────────────────────────────────────
// Collision callback trampolines — translate C++ CCollision calls to C ABI
// ─────────────────────────────────────────────────────

static bool s_CheckPoint(void* ud, float x, float y)
{
    return static_cast<CCollision*>(ud)->CheckPoint(x, y);
}

static int s_IntersectLine(void* ud,
    float x0, float y0, float x1, float y1,
    float* ox, float* oy)
{
    vec2 out;
    int r = static_cast<CCollision*>(ud)->IntersectLine(
        vec2(x0, y0), vec2(x1, y1), &out, nullptr);
    if(ox) *ox = out.x;
    if(oy) *oy = out.y;
    return r;
}

static void s_MoveBox(void* ud,
    float* px, float* py,
    float* vx, float* vy,
    float sx, float sy,
    float elasticity, bool* death)
{
    vec2 pos(*px, *py), vel(*vx, *vy);
    static_cast<CCollision*>(ud)->MoveBox(&pos, &vel, vec2(sx, sy), elasticity, death);
    *px = pos.x; *py = pos.y;
    *vx = vel.x; *vy = vel.y;
}

static bool s_TestBox(void* ud, float x, float y, float sx, float sy, int flag)
{
    return static_cast<CCollision*>(ud)->TestBox(vec2(x, y), vec2(sx, sy), flag);
}

// ─────────────────────────────────────────────────────
// CCharacterCoreRust
// ─────────────────────────────────────────────────────

CCharacterCoreRust::CCharacterCoreRust()
    : m_pCore(character_core_new())
    , m_pRustWorld(nullptr)
    , m_bOwnsWorld(false)
    , m_pCppWorld(nullptr)
    , m_ClientID(-1)
    , m_Pos(vec2(0,0))
    , m_Vel(vec2(0,0))
    , m_TriggeredEvents(0)
    , m_Death(false)
{
    mem_zero(&m_Input, sizeof(m_Input));
    mem_zero(&m_VTable, sizeof(m_VTable));
}

CCharacterCoreRust::~CCharacterCoreRust()
{
    // Unregister from shared WorldCore
    if(m_ClientID >= 0 && m_pRustWorld && !m_bOwnsWorld)
        world_core_set_character(m_pRustWorld, (size_t)m_ClientID, nullptr);

    if(m_bOwnsWorld)
        world_core_free(m_pRustWorld);

    character_core_free(m_pCore);
}

CCharacterCoreRust& CCharacterCoreRust::operator=(const CCharacterCoreRust& other)
{
    if(this == &other)
        return *this;
    // Copy member vars
    m_Pos             = other.m_Pos;
    m_Vel             = other.m_Vel;
    m_TriggeredEvents = other.m_TriggeredEvents;
    m_Death           = other.m_Death;
    m_Input           = other.m_Input;
    // Copy Rust physics state via netobj round-trip
    NetObjCharacterCore_C tmp;
    character_core_write(other.m_pCore, &tmp);
    character_core_read(m_pCore, &tmp);
    return *this;
}

/* static */ void CCharacterCoreRust::BuildVTable(CollisionVTable* pVT, CCollision* pCollision)
{
    pVT->userdata       = pCollision;
    pVT->check_point    = s_CheckPoint;
    pVT->intersect_line = s_IntersectLine;
    pVT->move_box       = s_MoveBox;
    pVT->test_box       = s_TestBox;
}

void CCharacterCoreRust::Init(CWorldCore* pWorld, CCollision* pCollision,
                               WorldCore_Opaque* pRustWorld, int ClientID)
{
    m_pCppWorld = pWorld;
    m_ClientID  = ClientID;

    if(pRustWorld)
    {
        m_pRustWorld = pRustWorld;
        m_bOwnsWorld = false;
    }
    else
    {
        // Standalone mode: create a private WorldCore (used by reckoning core / tests)
        m_pRustWorld = world_core_new();
        m_bOwnsWorld = true;
        // Sync default tuning from the C++ world if available
        if(pWorld)
            world_core_sync_tuning(m_pRustWorld,
                reinterpret_cast<const CTuningParams_C*>(&pWorld->m_Tuning));
    }

    BuildVTable(&m_VTable, pCollision);
    character_core_init(m_pCore, m_pRustWorld, &m_VTable);

    if(ClientID >= 0)
        world_core_set_character(m_pRustWorld, (size_t)ClientID, m_pCore);
}

void CCharacterCoreRust::Reset()
{
    character_core_reset(m_pCore);
    m_Pos             = vec2(0, 0);
    m_Vel             = vec2(0, 0);
    m_TriggeredEvents = 0;
    m_Death           = false;
    mem_zero(&m_Input, sizeof(m_Input));
}

void CCharacterCoreRust::SyncToRust()
{
    character_core_set_pos(m_pCore, m_Pos.x, m_Pos.y);
    character_core_set_vel(m_pCore, m_Vel.x, m_Vel.y);
}

void CCharacterCoreRust::SyncFromRust()
{
    float x, y;
    character_core_get_pos(m_pCore, &x, &y);  m_Pos = vec2(x, y);
    character_core_get_vel(m_pCore, &x, &y);  m_Vel = vec2(x, y);
    m_TriggeredEvents = character_core_get_triggered_events(m_pCore);
    m_Death           = character_core_get_death(m_pCore);
}

void CCharacterCoreRust::Tick(bool UseInput)
{
    // Sync latest tuning from C++ world (may have been changed via rcon)
    if(m_pCppWorld)
        world_core_sync_tuning(m_pRustWorld,
            reinterpret_cast<const CTuningParams_C*>(&m_pCppWorld->m_Tuning));

    character_core_set_input(m_pCore, &m_Input);
    SyncToRust();
    character_core_tick(m_pCore, UseInput);
    SyncFromRust();
}

void CCharacterCoreRust::Move()
{
    SyncToRust();
    character_core_move(m_pCore);
    SyncFromRust();
}

void CCharacterCoreRust::AddDragVelocity()
{
    SyncToRust();
    character_core_add_drag_velocity(m_pCore);
    float x, y;
    character_core_get_vel(m_pCore, &x, &y);
    m_Vel = vec2(x, y);
}

void CCharacterCoreRust::ResetDragVelocity()
{
    character_core_reset_drag_velocity(m_pCore);
}

void CCharacterCoreRust::Write(CNetObj_CharacterCore* pObjCore) const
{
    // character_core_write uses Rust's internal NetObjCharacterCore layout.
    // CNetObj_CharacterCore has different field order + extra m_Tick field,
    // so we map fields manually (same as original CCharacterCore::Write).
    NetObjCharacterCore_C tmp;
    character_core_write(m_pCore, &tmp);

    pObjCore->m_X           = tmp.m_x;
    pObjCore->m_Y           = tmp.m_y;
    pObjCore->m_VelX        = tmp.m_vel_x;
    pObjCore->m_VelY        = tmp.m_vel_y;
    pObjCore->m_HookState   = tmp.m_hook_state;
    pObjCore->m_HookTick    = tmp.m_hook_tick;
    pObjCore->m_HookX       = tmp.m_hook_x;
    pObjCore->m_HookY       = tmp.m_hook_y;
    pObjCore->m_HookDx      = tmp.m_hook_dx;
    pObjCore->m_HookDy      = tmp.m_hook_dy;
    pObjCore->m_HookedPlayer = tmp.m_hooked_player;
    pObjCore->m_Jumped      = tmp.m_jumped;
    pObjCore->m_Direction   = tmp.m_direction;
    pObjCore->m_Angle       = tmp.m_angle;
    // m_Tick is not written here — caller sets it (matches original behavior)
}

void CCharacterCoreRust::Read(const CNetObj_CharacterCore* pObjCore)
{
    NetObjCharacterCore_C tmp;
    tmp.m_x            = pObjCore->m_X;
    tmp.m_y            = pObjCore->m_Y;
    tmp.m_vel_x        = pObjCore->m_VelX;
    tmp.m_vel_y        = pObjCore->m_VelY;
    tmp.m_hook_state   = pObjCore->m_HookState;
    tmp.m_hook_tick    = pObjCore->m_HookTick;
    tmp.m_hook_x       = pObjCore->m_HookX;
    tmp.m_hook_y       = pObjCore->m_HookY;
    tmp.m_hook_dx      = pObjCore->m_HookDx;
    tmp.m_hook_dy      = pObjCore->m_HookDy;
    tmp.m_hooked_player = pObjCore->m_HookedPlayer;
    tmp.m_jumped       = pObjCore->m_Jumped;
    tmp.m_direction    = pObjCore->m_Direction;
    tmp.m_angle        = pObjCore->m_Angle;
    character_core_read(m_pCore, &tmp);
    SyncFromRust();
}

void CCharacterCoreRust::Quantize()
{
    SyncToRust();
    character_core_quantize(m_pCore);
    SyncFromRust();
}
