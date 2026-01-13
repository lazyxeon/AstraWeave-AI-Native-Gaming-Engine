# Feature Specification: AstraWeave Visual Editor Enhancement

**Date**: January 8, 2026  
**Version**: 1.0  
**Status**: Requirements Gathering Complete  
**Audit Reference**: Based on comprehensive audit of 67 editor source files, competitive analysis of Unity 6.x, Unreal 5.4, Godot 4.3, Bevy 0.15

---

## Executive Summary

The AstraWeave visual editor currently implements a foundational editor with 16+ panels and basic editing workflows, but falls significantly behind industry standards (Unity/Unreal) in critical areas. This specification defines enhancements to bring the editor from **0/100 (non-functional due to compilation issues) to 95/100 (competitive with Unity/Godot)** over 3-6 months.

**Current State**: 
- 16+ existing panels (Hierarchy, Inspector/Entity, Asset Browser, Viewport, etc.)
- Basic undo/redo, scene save/load, prefab system
- Play-in-Editor (F5-F8 hotkeys)
- 3D viewport with gizmos (G/R/S for Translate/Rotate/Scale)
- ❌ **CRITICAL**: Editor does not compile (per audit report)

**Target State**: Production-ready visual editor matching Unity/Godot feature parity

---

## User Stories

### User Story 1 - Level Designer: Efficient Scene Editing

**As a** level designer  
**I want** a fully functional visual editor with viewport navigation, gizmos, and panel workflows  
**So that** I can build game levels without writing code

**Acceptance Scenarios**:

1. **Given** I open the editor, **When** I navigate the 3D viewport, **Then** I can orbit camera with Alt+LMB, pan with MMB, zoom with scroll, and WASD fly mode
2. **Given** I select an entity in hierarchy, **When** I press G/R/S, **Then** I can translate/rotate/scale with visual gizmos and axis constraints (X/Y/Z)
3. **Given** I modify entity transforms, **When** I press Ctrl+Z, **Then** changes are undone with full state restoration
4. **Given** I have unsaved changes, **When** I attempt to close editor, **Then** I see a "Save Changes?" dialog before exit
5. **Given** I drag a prefab from asset browser, **When** I drop it in viewport, **Then** it spawns at mouse hit position with visual preview

---

### User Story 2 - Technical Artist: Advanced Material Authoring

**As a** technical artist  
**I want** a node-based material editor with PBR preview  
**So that** I can create complex shaders visually without writing WGSL code

**Acceptance Scenarios**:

1. **Given** I open material editor, **When** I add texture/color/math nodes, **Then** I can connect nodes with drag-and-drop wires
2. **Given** I modify material graph, **When** I press "Compile", **Then** WGSL shader is generated and applied to preview sphere in real-time
3. **Given** I have a PBR material, **When** I adjust metallic/roughness, **Then** BRDF preview updates with HDR environment lighting
4. **Given** I save material asset, **When** hot-reload triggers, **Then** all scene objects using material update instantly
5. **Given** I create a custom node, **When** I save it as preset, **Then** it appears in node library for reuse

---

### User Story 3 - Gameplay Programmer: Visual Scripting & Debugging

**As a** gameplay programmer  
**I want** visual scripting (node graphs) with live debugging  
**So that** I can prototype game logic and iterate faster than Rust recompilation

**Acceptance Scenarios**:

1. **Given** I open behavior graph editor, **When** I add GOAP/BT/Utility nodes, **Then** I can wire AI logic with visual connections
2. **Given** I have a running simulation (F5), **When** I inspect entity state, **Then** I see live values updating in inspector (velocity, AI state, etc.)
3. **Given** I place breakpoints on graph nodes, **When** simulation hits breakpoint, **Then** editor pauses and shows execution path highlight
4. **Given** I modify graph during pause, **When** I resume (F6), **Then** changes apply immediately (hot-reload)
5. **Given** I have validation errors in graph, **When** I compile, **Then** error messages show at node level with red highlights

---

### User Story 4 - Animator: Timeline & Animation Tools

