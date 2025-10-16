# Phase 8.3: Save/Load System Implementation Plan

**Document Version**: 1.0  
**Date**: October 14, 2025  
**Duration**: 2-3 weeks  
**Dependencies**: None (can run in parallel with Phase 8.1-8.2)

---

## Executive Summary

**Mission**: Implement production-quality save/load system enabling player progression, game state persistence, and replay functionality.

**Current State**:
- ✅ **Editor Level Saves**: Editor can save/load levels as TOML/JSON
- ✅ **Deterministic ECS**: BTreeMap-based entity iteration (perfect for serialization)
- ✅ **RON Support**: Rust Object Notation used in asset pipeline
- ❌ **Player Save System**: Not implemented
- ❌ **Save Slot Management**: Not implemented
- ❌ **Save Versioning**: Not implemented

**Target State** (Phase 8 Complete):
- ✅ Full ECS world serialization (all entities, components, systems)
- ✅ Player profile system (settings, unlocks, stats, inventory)
- ✅ Save slot management (3-10 slots, metadata, thumbnails)
- ✅ Save versioning & migration (forward/backward compatibility)
- ✅ Corruption detection & recovery (checksums, backups)
- ✅ Replay system (deterministic replay from save files)

**Timeline**: 2-3 weeks (10-15 working days)

**Success Criteria**: Veilweaver Demo Level can be saved/loaded with full state preservation

---

## Week 1: ECS World Serialization

**Goal**: Serialize entire ECS world to disk, enabling full game state saves

### Day 1-2: Component Serialization Infrastructure

**Tasks**:
1. **Derive Macros for Components**:
   - Add `#[derive(Serialize, Deserialize)]` to all component types
   - File audit: `astraweave-ecs/src/components/*.rs`
   - Count: ~30-50 component types (Position, Velocity, Health, etc.)
   - Validate: All components compile with serde derives

2. **Handle Non-Serializable Types**:
   - **Problem**: Some types can't be serialized (e.g., GPU handles, file handles)
   - **Strategy**: Use `#[serde(skip)]` for transient state
   - **Examples**:
     - `TextureHandle`: Don't serialize GPU handles (reload from asset path)
     - `AudioSink`: Don't serialize audio playback state (restart on load)
     - `PhysicsHandle`: Don't serialize Rapier handles (rebuild from component data)
   - **Pattern**: Serialize "data", not "handles"

3. **Custom Serialization for Complex Types**:
   - **Example 1**: `Entity` IDs
     - Problem: Entity IDs may not be stable across save/load
     - Solution: Remap entity IDs during deserialization
     - Implementation: `SerializedEntity` wrapper with ID remapping
   - **Example 2**: `TypeId` for components
     - Problem: TypeId not stable across Rust versions
     - Solution: Use string type names (e.g., "Position", "Velocity")
     - Implementation: `ComponentRegistry` with type name → TypeId mapping

**Deliverables**:
- All components have `Serialize + Deserialize` derives
- Transient state marked with `#[serde(skip)]`
- Custom serialization for Entity IDs and TypeIds

**Success Criteria**:
- ✅ All component types serialize/deserialize correctly
- ✅ No compilation errors with serde derives
- ✅ Unit tests: Serialize → Deserialize → Compare (roundtrip test)

---

### Day 3-4: Archetype & World Serialization

**Tasks**:
1. **Archetype Serialization**:
   - File: `astraweave-ecs/src/archetype.rs`
   - Data to save:
     - Component type list (e.g., ["Position", "Velocity", "Health"])
     - Entity list (all entities in this archetype)
     - Component data (packed arrays)
   - Format: RON (human-readable) or bincode (compact binary)
   - Validate: Archetypes serialize with correct structure

2. **World Serialization**:
   - File: `astraweave-ecs/src/world.rs`
   - Data to save:
     - All archetypes (entities + components)
     - Next entity ID (for ID stability)
     - System state (e.g., frame counter, tick count)
     - Resources (global singletons like settings, score)
   - API: `world.save_to_file(path)` and `World::load_from_file(path)`
   - Validate: Full world state serializes correctly

