//! Zone transition system — dispatches trigger actions to game systems.
//!
//! Parses the `"category.verb:target"` action format used in zone descriptors
//! and routes events to the appropriate subsystems (cell streaming, cinematics,
//! dialogue, VFX, boss encounters, etc.).

use astraweave_scene::world_partition::GridCoord;
use std::collections::HashMap;
use tracing::info;

// ── Action parsing ─────────────────────────────────────────────────────────

/// Parsed representation of a trigger action string.
///
/// Actions follow the format `"category.verb:target"`, e.g.:
/// - `"zone.transition:Z4_boss_courtyard"`
/// - `"cinematic.play:boss_intro"`
/// - `"dialogue.play:crossroads_arrival"`
/// - `"vfx.activate:storm_stabilize_blue"`
///
/// Legacy actions (PascalCase single tokens) are stored as
/// `category = "legacy"`, `verb = ""`, `target = "<action>"`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerAction {
    pub category: String,
    pub verb: String,
    pub target: String,
}

impl TriggerAction {
    /// Parses a raw action string into structured form.
    ///
    /// Supports both:
    /// - Namespaced: `"category.verb:target"`
    /// - Legacy: `"PascalCaseSingleToken"`
    #[must_use]
    pub fn parse(raw: &str) -> Self {
        if let Some((prefix, target)) = raw.split_once(':') {
            if let Some((cat, verb)) = prefix.split_once('.') {
                Self {
                    category: cat.to_string(),
                    verb: verb.to_string(),
                    target: target.to_string(),
                }
            } else {
                // "prefix:target" with no dot — treat prefix as category.
                Self {
                    category: prefix.to_string(),
                    verb: String::new(),
                    target: target.to_string(),
                }
            }
        } else {
            // Legacy single-token action.
            Self {
                category: "legacy".to_string(),
                verb: String::new(),
                target: raw.to_string(),
            }
        }
    }

    /// Returns `true` if this is a zone transition action.
    #[must_use]
    pub fn is_zone_transition(&self) -> bool {
        self.category == "zone" && self.verb == "transition"
    }

    /// Returns `true` if this is a cinematic play action.
    #[must_use]
    pub fn is_cinematic(&self) -> bool {
        self.category == "cinematic" && self.verb == "play"
    }

    /// Returns `true` if this is a dialogue play action.
    #[must_use]
    pub fn is_dialogue(&self) -> bool {
        self.category == "dialogue" && self.verb == "play"
    }

    /// Returns `true` if this is a decision open action.
    #[must_use]
    pub fn is_decision(&self) -> bool {
        self.category == "decision" && self.verb == "open"
    }

    /// Returns `true` if this is a VFX activate action.
    #[must_use]
    pub fn is_vfx(&self) -> bool {
        self.category == "vfx" && self.verb == "activate"
    }

    /// Returns `true` if this is a boss-related action.
    #[must_use]
    pub fn is_boss(&self) -> bool {
        self.category == "boss"
    }

    /// Returns `true` if this is a legacy (unscoped) action.
    #[must_use]
    pub fn is_legacy(&self) -> bool {
        self.category == "legacy"
    }
}

// ── Zone registry ──────────────────────────────────────────────────────────

/// Maps zone names (e.g. `"Z0_loomspire_sanctum"`) to their grid coordinates.
///
/// Populated at startup from the cell manifest. Used by the zone transition
/// system to resolve `"zone.transition:Z4_boss_courtyard"` → `GridCoord(104,0,0)`.
#[derive(Debug, Clone, Default)]
pub struct ZoneRegistry {
    name_to_coord: HashMap<String, GridCoord>,
    coord_to_name: HashMap<GridCoord, String>,
}

impl ZoneRegistry {
    /// Creates a new empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a zone name ↔ coordinate mapping.
    pub fn register(&mut self, name: impl Into<String>, coord: GridCoord) {
        let name = name.into();
        self.name_to_coord.insert(name.clone(), coord);
        self.coord_to_name.insert(coord, name);
    }

    /// Looks up the grid coordinate for a zone name.
    #[must_use]
    pub fn coord_for(&self, name: &str) -> Option<GridCoord> {
        self.name_to_coord.get(name).copied()
    }

    /// Looks up the zone name for a grid coordinate.
    #[must_use]
    pub fn name_for(&self, coord: GridCoord) -> Option<&str> {
        self.coord_to_name.get(&coord).map(String::as_str)
    }