**As an** animator  
**I want** a timeline sequencer for keyframe animation  
**So that** I can create cutscenes, character animations, and UI transitions

**Acceptance Scenarios**:

1. **Given** I open animation timeline, **When** I add position/rotation/scale tracks, **Then** I can set keyframes at specific timestamps
2. **Given** I scrub timeline, **When** I drag playhead, **Then** viewport shows interpolated preview of animation
3. **Given** I select keyframes, **When** I adjust easing curves, **Then** I see Bezier handles for custom interpolation
4. **Given** I have multiple clips, **When** I blend between them, **Then** I can set blend weights and transition duration
5. **Given** I export animation, **When** I save as asset, **Then** it serializes to .anim.ron format loadable by runtime

---

### User Story 5 - VFX Artist: Particle System Editor

**As a** VFX artist  
**I want** a particle system editor with GPU preview  
**So that** I can create effects (explosions, magic, weather) with visual feedback

**Acceptance Scenarios**:

1. **Given** I open particle editor, **When** I adjust emitter properties (rate, lifetime, size), **Then** viewport shows real-time GPU particle preview
2. **Given** I configure forces (gravity, wind, turbulence), **When** particles spawn, **Then** simulation shows physics-based motion
3. **Given** I add color gradient over lifetime, **When** particles age, **Then** colors interpolate smoothly per gradient curve
4. **Given** I enable collision detection, **When** particles hit terrain, **Then** they bounce/die based on settings
5. **Given** I save particle system, **When** I spawn prefab in scene, **Then** particles play automatically with saved configuration

---

### User Story 6 - System Administrator: Build Pipeline & Deployment

**As a** build engineer  
**I want** automated build configuration and profiling tools  
**So that** I can optimize performance and deploy to multiple platforms

**Acceptance Scenarios**:

1. **Given** I open build manager panel, **When** I configure Windows/Linux/macOS targets, **Then** I can trigger multi-platform builds with one click
2. **Given** I enable profiling mode, **When** I run simulation, **Then** Tracy profiler integrates with flamegraph visualization in editor
3. **Given** I analyze frame budget, **When** I view performance panel, **Then** I see ECS/AI/Physics/Rendering breakdown with budget % (16.67ms @ 60 FPS)
4. **Given** I detect performance regression, **When** system exceeds budget, **Then** warning icon appears in profiler panel with details
5. **Given** I export profiling data, **When** I save report, **Then** it generates markdown with charts for documentation

---

### User Story 7 - Solo Developer: Project Management & Workflow

**As a** solo indie developer  
**I want** streamlined project settings and asset management  
**So that** I can focus on game development without workflow friction

**Acceptance Scenarios**:

1. **Given** I open project settings, **When** I configure physics layers, input mappings, tags, **Then** changes propagate to runtime without restart
2. **Given** I import assets (FBX, GLTF, PNG), **When** I drag files to asset browser, **Then** automatic import pipeline processes them with progress bar
3. **Given** I organize assets, **When** I create folders in asset browser, **Then** I can drag/drop to rearrange with automatic path updates
4. **Given** I search assets, **When** I type in search bar, **Then** fuzzy matching filters by name/type/tag with instant results
5. **Given** I enable version control, **When** I modify scene, **Then** Git status indicators show (modified/untracked) in asset browser and hierarchy

---

## Requirements

### Functional Requirements

#### FR-1: Core Editor Infrastructure (Critical - Unblocks all other work)

**Priority**: P0 (CRITICAL - Editor currently does not compile)

1. **FR-1.1**: Fix compilation errors blocking editor build
   - **Rationale**: Per audit report, editor is non-functional. This blocks all productivity.
   - **Acceptance**: `cargo build -p aw_editor --release` succeeds with 0 errors, 0 warnings
   - **Estimated Effort**: 1-2 days (investigate wgpu 25.0.2/egui 0.32/winit 0.30 API changes)

