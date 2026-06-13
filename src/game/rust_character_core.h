#pragma once

#include <game/gamecore.h>
#include <game/collision.h>
#include <generated/protocol.h>
#include <rust_core/include/rust_character_core.h>

/// C++ wrapper around the Rust CharacterCore.
/// Drop-in replacement for CCharacterCore — same public interface.
class CCharacterCoreRust
{
public:
    static const float PHYS_SIZE; // = 28.0f

    // Public fields mirroring CCharacterCore (character.cpp accesses these directly)
    vec2                m_Pos;
    vec2                m_Vel;
    int                 m_TriggeredEvents;
    bool                m_Death;
    CNetObj_PlayerInput m_Input;

    CCharacterCoreRust();
    ~CCharacterCoreRust();

    // Copy constructor deleted — use operator= to copy only physics state
    CCharacterCoreRust(const CCharacterCoreRust&) = delete;
    /// Copies physics state (pos/vel/hook/…) from other into this core.
    /// Keeps this core's own WorldCore and Collision intact.
    CCharacterCoreRust& operator=(const CCharacterCoreRust& other);

    /// pRustWorld: shared Rust WorldCore for player-player collision.
    ///             Pass nullptr for standalone cores (reckoning, tests).
    /// ClientID:   slot index used to register this core in pRustWorld.
    ///             Pass -1 for standalone cores.
    void Init(CWorldCore* pWorld, CCollision* pCollision,
              WorldCore_Opaque* pRustWorld = nullptr, int ClientID = -1);

    void Reset();
    void Tick(bool UseInput);
    void Move();
    void AddDragVelocity();
    void ResetDragVelocity();

    /// Write physics state into a CNetObj_CharacterCore for network snapshots.
    void Write(CNetObj_CharacterCore* pObjCore) const;
    /// Read physics state from a CNetObj_CharacterCore.
    void Read(const CNetObj_CharacterCore* pObjCore);
    void Quantize();

    CharacterCore_Opaque* GetRustCore() const { return m_pCore; }

private:
    CharacterCore_Opaque* m_pCore;
    WorldCore_Opaque*     m_pRustWorld;
    bool                  m_bOwnsWorld; // true → we created m_pRustWorld, we free it
    CWorldCore*           m_pCppWorld;  // for tuning sync before each tick
    CollisionVTable       m_VTable;
    int                   m_ClientID;

    // Push m_Pos / m_Vel into Rust (e.g. after ninja code modifies them directly)
    void SyncToRust();
    // Pull pos / vel / triggered_events / death from Rust into member vars
    void SyncFromRust();

    static void BuildVTable(CollisionVTable* pVT, CCollision* pCollision);
};