3. **Entity ID Remapping**:
   - **Problem**: Entity IDs may conflict if loading into non-empty world
   - **Solution**: Remap all entity IDs during deserialization
   - **Algorithm**:
     1. Build map: `old_id → new_id`
     2. For each entity: Allocate new ID
     3. For each component with entity references: Remap using map
   - **Example**: Parent/child entity references in transform hierarchy
   - Validate: Entity references preserved after load

**Deliverables**:
- `world.save_to_file(path)` API
- `World::load_from_file(path)` API
- Entity ID remapping system

**Success Criteria**:
- ✅ Full world state saves to disk (all entities, components)
- ✅ World loads correctly (same state after load)
- ✅ Entity references preserved (parent/child, AI targets, etc.)

---

### Day 5: Testing & Validation

**Tasks**:
1. **Unit Tests**:
   - Test: Empty world save/load
   - Test: World with 1,000 entities save/load
   - Test: World with complex entity references (parent/child hierarchy)
   - Test: Roundtrip stability (save → load → save → compare)

2. **Integration Tests**:
   - Test: `hello_companion` save/load (AI state preserved)
   - Test: `unified_showcase` save/load (terrain, materials preserved)
   - Validate: No data loss, no corruption

3. **Performance Validation**:
   - Measure: Serialize time for 10,000 entities
   - Target: <100ms for save, <200ms for load
   - Format comparison: RON (human-readable) vs bincode (compact)
   - Decision: Use bincode for production (10× smaller, 5× faster)

**Deliverables**:
- 10+ unit tests for ECS serialization
- 2+ integration tests for examples
- Performance benchmarks

**Success Criteria**:
- ✅ All tests pass (100% success rate)
- ✅ Save/load <300ms total for typical game (10,000 entities)
- ✅ No data loss or corruption

---

## Week 2: Player Profile & Save Slots

**Goal**: Implement player save system with multiple save slots

### Day 6-7: Player Profile System

**Tasks**:
1. **Profile Data Structure**:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct PlayerProfile {
       pub name: String,
       pub settings: GameSettings,
       pub stats: PlayerStats,
       pub unlocks: Unlocks,
       pub inventory: Inventory,
       pub quest_progress: QuestProgress,
   }

   pub struct GameSettings {
       pub audio: AudioSettings,     // Volume, mute, etc.
       pub graphics: GraphicsSettings, // Resolution, quality, etc.
       pub controls: ControlSettings,  // Keybindings, sensitivity, etc.
   }

   pub struct PlayerStats {
       pub playtime: Duration,
       pub enemies_defeated: u32,
       pub deaths: u32,
       pub achievements: Vec<String>,
   }

   pub struct Unlocks {
       pub abilities: Vec<String>,
       pub items: Vec<String>,
       pub levels: Vec<String>,
   }
   ```

2. **Profile Serialization**:
   - Format: RON (human-readable for user editing)
   - Path: `saves/player_profile.ron`
   - Auto-save: Save on every unlock, achievement, stat change
   - Validate: Profile saves/loads correctly

3. **Settings Integration**:
   - Integrate with Phase 8.1 UI (settings menu)
   - Integrate with Phase 8.4 Audio (audio mixer)
   - Integrate with Phase 8.2 Rendering (graphics quality)
   - Validate: Settings persist across sessions

**Deliverables**:
- `PlayerProfile` struct with serde derives
- `profile.save()` and `PlayerProfile::load()` APIs
- Integration with settings systems

**Success Criteria**:
- ✅ Player profile saves on every change
- ✅ Settings persist across game sessions
- ✅ Stats and unlocks preserved

---

### Day 8-9: Save Slot Management

**Tasks**:
1. **Save Slot Data Structure**:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SaveSlot {
       pub id: u32,                    // Slot number (1-10)
       pub metadata: SaveMetadata,
       pub world_state: WorldSnapshot, // ECS world
       pub player_state: PlayerState,  // Inventory, health, etc.
   }

   pub struct SaveMetadata {
       pub timestamp: DateTime<Utc>,
       pub playtime: Duration,
       pub level_name: String,
       pub checkpoint: String,
       pub thumbnail: Option<PathBuf>, // Screenshot
   }
   ```