2. **FR-1.2**: Restore functional parity with pre-broken state
   - **Rationale**: Ensure existing features (16 panels, undo/redo, scene save/load) work
   - **Acceptance**: Manual smoke test checklist passes (30 minutes, documented in verification plan)
   - **Estimated Effort**: 1 day (regression testing, minor fixes)

3. **FR-1.3**: Implement panel docking system (Unity/Godot-style)
   - **Rationale**: Current panel layout is fixed. Competitors allow flexible workspace customization.
   - **Technology**: `egui_dock` crate (already in dependencies)
   - **Acceptance**: User can drag panels to reposition, create tabs, split into sub-regions, save layouts
   - **Estimated Effort**: 3-4 days (integrate egui_dock, migrate 16 existing panels)

---

#### FR-2: Enhanced 3D Viewport (High Priority)

**Priority**: P1-A (Essential for level design workflow)

1. **FR-2.1**: Multi-camera viewport modes
   - **Requirement**: Orthographic (Top/Front/Side) + Perspective views
   - **Acceptance**: Dropdown menu switches between views, ortho uses parallel projection
   - **Estimated Effort**: 2 days

2. **FR-2.2**: Debug draw modes
   - **Requirement**: Wireframe, Unlit, Lighting Only, Overdraw, LOD Visualization, Physics Colliders
   - **Acceptance**: Toolbar buttons toggle each mode, can combine (e.g., Wireframe + Colliders)
   - **Estimated Effort**: 3 days (shader variants, collision visualization)

3. **FR-2.3**: Grid & Snap Settings
   - **Requirement**: Configurable grid size (0.1, 0.25, 0.5, 1.0, 2.0, 5.0 units), snap-to-grid toggle
   - **Acceptance**: Gizmo operations snap to grid when enabled, visual grid updates to match size
   - **Estimated Effort**: 1 day (already partially implemented, needs UI)

4. **FR-2.4**: Camera bookmarks (F1-F12)
   - **Requirement**: Save/restore camera position/orientation with function keys
   - **Acceptance**: Press F1-F12 to save view, Shift+F1-F12 to restore, visual indicator shows bookmarked slots
   - **Estimated Effort**: 1 day (already in viewport/widget.rs:49-56, needs UI integration)

5. **FR-2.5**: Scene Gizmos (Lights/Cameras/Audio/Nav Volumes)
   - **Requirement**: Visual representation of non-mesh entities (point light = sphere, directional = arrow, camera = frustum)
   - **Acceptance**: Gizmos render in viewport, selectable, scale with camera distance, color-coded by type
   - **Estimated Effort**: 4-5 days (custom gizmo renderer per entity type)

---

#### FR-3: Visual Material Editor (High Priority)

**Priority**: P1-B (Critical for technical artists, but not blocking level design)

1. **FR-3.1**: Node-based material graph UI
   - **Requirement**: Drag-and-drop nodes (Texture Sample, Color, Math Ops, PBR Master), wire connections
   - **Technology**: `egui_graphs` crate (already in dependencies)
   - **Acceptance**: Create material with 3+ nodes, connect outputs to inputs, UI validates data types (float/vec3/texture)
   - **Estimated Effort**: 5-6 days (graph UI, node library, data model)

2. **FR-3.2**: Real-time WGSL shader code generation
   - **Requirement**: Compile graph to WGSL fragment shader, apply to preview mesh
   - **Acceptance**: Modifications update preview within 500ms (hot-reload), compilation errors show in console with node highlights
   - **Estimated Effort**: 6-7 days (graph → WGSL transpiler, error handling)

3. **FR-3.3**: PBR material preview sphere
   - **Requirement**: Lit sphere with HDR environment, rotate to inspect reflections
   - **Acceptance**: Mouse drag rotates environment, material updates show metallic/roughness/normal map effects
   - **Estimated Effort**: 2-3 days (existing BRDF preview integration)

4. **FR-3.4**: Material asset management
   - **Requirement**: Save .mat.ron files, load in asset browser, apply to scene meshes
   - **Acceptance**: Drag material from asset browser to entity → applies to renderer component
   - **Estimated Effort**: 2 days (serialization, asset browser integration)