    /// Returns the total number of registered zones.
    #[must_use]
    pub fn len(&self) -> usize {
        self.name_to_coord.len()
    }

    /// Returns `true` if no zones are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.name_to_coord.is_empty()
    }

    /// Builds the default Veilweaver vertical-slice zone registry.
    ///
    /// Zone layout:
    /// - Z0 `(100,0,0)` — Loomspire Sanctum (tutorial)
    /// - Z1 `(101,0,0)` — Echo Grove (combat intro)
    /// - Z2 `(102,0,0)` — Fractured Cliffs (traversal)
    /// - Z2a `(102,1,0)` — Side Alcove (optional secret)
    /// - Z3 `(103,0,0)` — Loom Crossroads (storm choice)
    /// - Z4 `(104,0,0)` — Boss Courtyard (Oathbound Warden)
    #[must_use]
    pub fn veilweaver_default() -> Self {
        let mut reg = Self::new();
        reg.register("Z0_loomspire_sanctum", GridCoord::new(100, 0, 0));
        reg.register("Z1_echo_grove", GridCoord::new(101, 0, 0));
        reg.register("Z2_fractured_cliffs", GridCoord::new(102, 0, 0));
        reg.register("Z2a_side_alcove", GridCoord::new(102, 1, 0));
        reg.register("Z3_loom_crossroads", GridCoord::new(103, 0, 0));
        reg.register("Z4_boss_courtyard", GridCoord::new(104, 0, 0));
        info!("ZoneRegistry loaded {} zones", reg.len());
        reg
    }
}

// ── Transition events ──────────────────────────────────────────────────────

/// Events produced by the zone transition dispatcher.
///
/// These are consumed by the game loop to orchestrate loading, cinematics, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum ZoneTransitionEvent {
    /// Player triggered a zone transition.
    ZoneTransition {
        from_trigger: String,
        target_zone: String,
        target_coord: GridCoord,
    },
    /// A cinematic should begin playing.
    CinematicTrigger {
        trigger_id: String,
        cinematic_name: String,
    },
    /// A dialogue should begin playing.
    DialogueTrigger {
        trigger_id: String,
        dialogue_name: String,
    },
    /// A decision prompt should be presented.
    DecisionTrigger {
        trigger_id: String,
        decision_name: String,
    },
    /// A VFX should activate.
    VfxTrigger {
        trigger_id: String,
        vfx_name: String,
    },
    /// A boss event was triggered.
    BossTrigger {
        trigger_id: String,
        action: TriggerAction,
    },
    /// Legacy action (unscoped PascalCase).
    LegacyAction {
        trigger_id: String,
        action_name: String,
    },
}