2. **Save Slot API**:
   - `SaveManager::save_game(slot_id, world, player)` → Save to slot
   - `SaveManager::load_game(slot_id)` → Load from slot
   - `SaveManager::delete_slot(slot_id)` → Delete save
   - `SaveManager::list_slots()` → List all saves with metadata
   - Path: `saves/slot_<id>.sav` (bincode format for production)
   - Validate: All APIs work correctly

3. **Screenshot Thumbnails**:
   - Capture: Take screenshot before saving (64x64 or 128x128)
   - Format: PNG (compressed, small file size)
   - Storage: `saves/slot_<id>_thumb.png`
   - UI: Display thumbnail in load game menu (Phase 8.1)

**Deliverables**:
- `SaveManager` struct with save/load/delete APIs
- Save slot metadata system
- Screenshot thumbnail generation

**Success Criteria**:
- ✅ 3-10 save slots supported
- ✅ Save metadata displays in UI (timestamp, level, playtime)
- ✅ Screenshot thumbnails generated automatically

---

### Day 10: Save Slot UI Integration

**Tasks**:
1. **Load Game Menu** (Phase 8.1 integration):
   - UI: Grid of save slots with thumbnails
   - Display: Timestamp, level name, playtime
   - Actions: Load, delete, overwrite
   - Validation: Empty slots show "New Game"

2. **Save Game Menu**:
   - UI: Quick save (F5) and manual save (menu)
   - Prompt: "Overwrite slot X?" if already exists
   - Auto-save: Save on checkpoint, level transition
   - Validation: Player can save/load from UI

3. **Save Corruption Handling**:
   - Detection: Checksum validation (CRC32 or SHA256)
   - Recovery: Fallback to backup save (slot_X.sav.bak)
   - UI: Display error message if save corrupted
   - Validation: Corrupted saves don't crash game

**Deliverables**:
- Load game menu with save slot grid
- Save game menu with quick save
- Corruption detection & recovery

**Success Criteria**:
- ✅ Player can save/load from in-game menu
- ✅ Quick save works (F5 hotkey)
- ✅ Corrupted saves detected and recovered

---

## Week 3: Versioning, Migration & Replay

**Goal**: Handle save format changes and enable deterministic replay

### Day 11-12: Save Versioning & Migration

**Tasks**:
1. **Version Metadata**:
   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SaveFileHeader {
       pub magic: [u8; 4],          // "ASTR" magic number
       pub version: u32,            // Save format version
       pub game_version: String,    // e.g., "0.8.0"
       pub checksum: [u8; 32],      // SHA256 checksum
   }
   ```

2. **Migration System**:
   - **Problem**: Component fields change over time (e.g., adding new stats)
   - **Solution**: Migrate old saves to new format
   - **Example**:
     ```rust
     fn migrate_v1_to_v2(old_save: SaveV1) -> SaveV2 {
         // Add new fields with default values
         SaveV2 {
             world: old_save.world,
             player: PlayerState {
                 health: old_save.player.health,
                 mana: 100.0, // NEW FIELD: Default value
             },
         }
     }
     ```
   - **Registry**: `MigrationRegistry` maps version → migration function
   - Validate: Old saves load correctly with migration

3. **Forward Compatibility** (Optional):
   - **Problem**: Loading new saves in old game versions
   - **Solution**: Strip unknown fields, warn user
   - **Implementation**: Use `#[serde(default)]` for new fields
   - Validate: New saves load in old versions (degraded, but safe)

**Deliverables**:
- Save file header with version metadata
- Migration system for version upgrades
- Optional: Forward compatibility warnings

**Success Criteria**:
- ✅ Old saves migrate automatically to new format
- ✅ Save version displayed in UI (e.g., "Save v3, Game v0.8.0")
- ✅ Migration errors handled gracefully (fallback to new game)

---

### Day 13-14: Replay System (Deterministic)

**Tasks**:
1. **Input Recording**:
   - Record: All player inputs (keyboard, mouse, gamepad)
   - Format: `Vec<(frame, input_event)>`
   - Storage: Append to save file or separate replay file
   - Validate: Inputs recorded every frame