---

#### FR-4: Visual Scripting & Behavior Graphs (Medium Priority)

**Priority**: P1-C (Productivity booster, not critical path)

1. **FR-4.1**: Enhanced behavior graph editor
   - **Requirement**: Extend existing `behavior_graph_ui` with better UX (search nodes, minimap, zoom/pan)
   - **Acceptance**: Can create GOAP/BT/Utility graphs with 20+ nodes, navigate large graphs comfortably
   - **Estimated Effort**: 3-4 days (already exists, needs refinement)

2. **FR-4.2**: Live debugging & breakpoints
   - **Requirement**: During Play-in-Editor (F5), highlight active nodes, pause on breakpoint
   - **Acceptance**: Right-click node → Add Breakpoint, simulation pauses when hit, inspector shows execution stack
   - **Estimated Effort**: 5-6 days (runtime → editor communication, pause/resume logic)

3. **FR-4.3**: Graph validation & error reporting
   - **Requirement**: Detect cycles, missing connections, type mismatches
   - **Acceptance**: Compile button shows errors in panel, nodes with errors highlighted red, tooltip shows fix hints
   - **Estimated Effort**: 2-3 days (validation rules, UI error display)

---

#### FR-5: Animation Timeline (Medium Priority)

**Priority**: P1-D (Nice-to-have for cutscenes, not MVP)

1. **FR-5.1**: Keyframe timeline UI
   - **Requirement**: Horizontal timeline with tracks (Transform, Color, Material Props), keyframe diamonds
   - **Acceptance**: Click timeline to place keyframe, drag to move, right-click to delete
   - **Estimated Effort**: 6-7 days (timeline widget, track system)

2. **FR-5.2**: Curve editor for easing
   - **Requirement**: Bezier curve editor for interpolation between keyframes
   - **Acceptance**: Double-click keyframe → opens curve editor, drag handles to adjust ease-in/ease-out
   - **Estimated Effort**: 4-5 days (Bezier math, curve UI)

3. **FR-5.3**: Animation clip blending
   - **Requirement**: Blend between multiple clips with weight sliders
   - **Acceptance**: Two clips at 50%/50% → output is averaged, preview updates in real-time
   - **Estimated Effort**: 3-4 days (blend logic, UI controls)

---

#### FR-6: Particle System Editor (Low Priority)

**Priority**: P2 (Advanced feature, defer to post-MVP)

1. **FR-6.1**: GPU particle preview
   - **Requirement**: Real-time particle simulation in viewport using `astraweave-render/gpu_particles.rs`
   - **Acceptance**: Modify emitter properties → preview updates within 1 frame (16.67ms)
   - **Estimated Effort**: 5-6 days (GPU compute integration, viewport rendering)

2. **FR-6.2**: Particle property inspector
   - **Requirement**: UI sliders for rate, lifetime, size, velocity, forces, color gradient
   - **Acceptance**: All properties exposed, tooltips show value ranges, reset button restores defaults
   - **Estimated Effort**: 3 days (UI forms, property bindings)

---

#### FR-7: Build Pipeline & Profiling (Medium Priority)

**Priority**: P1-D (Production hygiene, not blocking creative work)

1. **FR-7.1**: Enhanced build manager panel
   - **Requirement**: Configure target platforms (Windows/Linux/macOS/WASM), optimization levels, feature flags
   - **Acceptance**: Build button triggers `cargo build` with selected config, progress bar shows status, output logs to console
   - **Estimated Effort**: 4-5 days (existing BuildManagerPanel refinement)

2. **FR-7.2**: Tracy profiler integration
   - **Requirement**: Embed Tracy client, visualize flamegraphs in editor panel
   - **Acceptance**: Enable profiling → captures next frame → shows ECS/AI/Physics breakdown in panel
   - **Estimated Effort**: 6-7 days (Tracy Rust bindings, visualization UI)

