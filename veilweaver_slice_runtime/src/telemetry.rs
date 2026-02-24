//! Telemetry collector — wires game loop events into the recap panel.
//!
//! Listens to [`GameLoopEvent`]s each tick and accumulates gameplay metrics
//! in a [`RecapPanel`]. When the run completes, call [`finalize`] to compute
//! category ratings.
//!
//! Also aggregates real-time counters that feed into the HUD state
//! (echo balance, anchor count, damage stats).

use crate::boss_hud::BossPhase;
use crate::companion_hud::AffinityRank;
use crate::game_loop::GameLoopEvent;
use crate::recap_panel::{MetricCategory, Rating, RecapPanel};
use crate::storm_choice::StormChoice;

// ── Telemetry Collector ────────────────────────────────────────────────

/// Accumulates gameplay telemetry and populates the recap panel.
#[derive(Debug, Clone)]
pub struct TelemetryCollector {
    // ── Combat counters ────────────────────────────────────────────
    pub damage_dealt: f32,
    pub damage_taken: f32,
    pub enemies_defeated: u32,
    pub deaths: u32,
    pub combos_landed: u32,

    // ── Exploration ────────────────────────────────────────────────
    pub zones_visited: Vec<String>,
    pub secrets_found: u32,

    // ── Weaving ────────────────────────────────────────────────────
    pub anchors_repaired: u32,
    pub echoes_collected: u32,
    pub echoes_spent: u32,

    // ── Companion ──────────────────────────────────────────────────
    pub companion_support_actions: u32,
    pub final_affinity_rank: Option<AffinityRank>,

    // ── Narrative ──────────────────────────────────────────────────
    pub storm_choice: Option<StormChoice>,
    pub dialogues_completed: u32,

    // ── Boss ───────────────────────────────────────────────────────
    pub boss_phases_seen: Vec<BossPhase>,
    pub boss_telegraphs_dodged: u32,
    pub boss_defeated: bool,