2. **Deterministic Replay**:
   - **Prerequisite**: Deterministic ECS (already implemented!)
   - **Algorithm**:
     1. Load save file (initial state)
     2. For each frame: Apply recorded input
     3. Tick game simulation (deterministic)
     4. Validate: Replay produces identical results
   - **Use cases**:
     - Bug reproduction (send replay file to devs)
     - Speedrunning (verify no cheating)
     - AI training (record human gameplay)

3. **Replay Validation**:
   - Test: Record 1,000 frames, replay, compare final state
   - Checksum: Hash final world state (should match)
   - Divergence detection: Flag if replay diverges from recorded state
   - Validate: 100% determinism (same input → same output)

**Deliverables**:
- Input recording system
- Deterministic replay playback
- Replay validation tests

**Success Criteria**:
- ✅ 100% deterministic replay (no RNG drift)
- ✅ Replay files <1MB for 10,000 frames
- ✅ Replay speed: 1× (real-time) or 10× (fast-forward)

---

### Day 15: Backup & Recovery System

**Tasks**:
1. **Auto-Backup**:
   - Strategy: Keep last N backups (e.g., 3-5)
   - Naming: `slot_1.sav`, `slot_1.sav.bak1`, `slot_1.sav.bak2`, etc.
   - Rotation: Delete oldest backup when saving new one
   - Validate: Backups created on every save

2. **Corruption Recovery**:
   - Detection: Checksum mismatch on load
   - Recovery: Try backups in order (bak1 → bak2 → bak3)
   - UI: "Save corrupted, loaded from backup (5 minutes old)"
   - Fallback: If all backups corrupted, offer "New Game"

3. **Cloud Save Integration** (Optional, defer to Phase 9):
   - Platform: Steam Cloud, Epic Cloud, or custom backend
   - Sync: Upload saves after every save, download on game start
   - Conflict resolution: Use latest timestamp
   - Defer: Complex, not critical for Phase 8

**Deliverables**:
- Auto-backup system with rotation
- Corruption recovery with UI notifications

**Success Criteria**:
- ✅ Backups created automatically (last 3-5 saves)
- ✅ Corrupted saves recover from backup
- ✅ No data loss even if primary save corrupted

---

## Integration & Testing

### Testing Strategy

**Unit Tests**:
- Component serialization roundtrip (30+ components)
- World serialization roundtrip (empty, 1,000 entities, 10,000 entities)
- Save slot management (save, load, delete, list)
- Version migration (v1 → v2, v2 → v3)
- Replay determinism (1,000 frame replay)

**Integration Tests**:
- `hello_companion`: Save/load AI state (behavior tree, GOAP plan)
- `unified_showcase`: Save/load terrain, materials, lighting
- Veilweaver Demo: Save/load full game state (combat, inventory, quests)

**Stress Tests**:
- Save 100,000 entities (performance test)
- Load 100 times in a row (memory leak test)
- Corrupt save file (recovery test)
- Migrate from v1 save (100 versions old)

**Manual Tests**:
- Player saves in middle of combat → loads correctly
- Player saves with full inventory → all items preserved
- Player saves at checkpoint → respawns at checkpoint on load
- Player changes settings → settings persist after restart

---

## Success Criteria (Phase 8.3 Complete)

### Functionality

- ✅ **Full ECS Serialization**: All entities, components, systems save/load correctly
- ✅ **Player Profile**: Settings, stats, unlocks, inventory persist across sessions
- ✅ **Save Slots**: 3-10 slots with metadata (timestamp, level, playtime, thumbnail)
- ✅ **Versioning**: Old saves migrate automatically to new format
- ✅ **Corruption Recovery**: Backups and checksums prevent data loss
- ✅ **Replay System**: Deterministic replay from save files (100% identical results)

### Performance

- ✅ **Save Time**: <100ms for 10,000 entities
- ✅ **Load Time**: <200ms for 10,000 entities
- ✅ **File Size**: <10MB for typical game state (bincode compression)
- ✅ **Backup Overhead**: <50ms to rotate backups

### Code Quality

- ✅ **Zero `.unwrap()`**: All save/load code uses proper error handling
- ✅ **Zero panics**: Corrupted saves handled gracefully
- ✅ **50%+ test coverage**: Unit + integration tests for save/load
- ✅ **Documentation**: API docs for SaveManager, PlayerProfile