3. **FR-7.3**: Performance budget warnings
   - **Requirement**: Red/yellow/green indicators for 60 FPS budget (16.67ms total)
   - **Acceptance**: System over budget → red icon in profiler panel, tooltip shows exceeded category
   - **Estimated Effort**: 2 days (threshold checks, UI indicators)

---

#### FR-8: Project Settings & Asset Pipeline (High Priority)

**Priority**: P1-A (Foundational for all workflows)

1. **FR-8.1**: Project settings panel
   - **Requirement**: UI for physics layers (32 max), input mappings (keyboard/gamepad), tags/groups
   - **Acceptance**: Save settings to `project_settings.toml`, hot-reload applies changes without restart
   - **Estimated Effort**: 4-5 days (settings data model, UI forms, persistence)

2. **FR-8.2**: Asset import pipeline UI
   - **Requirement**: Drag FBX/GLTF/PNG to asset browser → automatic import with progress bar
   - **Acceptance**: FBX imports as mesh asset, GLTF as scene prefab, PNG as texture with mipmap generation
   - **Estimated Effort**: 7-8 days (file watcher, import jobs, progress UI)

3. **FR-8.3**: Asset organization (folders/tags/search)
   - **Requirement**: Create folders in asset browser, drag assets to organize, tag with labels, fuzzy search
   - **Acceptance**: Search "hero" → finds "hero_mesh.gltf", "hero_texture.png", etc. with instant results
   - **Estimated Effort**: 3-4 days (tree view, tagging system, search index)

---

#### FR-9: Version Control Integration (Low Priority)

**Priority**: P2 (Nice-to-have, defer to post-MVP)

1. **FR-9.1**: Git status indicators
   - **Requirement**: Show modified/untracked/conflicted icons in asset browser and hierarchy
   - **Acceptance**: Modified scene file → yellow "M" icon, untracked asset → green "?" icon
   - **Estimated Effort**: 3-4 days (git2-rs integration, status polling)

2. **FR-9.2**: Diff viewer for scenes
   - **Requirement**: Visual diff for .scene.ron files (added/removed/modified entities)
   - **Acceptance**: Right-click scene → "Show Diff" → side-by-side comparison with highlights
   - **Estimated Effort**: 6-7 days (RON parser, diff algorithm, UI renderer)

---

### Non-Functional Requirements

#### NFR-1: Performance

1. **NFR-1.1**: Editor must maintain 60 FPS during editing (idle + camera movement)
   - **Acceptance**: Profiler shows <16.67ms frame time with 1000 entities in scene
   - **Rationale**: Per audit, rendering pipeline achieves 2.70ms @ 1k entities (84% headroom). Editor UI must not consume remaining budget.

2. **NFR-1.2**: Panel updates must not block main thread
   - **Acceptance**: Scrolling console with 10k lines maintains 60 FPS
   - **Rationale**: Use `egui::ScrollArea` with virtual scrolling, only render visible lines

3. **NFR-1.3**: Asset import must run async with progress feedback
   - **Acceptance**: Importing 100 textures shows progress bar, editor remains responsive
   - **Rationale**: Tokio async tasks for file I/O, send progress to main thread via channels

---

#### NFR-2: Usability

1. **NFR-2.1**: All features must have keyboard shortcuts documented in Help dialog (F1)
   - **Acceptance**: Help dialog lists shortcuts in table format, searchable
   - **Rationale**: Competitors (Unity/Unreal) provide comprehensive shortcut references

2. **NFR-2.2**: Error messages must be actionable
   - **Acceptance**: "Material compilation failed" → shows WGSL error line number + snippet, "Fix" button opens material editor
   - **Rationale**: Per audit, current errors lack context. Improve UX with guidance.

3. **NFR-2.3**: Undo/redo must work across all editor actions (not just transform edits)
   - **Acceptance**: Material edits, graph changes, asset moves all undoable with Ctrl+Z
   - **Rationale**: Existing `command::UndoStack` only handles scene edits. Extend to all actions.

---

#### NFR-3: Reliability