    // ── Timing ─────────────────────────────────────────────────────
    /// Accumulated total playtime in seconds.
    ///
    /// NOTE: f32 precision is adequate for the ~30-minute vertical slice
    /// (epsilon ~0.0001s at 1800.0). For sessions >2 hours, consider f64.
    pub total_time: f32,
    pub boss_fight_time: f32,
    boss_fight_active: bool,
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryCollector {
    /// Creates a zeroed telemetry collector ready to accumulate stats.
    #[must_use]
    pub fn new() -> Self {
        Self {
            damage_dealt: 0.0,
            damage_taken: 0.0,
            enemies_defeated: 0,
            deaths: 0,
            combos_landed: 0,
            zones_visited: Vec::new(),
            secrets_found: 0,
            anchors_repaired: 0,
            echoes_collected: 0,
            echoes_spent: 0,
            companion_support_actions: 0,
            final_affinity_rank: None,
            storm_choice: None,
            dialogues_completed: 0,
            boss_phases_seen: Vec::new(),
            boss_telegraphs_dodged: 0,
            boss_defeated: false,
            total_time: 0.0,
            boss_fight_time: 0.0,
            boss_fight_active: false,
        }
    }

    // ── Event processing ───────────────────────────────────────────

    /// Process a batch of game loop events from one tick.
    pub fn process_events(&mut self, events: &[GameLoopEvent], dt: f32) {
        self.total_time += dt;
        if self.boss_fight_active {
            self.boss_fight_time += dt;
        }

        for event in events {
            match event {
                GameLoopEvent::ZoneLoading { zone_name, .. } => {
                    if !self.zones_visited.contains(zone_name) {
                        self.zones_visited.push(zone_name.clone());
                    }
                }
                GameLoopEvent::DialogueEnded { .. } => {
                    self.dialogues_completed += 1;
                }
                GameLoopEvent::StormDecisionMade { choice } => {
                    self.storm_choice = Some(*choice);
                }
                GameLoopEvent::StormResolved { .. } => {
                    // Storm resolved — could track additional data here
                }
                GameLoopEvent::CinematicStarted { .. }
                | GameLoopEvent::CinematicFinished { .. }
                | GameLoopEvent::DialogueDisplay { .. }
                | GameLoopEvent::StormDecisionPrompt => {
                    // Informational — no telemetry impact
                }
            }
        }
    }

    // ── Manual recording (for events outside GameLoopEvent) ────────

    /// Record combat damage dealt by the player.
    ///
    /// NaN, infinity, and negative values are silently rejected.
    pub fn record_damage_dealt(&mut self, amount: f32) {
        if amount.is_finite() && amount >= 0.0 {
            self.damage_dealt += amount;
        }
    }

    /// Record combat damage taken by the player.
    ///
    /// NaN, infinity, and negative values are silently rejected.
    pub fn record_damage_taken(&mut self, amount: f32) {
        if amount.is_finite() && amount >= 0.0 {
            self.damage_taken += amount;
        }
    }

    /// Record an enemy defeat.
    pub fn record_enemy_defeated(&mut self) {
        self.enemies_defeated += 1;
    }

    /// Record a player death.
    pub fn record_death(&mut self) {
        self.deaths += 1;
    }

    /// Record a combo landed.
    pub fn record_combo(&mut self) {
        self.combos_landed += 1;
    }

    /// Record a secret found.
    pub fn record_secret(&mut self) {
        self.secrets_found += 1;
    }

    /// Record an anchor repair.
    pub fn record_anchor_repaired(&mut self) {
        self.anchors_repaired += 1;
    }

    /// Record echoes gained.
    pub fn record_echoes_collected(&mut self, amount: u32) {
        self.echoes_collected += amount;
    }

    /// Record echoes spent.
    pub fn record_echoes_spent(&mut self, amount: u32) {
        self.echoes_spent += amount;
    }

    /// Record a companion support action.
    pub fn record_companion_action(&mut self) {
        self.companion_support_actions += 1;
    }

    /// Set the final companion affinity rank (at end of run).
    pub fn set_final_affinity_rank(&mut self, rank: AffinityRank) {
        self.final_affinity_rank = Some(rank);
    }

    /// Record a boss phase transition.
    pub fn record_boss_phase(&mut self, phase: BossPhase) {
        if !self.boss_phases_seen.contains(&phase) {
            self.boss_phases_seen.push(phase);
        }
    }

    /// Record a successfully dodged boss telegraph.
    pub fn record_telegraph_dodged(&mut self) {
        self.boss_telegraphs_dodged += 1;
    }

    /// Mark the boss fight as started (begins timing).
    pub fn start_boss_fight(&mut self) {
        self.boss_fight_active = true;
    }

    /// Mark the boss as defeated (stops timing).
    pub fn finish_boss_fight(&mut self) {
        self.boss_fight_active = false;
        self.boss_defeated = true;
    }

    // ── Finalization ───────────────────────────────────────────────

    /// Populate a [`RecapPanel`] with all collected metrics and compute ratings.
    pub fn finalize(&self, panel: &mut RecapPanel) {
        // Combat
        panel.record_float(
            "damage_dealt",
            "Damage Dealt",
            self.damage_dealt as f64,
            MetricCategory::Combat,
            "HP",
        );
        panel.record_float(
            "damage_taken",
            "Damage Taken",
            self.damage_taken as f64,
            MetricCategory::Combat,
            "HP",
        );
        panel.record_int(
            "enemies_defeated",
            "Enemies Defeated",
            self.enemies_defeated as i64,
            MetricCategory::Combat,
            "",
        );
        panel.record_int(
            "deaths",
            "Deaths",
            self.deaths as i64,
            MetricCategory::Combat,
            "",
        );
        panel.record_int(
            "combos",
            "Combos Landed",
            self.combos_landed as i64,
            MetricCategory::Combat,
            "",
        );

        // Exploration
        panel.record_int(
            "zones_visited",
            "Zones Visited",
            self.zones_visited.len() as i64,
            MetricCategory::Exploration,
            "",
        );
        panel.record_int(
            "secrets_found",
            "Secrets Found",
            self.secrets_found as i64,
            MetricCategory::Exploration,
            "",
        );

        // Weaving
        panel.record_int(
            "anchors_repaired",
            "Anchors Repaired",
            self.anchors_repaired as i64,
            MetricCategory::Weaving,
            "",
        );
        panel.record_int(
            "echoes_collected",
            "Echoes Collected",
            self.echoes_collected as i64,
            MetricCategory::Weaving,
            "",
        );
        panel.record_int(
            "echoes_spent",
            "Echoes Spent",
            self.echoes_spent as i64,
            MetricCategory::Weaving,
            "",
        );

        // Companion
        panel.record_int(
            "companion_actions",
            "Support Actions",
            self.companion_support_actions as i64,
            MetricCategory::Companion,
            "",
        );
        if let Some(rank) = &self.final_affinity_rank {
            panel.record_text(
                "final_rank",
                "Final Rank",
                format!("{}", rank),
                MetricCategory::Companion,
            );
        }

        // Narrative
        if let Some(choice) = &self.storm_choice {
            panel.record_text(
                "storm_choice",
                "Storm Decision",
                format!("{:?}", choice),
                MetricCategory::Narrative,
            );
        }
        panel.record_int(
            "dialogues_completed",
            "Dialogues Completed",
            self.dialogues_completed as i64,
            MetricCategory::Narrative,
            "",
        );

        // Timing
        panel.record_float(
            "total_time",
            "Total Time",
            self.total_time as f64,
            MetricCategory::Timing,
            "s",
        );
        panel.record_float(
            "boss_fight_time",
            "Boss Fight",
            self.boss_fight_time as f64,
            MetricCategory::Timing,
            "s",
        );

        // Compute ratings
        panel.finalize_with(|category, metrics| self.rate_category(category, metrics));
    }

    /// Category rating logic based on actual gameplay performance.
    fn rate_category(
        &self,
        category: MetricCategory,
        _metrics: &[&crate::recap_panel::MetricEntry],
    ) -> Rating {
        match category {
            MetricCategory::Combat => {
                let kill_ratio = if self.deaths > 0 {
                    self.enemies_defeated as f32 / self.deaths as f32
                } else if self.enemies_defeated > 0 {
                    10.0 // Deathless → high score
                } else {
                    1.0
                };
                match kill_ratio {
                    r if r >= 8.0 => Rating::S,
                    r if r >= 5.0 => Rating::A,
                    r if r >= 3.0 => Rating::B,
                    r if r >= 1.0 => Rating::C,
                    _ => Rating::D,
                }
            }
            MetricCategory::Exploration => {
                let zone_count = self.zones_visited.len();
                match zone_count {
                    n if n >= 6 => Rating::S,
                    n if n >= 5 => Rating::A,
                    n if n >= 4 => Rating::B,
                    n if n >= 2 => Rating::C,
                    _ => Rating::D,
                }
            }
            MetricCategory::Weaving => match self.anchors_repaired {
                n if n >= 5 => Rating::S,
                n if n >= 3 => Rating::A,
                n if n >= 2 => Rating::B,
                n if n >= 1 => Rating::C,
                _ => Rating::D,
            },
            MetricCategory::Companion => match self.final_affinity_rank {
                Some(AffinityRank::Synced) => Rating::S,
                Some(AffinityRank::Bonded) => Rating::A,
                Some(AffinityRank::Cooperative) => Rating::B,
                Some(AffinityRank::Cautious) => Rating::C,
                _ => Rating::D,
            },
            MetricCategory::Narrative => {
                // Having made a storm choice + completing dialogues is good
                let has_choice = self.storm_choice.is_some();
                let dialogue_count = self.dialogues_completed;
                match (has_choice, dialogue_count) {
                    (true, n) if n >= 3 => Rating::S,
                    (true, n) if n >= 2 => Rating::A,
                    (true, _) => Rating::B,
                    (false, n) if n >= 1 => Rating::C,
                    _ => Rating::D,
                }
            }
            MetricCategory::Timing => {
                // Faster = better (under 30 min = S, under 40 = A, etc.)
                let minutes = self.total_time / 60.0;
                match minutes {
                    m if m <= 25.0 => Rating::S,
                    m if m <= 35.0 => Rating::A,
                    m if m <= 45.0 => Rating::B,
                    m if m <= 60.0 => Rating::C,
                    _ => Rating::D,
                }
            }
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_loop::GameLoopEvent;
    use crate::recap_panel::RecapPanel;
    use astraweave_scene::world_partition::GridCoord;

    #[test]
    fn initial_state() {
        let tc = TelemetryCollector::new();
        assert_eq!(tc.damage_dealt, 0.0);
        assert_eq!(tc.total_time, 0.0);
        assert!(tc.zones_visited.is_empty());
    }

    #[test]
    fn process_zone_events() {
        let mut tc = TelemetryCollector::new();
        let events = vec![
            GameLoopEvent::ZoneLoading {
                zone_name: "Z0_loomspire_sanctum".to_string(),
                coord: GridCoord::new(100, 0, 0),
            },
            GameLoopEvent::ZoneLoading {
                zone_name: "Z1_echo_grove".to_string(),
                coord: GridCoord::new(101, 0, 0),
            },
        ];
        tc.process_events(&events, 1.0 / 60.0);
        assert_eq!(tc.zones_visited.len(), 2);
    }

    #[test]
    fn duplicate_zone_not_counted() {
        let mut tc = TelemetryCollector::new();
        let event = GameLoopEvent::ZoneLoading {
            zone_name: "Z0_loomspire_sanctum".to_string(),
            coord: GridCoord::new(100, 0, 0),
        };
        tc.process_events(std::slice::from_ref(&event), 0.016);
        tc.process_events(std::slice::from_ref(&event), 0.016);
        assert_eq!(tc.zones_visited.len(), 1);
    }

    #[test]
    fn dialogue_completion_counted() {
        let mut tc = TelemetryCollector::new();
        tc.process_events(
            &[GameLoopEvent::DialogueEnded {
                dialogue_id: "intro".to_string(),
            }],
            0.016,
        );
        assert_eq!(tc.dialogues_completed, 1);
    }

    #[test]
    fn storm_choice_recorded() {
        let mut tc = TelemetryCollector::new();
        tc.process_events(
            &[GameLoopEvent::StormDecisionMade {
                choice: StormChoice::Redirect,
            }],
            0.016,
        );
        assert_eq!(tc.storm_choice, Some(StormChoice::Redirect));
    }

    #[test]
    fn manual_combat_recording() {
        let mut tc = TelemetryCollector::new();
        tc.record_damage_dealt(500.0);
        tc.record_damage_dealt(300.0);
        tc.record_damage_taken(100.0);
        tc.record_enemy_defeated();
        tc.record_enemy_defeated();
        tc.record_combo();

        assert_eq!(tc.damage_dealt, 800.0);
        assert_eq!(tc.damage_taken, 100.0);
        assert_eq!(tc.enemies_defeated, 2);
        assert_eq!(tc.combos_landed, 1);
    }

    #[test]
    fn boss_fight_timing() {
        let mut tc = TelemetryCollector::new();

        // Regular play
        tc.process_events(&[], 10.0);
        assert_eq!(tc.total_time, 10.0);
        assert_eq!(tc.boss_fight_time, 0.0);

        // Boss fight starts
        tc.start_boss_fight();
        tc.process_events(&[], 5.0);
        assert_eq!(tc.boss_fight_time, 5.0);

        // Boss defeated
        tc.finish_boss_fight();
        tc.process_events(&[], 3.0);
        assert_eq!(tc.boss_fight_time, 5.0); // Stopped accumulating
        assert!(tc.boss_defeated);
    }

    #[test]
    fn finalize_populates_recap() {
        let mut tc = TelemetryCollector::new();
        tc.record_damage_dealt(5000.0);
        tc.record_enemy_defeated();
        tc.record_enemy_defeated();
        tc.record_enemy_defeated();
        tc.record_anchor_repaired();
        tc.record_anchor_repaired();
        tc.record_anchor_repaired();
        tc.record_echoes_collected(50);
        tc.set_final_affinity_rank(AffinityRank::Bonded);
        tc.storm_choice = Some(StormChoice::Stabilize);
        tc.dialogues_completed = 3;
        tc.total_time = 1500.0; // 25 min

        for z in &["Z0", "Z1", "Z2", "Z3", "Z4"] {
            tc.zones_visited.push(z.to_string());
        }

        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);

        assert!(panel.is_finalized());
        assert!(panel.overall_rating().is_some());
        assert!(panel.metric_count() > 10);

        let zones = panel.metric("zones_visited").unwrap();
        assert_eq!(zones.value.as_f64(), 5.0);
    }

    #[test]
    fn combat_rating_deathless() {
        let mut tc = TelemetryCollector::new();
        tc.record_enemy_defeated();
        tc.record_enemy_defeated();
        // No deaths → kill_ratio = 10.0 → S
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        let combat_rating = panel.category_ratings().get(&MetricCategory::Combat);
        assert_eq!(combat_rating, Some(&Rating::S));
    }

    #[test]
    fn exploration_rating_all_zones() {
        let mut tc = TelemetryCollector::new();
        for z in &["Z0", "Z1", "Z2", "Z3", "Z4", "Z2a"] {
            tc.zones_visited.push(z.to_string());
        }
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        let rating = panel.category_ratings().get(&MetricCategory::Exploration);
        assert_eq!(rating, Some(&Rating::S));
    }

    #[test]
    fn timing_rating_fast_run() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 20.0 * 60.0; // 20 minutes
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        let rating = panel.category_ratings().get(&MetricCategory::Timing);
        assert_eq!(rating, Some(&Rating::S));
    }

    #[test]
    fn timing_rating_slow_run() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 90.0 * 60.0; // 90 minutes
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        let rating = panel.category_ratings().get(&MetricCategory::Timing);
        assert_eq!(rating, Some(&Rating::D));
    }
}