/// Dispatches trigger actions into structured events.
///
/// Takes a list of `(trigger_id, action_string)` pairs and produces
/// [`ZoneTransitionEvent`] values. Requires a [`ZoneRegistry`] to resolve
/// zone names to grid coordinates.
pub fn dispatch_trigger_actions(
    fired_triggers: &[(String, String)],
    registry: &ZoneRegistry,
) -> Vec<ZoneTransitionEvent> {
    let mut events = Vec::new();

    for (trigger_id, raw_action) in fired_triggers {
        let action = TriggerAction::parse(raw_action);

        let event = if action.is_zone_transition() {
            if let Some(coord) = registry.coord_for(&action.target) {
                info!(
                    "Zone transition: trigger '{}' → {} ({:?})",
                    trigger_id, action.target, coord
                );
                ZoneTransitionEvent::ZoneTransition {
                    from_trigger: trigger_id.clone(),
                    target_zone: action.target.clone(),
                    target_coord: coord,
                }
            } else {
                tracing::warn!(
                    "Zone transition target '{}' not found in registry",
                    action.target
                );
                continue;
            }
        } else if action.is_cinematic() {
            ZoneTransitionEvent::CinematicTrigger {
                trigger_id: trigger_id.clone(),
                cinematic_name: action.target.clone(),
            }
        } else if action.is_dialogue() {
            ZoneTransitionEvent::DialogueTrigger {
                trigger_id: trigger_id.clone(),
                dialogue_name: action.target.clone(),
            }
        } else if action.is_decision() {
            ZoneTransitionEvent::DecisionTrigger {
                trigger_id: trigger_id.clone(),
                decision_name: action.target.clone(),
            }
        } else if action.is_vfx() {
            ZoneTransitionEvent::VfxTrigger {
                trigger_id: trigger_id.clone(),
                vfx_name: action.target.clone(),
            }
        } else if action.is_boss() {
            ZoneTransitionEvent::BossTrigger {
                trigger_id: trigger_id.clone(),
                action,
            }
        } else {
            ZoneTransitionEvent::LegacyAction {
                trigger_id: trigger_id.clone(),
                action_name: action.target.clone(),
            }
        };

        events.push(event);
    }

    events
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_namespaced_action() {
        let a = TriggerAction::parse("zone.transition:Z4_boss_courtyard");
        assert_eq!(a.category, "zone");
        assert_eq!(a.verb, "transition");
        assert_eq!(a.target, "Z4_boss_courtyard");
        assert!(a.is_zone_transition());
    }

    #[test]
    fn parse_cinematic_action() {
        let a = TriggerAction::parse("cinematic.play:boss_intro");
        assert!(a.is_cinematic());
        assert_eq!(a.target, "boss_intro");
    }

    #[test]
    fn parse_dialogue_action() {
        let a = TriggerAction::parse("dialogue.play:crossroads_arrival");
        assert!(a.is_dialogue());
        assert_eq!(a.target, "crossroads_arrival");
    }

    #[test]
    fn parse_decision_action() {
        let a = TriggerAction::parse("decision.open:storm_routing");
        assert!(a.is_decision());
        assert_eq!(a.target, "storm_routing");
    }

    #[test]
    fn parse_vfx_action() {
        let a = TriggerAction::parse("vfx.activate:storm_stabilize_blue");
        assert!(a.is_vfx());
        assert_eq!(a.target, "storm_stabilize_blue");
    }

    #[test]
    fn parse_boss_action() {
        let a = TriggerAction::parse("boss.defeat:oathbound_warden");
        assert!(a.is_boss());
        assert_eq!(a.target, "oathbound_warden");
    }

    #[test]
    fn parse_legacy_action() {
        let a = TriggerAction::parse("StartWeavingTutorial");
        assert!(a.is_legacy());
        assert_eq!(a.target, "StartWeavingTutorial");
    }

    #[test]
    fn zone_registry_default() {
        let reg = ZoneRegistry::veilweaver_default();
        assert_eq!(reg.len(), 6);
        assert_eq!(
            reg.coord_for("Z0_loomspire_sanctum"),
            Some(GridCoord::new(100, 0, 0))
        );
        assert_eq!(
            reg.coord_for("Z4_boss_courtyard"),
            Some(GridCoord::new(104, 0, 0))
        );
        assert_eq!(
            reg.name_for(GridCoord::new(103, 0, 0)),
            Some("Z3_loom_crossroads")
        );
    }

    #[test]
    fn dispatch_zone_transition() {
        let reg = ZoneRegistry::veilweaver_default();
        let triggers = vec![(
            "zone_exit_boss".to_string(),
            "zone.transition:Z4_boss_courtyard".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            ZoneTransitionEvent::ZoneTransition {
                target_zone,
                target_coord,
                ..
            } if target_zone == "Z4_boss_courtyard"
                && *target_coord == GridCoord::new(104, 0, 0)
        ));
    }

    #[test]
    fn dispatch_cinematic_trigger() {
        let reg = ZoneRegistry::new();
        let triggers = vec![(
            "boss_intro".to_string(),
            "cinematic.play:boss_intro".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 1);
        assert!(matches!(
            &events[0],
            ZoneTransitionEvent::CinematicTrigger {
                cinematic_name, ..
            } if cinematic_name == "boss_intro"
        ));
    }

    #[test]
    fn dispatch_unknown_zone_skipped() {
        let reg = ZoneRegistry::new(); // Empty registry
        let triggers = vec![(
            "test".to_string(),
            "zone.transition:nonexistent".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert!(events.is_empty());
    }

    #[test]
    fn dispatch_multiple_actions() {
        let reg = ZoneRegistry::veilweaver_default();
        let triggers = vec![
            (
                "entry".to_string(),
                "dialogue.play:crossroads_arrival".to_string(),
            ),
            (
                "exit".to_string(),
                "zone.transition:Z4_boss_courtyard".to_string(),
            ),
            ("fx".to_string(), "vfx.activate:storm_blue".to_string()),
        ];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 3);
    }
}