1. **NFR-3.1**: Autosave must trigger every 5 minutes with recovery on crash
   - **Acceptance**: Kill editor process → relaunch → "Recover autosave?" dialog appears
   - **Rationale**: Existing autosave saves to `.autosave/` directory. Add crash recovery logic.

2. **NFR-3.2**: Invalid assets must not crash editor
   - **Acceptance**: Drag corrupted FBX to asset browser → shows error toast, editor continues running
   - **Rationale**: Use `anyhow::Result` for all I/O, catch panics with `std::panic::catch_unwind`

3. **NFR-3.3**: Editor state must persist across sessions
   - **Acceptance**: Close editor → reopen → last scene, camera position, panel layout restored
   - **Rationale**: Extend `editor_preferences.rs` to include layout serialization

---

#### NFR-4: Compatibility

1. **NFR-4.1**: Support wgpu 25.0.2, egui 0.32, winit 0.30 (current dependencies)
   - **Acceptance**: No dependency downgrades required to fix compilation
   - **Rationale**: Maintain compatibility with latest rendering pipeline

2. **NFR-4.2**: Asset formats must match runtime expectations
   - **Acceptance**: Material saved in editor → loads in game runtime without conversion
   - **Rationale**: Use same serialization (`serde_json`, `ron`) as runtime crates

---

## Success Criteria

### Quantitative Metrics

1. **Build Success**: Editor compiles with `0 errors, 0 warnings` (currently fails)
2. **Performance**: Maintains 60 FPS with 1000 entities in viewport (currently 370 FPS @ 1k per audit, editor UI must not degrade this)
3. **Feature Parity**: Achieves 95/100 vs Unity Editor (currently 0/100 per audit)
4. **Test Coverage**: 80%+ coverage for editor crates (currently unmeasured)
5. **Regression Prevention**: Zero critical bugs in release candidate (manual QA checklist: 100 test cases)

### Qualitative Metrics

1. **User Satisfaction**: Internal playtest with 5 developers → average rating 8/10 for "productivity improvement"
2. **Workflow Efficiency**: Level design task (create simple scene with 10 prefabs, 5 materials) completes in <30 minutes
3. **Documentation Quality**: All features documented with video tutorials (5-10 minutes each)
4. **Onboarding**: New developer can complete "First Level" tutorial in <2 hours without prior AstraWeave experience

---

## Assumptions

1. **Existing Infrastructure**: Assumes `astraweave-render`, `astraweave-ecs`, `astraweave-asset` crates are stable and provide necessary APIs
2. **Team Size**: Assumes 1-2 FTE working on editor (per audit estimates)
3. **Iteration Velocity**: Assumes 2-week sprints with weekly demos to validate progress
4. **No Scope Creep**: P2 features (particles, VCS, animation timeline) deferred to post-MVP unless capacity allows

---

## Out of Scope