### Integration

- ✅ **Phase 8.1 UI**: Load/save menus integrated
- ✅ **Veilweaver Demo**: Full game saves/loads correctly
- ✅ **Examples**: `hello_companion`, `unified_showcase` support save/load

---

## Dependencies & Risks

### Dependencies

**Upstream** (Blocks this work):
- None (can run in parallel with Phase 8.1-8.2)

**Downstream** (Blocked by this work):
- Veilweaver Demo Level (needs progression system)
- Multiplayer (Phase 10, needs save state for client sync)

### Risks

**High Risk**:
1. **Non-Serializable Components**: Some components can't be serialized (GPU handles, file handles)
   - Mitigation: Use `#[serde(skip)]`, rebuild transient state on load
   - Fallback: Mark as "not saveable", warn player

2. **Entity ID Stability**: Entity IDs may conflict across save/load
   - Mitigation: Remap all entity IDs during deserialization
   - Validation: Test with complex entity references (parent/child, AI targets)

**Medium Risk**:
3. **Save Format Changes**: Component fields change frequently during development
   - Mitigation: Version migration system
   - Fallback: Accept breaking changes pre-1.0, document in release notes

4. **Replay Determinism**: Hard to achieve 100% determinism (floating-point, RNG)
   - Mitigation: Already have deterministic ECS (BTreeMap, fixed-seed RNG)
   - Validation: Extensive testing with 10,000+ frame replays

**Low Risk**:
5. **File Corruption**: Disk errors, power loss during save
   - Mitigation: Checksums + backups
   - Recovery: Automatic fallback to backups

---

## Deliverables Checklist

### Code

- [ ] Component serialization (all components have `Serialize + Deserialize`)
- [ ] World serialization (`world.save_to_file()`, `World::load_from_file()`)
- [ ] Player profile system (`PlayerProfile` struct + save/load)
- [ ] Save slot management (`SaveManager` with save/load/delete APIs)
- [ ] Screenshot thumbnails (generated on save)
- [ ] Save versioning & migration (`SaveFileHeader`, `MigrationRegistry`)
- [ ] Replay system (input recording, deterministic playback)
- [ ] Backup & recovery (auto-backup, corruption recovery)

### Documentation

- [ ] `SAVE_LOAD_API.md`: API docs for SaveManager, PlayerProfile
- [ ] `SAVE_FORMAT.md`: File format specification (for modding, debugging)
- [ ] `MIGRATION_GUIDE.md`: How to migrate saves across versions
- [ ] `REPLAY_SYSTEM.md`: How to use replay for debugging, speedrunning

### Examples

- [ ] `save_load_demo`: Standalone example demonstrating save/load
- [ ] `hello_companion`: Save/load AI state integration
- [ ] `unified_showcase`: Save/load terrain/materials integration

### Tests

- [ ] Unit tests: Component serialization, world serialization, save slots
- [ ] Integration tests: Full game save/load, replay determinism
- [ ] Stress tests: 100,000 entities, 100× load, corruption recovery
- [ ] Manual tests: Save during combat, full inventory, checkpoint respawn

---

## Timeline Summary

| Week | Days | Focus | Deliverables |
|------|------|-------|--------------|
| 1 | 1-5 | ECS World Serialization | Component derives, archetype/world save/load, entity ID remapping |
| 2 | 6-10 | Player Profile & Save Slots | PlayerProfile struct, save slot management, UI integration |
| 3 | 11-15 | Versioning & Replay | Save versioning, migration system, deterministic replay, backups |

**Total Duration**: 2-3 weeks (10-15 days)

**Estimated Effort**: 80-120 hours (1 FTE)

---

## Next Steps

1. **Read this plan**: Understand scope, timeline, risks
2. **Create Phase 8.4 Audio plan**: Production audio implementation plan
3. **Create Master Integration Plan**: Coordinate all 4 Phase 8 priorities
4. **Begin Phase 8.1**: UI framework (can run in parallel)
5. **Begin Phase 8.3 Week 1**: Component serialization

---

**Document Status**: Implementation plan ready for execution  
**Last Updated**: October 14, 2025  
**Next Document**: PHASE_8_PRIORITY_4_AUDIO_PLAN.md