1. **Multiplayer Editing**: Collaborative editing (like Unreal's Multi-User Editing) is not required for MVP
2. **Mobile Editor**: Desktop-only (Windows/Linux/macOS). No tablet/mobile editor UI.
3. **Plugin Marketplace**: Asset store integration deferred (per audit, 0 plugins currently)
4. **Console Deployment**: No Xbox/PlayStation export in MVP (desktop platforms only)
5. **Cloud Services**: No cloud saves, analytics, or online services

---

## Risks & Mitigation

### Risk 1: Compilation Issues Persist

**Risk**: FR-1.1 (fix compilation) takes longer than 1-2 days due to deep API changes
**Likelihood**: Medium (30%)  
**Impact**: High (blocks all work)  
**Mitigation**:
- Allocate 1 week buffer for investigation
- Fallback: Pin dependencies to last working versions (wgpu 24.x, egui 0.31, winit 0.29)
- Escalation: Engage wgpu/egui community for migration guidance

### Risk 2: Performance Regression

**Risk**: Panel updates cause FPS drops below 60 FPS
**Likelihood**: Low (20%)  
**Impact**: Medium (degrades UX but not blocking)  
**Mitigation**:
- Profile early and often (every sprint)
- Use `egui::Context::request_repaint()` sparingly (only on data changes, not per frame)
- Implement virtual scrolling for large lists (hierarchy with 10k+ entities)

### Risk 3: Scope Creep

**Risk**: Stakeholders request P2 features during P1 implementation
**Likelihood**: High (60%)  
**Impact**: Medium (delays MVP timeline)  
**Mitigation**:
- Strict PRD adherence (P0 → P1-A → P1-B → P1-C → P1-D → P2)
- Weekly stakeholder reviews with burn-down charts
- Maintain "Post-MVP Backlog" document for feature requests

### Risk 4: UX Mismatch with Competitors

**Risk**: Users expect Unity/Unreal workflows, find AstraWeave editor unintuitive
**Likelihood**: Medium (40%)  
**Impact**: High (adoption blocker)  
**Mitigation**:
- User testing with Unity/Unreal veterans (3-5 developers)
- Iterate on feedback weekly
- Provide "Migration Guides" documentation (Unity → AstraWeave shortcuts/workflows)

---

## Dependencies

### Internal Dependencies

1. **astraweave-render**: Must support hot-reload of materials/shaders (FR-3.2)
2. **astraweave-ecs**: Must expose World serialization for scene save/load (FR-1.2, already exists)
3. **astraweave-asset**: Must trigger callbacks on file changes for hot-reload (FR-3.4)
4. **astraweave-physics**: Must provide debug draw API for collider visualization (FR-2.2)

### External Dependencies

1. **egui_dock**: Panel docking system (FR-1.3) - already in Cargo.toml
2. **egui_graphs**: Material graph UI (FR-3.1) - already in Cargo.toml
3. **rfd**: Native file dialogs for Open/Save (FR-8.2) - needs addition
4. **git2**: Git integration (FR-9.1) - needs addition (optional, P2)
5. **tracy-client**: Profiler integration (FR-7.2) - needs addition

---

## Prioritization

### Phase 1: Critical Path (Weeks 1-4)

- **FR-1.1, FR-1.2**: Fix compilation, restore functionality (1 week)
- **FR-1.3**: Panel docking system (1 week)
- **FR-2.1, FR-2.2, FR-2.3**: Viewport enhancements (1.5 weeks)
- **FR-8.1**: Project settings panel (0.5 weeks)

**Deliverable**: Functional editor with flexible layout, usable viewport, project configuration

---

### Phase 2: High-Value Features (Weeks 5-8)

- **FR-3.1, FR-3.2, FR-3.3, FR-3.4**: Material editor (3 weeks)
- **FR-8.2, FR-8.3**: Asset import pipeline (2 weeks)

**Deliverable**: Technical artists can create materials, asset management workflows functional

---

### Phase 3: Productivity Multipliers (Weeks 9-12)

- **FR-4.1, FR-4.2, FR-4.3**: Visual scripting enhancements (2 weeks)
- **FR-7.1, FR-7.2, FR-7.3**: Build pipeline & profiling (2 weeks)
- **FR-2.5**: Scene gizmos (1 week)

**Deliverable**: Programmers can debug visually, builds automated, performance monitoring integrated

---

### Phase 4: Polish & Deferred Features (Weeks 13-16+)

- **FR-5**: Animation timeline (3 weeks) - *Optional if capacity allows*
- **FR-6**: Particle system editor (2 weeks) - *Optional if capacity allows*
- **FR-9**: Version control integration (2 weeks) - *Optional if capacity allows*

**Deliverable**: AAA feature parity (animation, VFX, VCS)

---

## Acceptance Criteria Summary

**Editor is considered "MVP Complete" when**:

1. ✅ Compiles with 0 errors, 0 warnings
2. ✅ All P0 and P1-A features implemented and tested
3. ✅ Maintains 60 FPS with 1000 entities in viewport
4. ✅ 80%+ test coverage for editor crates
5. ✅ Manual QA checklist (100 test cases) passes with 0 critical bugs
6. ✅ User testing (5 developers) averages 8/10 satisfaction rating
7. ✅ Documentation complete for all features (video tutorials + written guides)
8. ✅ "First Level" tutorial completes in <2 hours for new users

---

## Appendix A: Competitive Feature Matrix

| Feature | Unity 6.x | Unreal 5.4 | Godot 4.3 | Bevy 0.15 | AstraWeave (Current) | AstraWeave (Target) |
|---------|-----------|------------|-----------|-----------|----------------------|---------------------|
| **Panel Docking** | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ (FR-1.3) |
| **Material Editor** | ✅ Shader Graph | ✅ Material Editor | ✅ Shader Editor | ❌ | ⚠️ Basic | ✅ Node-based (FR-3) |
| **Visual Scripting** | ✅ Visual Scripting | ✅ Blueprint | ✅ VisualScript | ❌ | ⚠️ Graph Editor | ✅ Enhanced (FR-4) |
| **Animation Timeline** | ✅ Timeline | ✅ Sequencer | ✅ AnimationPlayer | ❌ | ❌ | ✅ (FR-5) |
| **Particle Editor** | ✅ VFX Graph | ✅ Niagara | ✅ Particles 2D/3D | ❌ | ❌ | ✅ (FR-6) |
| **Profiler** | ✅ Profiler | ✅ Insights | ✅ Debugger | ⚠️ bevy-inspector-egui | ❌ | ✅ Tracy (FR-7) |
| **Asset Import** | ✅ Auto | ✅ Auto | ✅ Auto | ❌ | ⚠️ Manual | ✅ Drag-drop (FR-8) |
| **Project Settings** | ✅ | ✅ | ✅ | ❌ | ❌ | ✅ (FR-8) |
| **VCS Integration** | ✅ Collab, Git | ✅ Perforce, Git | ✅ Git | ❌ | ❌ | ✅ (FR-9, P2) |
| **Play-in-Editor** | ✅ | ✅ | ✅ | ❌ | ✅ F5-F8 | ✅ (existing) |
| **3D Gizmos** | ✅ | ✅ | ✅ | ⚠️ Basic | ✅ G/R/S | ✅ Enhanced (FR-2) |

**Legend**: ✅ Full support | ⚠️ Partial support | ❌ Not available

---

## Appendix B: Clarifications Needed

[NEEDS CLARIFICATION: Multi-user Editing Priority]
**Question**: Is collaborative editing (multiple users editing same scene simultaneously) required for MVP?  
**Context**: Unreal has Multi-User Editing, Unity has Collab. This is complex (conflict resolution, real-time sync) and may delay MVP by 2-3 months.  
**Suggested Default**: Defer to post-MVP (out of scope)

[NEEDS CLARIFICATION: Target Platform Priority]
**Question**: Should editor support WASM (browser-based) or desktop-only?  
**Context**: WASM limits file system access, requires different UI patterns. Desktop-only is faster to implement.  
**Suggested Default**: Desktop-only (Windows/Linux/macOS) for MVP, WASM as future enhancement

[NEEDS CLARIFICATION: Scripting Language]
**Question**: Should visual scripting compile to Rhai scripts, or remain graph-only?  
**Context**: Rhai integration exists but per audit is not production-ready. Graph-only is simpler but less flexible.  
**Suggested Default**: Graph-only for MVP (Rhai as text scripting is separate P2 feature)

[NEEDS CLARIFICATION: Asset Store Integration]
**Question**: Is plugin marketplace / asset store required for MVP?  
**Context**: Per audit, ecosystem is 40/100. Building asset store is 6-12 month effort (community building, payment processing, moderation).  
**Suggested Default**: Defer to post-MVP (out of scope)

[NEEDS CLARIFICATION: Console Support]
**Question**: Should build manager support Xbox/PlayStation/Switch export?  
**Context**: Console SDKs require NDAs, platform-specific tooling. This is 6-12 month effort.  
**Suggested Default**: Desktop platforms only for MVP (consoles as future paid licenses)

---

**End of Requirements Document**
