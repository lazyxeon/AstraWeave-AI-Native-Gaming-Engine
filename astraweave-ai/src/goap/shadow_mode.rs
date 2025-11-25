// Shadow Mode: Side-by-side comparison of GOAP vs RuleOrchestrator
// Phase 2: Engine Integration

use crate::orchestrator::Orchestrator;
use astraweave_core::{PlanIntent, WorldSnapshot};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Comparison result between two planning approaches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanComparison {
    pub timestamp: f32,
    pub tactical_summary: String,
    pub rule_plan: PlanSummary,
    pub goap_plan: PlanSummary,
    pub differences: PlanDiff,
    pub metrics: ComparisonMetrics,
}

/// Summary of a single plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanSummary {
    pub plan_id: String,
    pub step_count: usize,
    pub action_types: Vec<String>,
    pub planning_time_ms: f64,
    pub empty: bool,
}

/// Differences between two plans
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanDiff {
    pub step_count_diff: i32,
    pub actions_in_common: usize,
    pub unique_to_rule: Vec<String>,
    pub unique_to_goap: Vec<String>,
    pub order_differs: bool,
    pub similarity_score: f32, // 0.0 = completely different, 1.0 = identical
}

/// Performance and quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonMetrics {
    pub rule_faster: bool,
    pub time_difference_ms: f64,
    pub goap_more_steps: bool,
    pub both_empty: bool,
    pub both_non_empty: bool,
}

impl PlanComparison {
    /// Create a new comparison from two plans
    pub fn new(
        snap: &WorldSnapshot,
        rule_plan: PlanIntent,
        rule_time_ms: f64,
        goap_plan: PlanIntent,
        goap_time_ms: f64,
    ) -> Self {
        let tactical_summary = crate::goap::adapter::SnapshotAdapter::tactical_summary(snap);

        let rule_summary = PlanSummary::from_intent(&rule_plan, rule_time_ms);
        let goap_summary = PlanSummary::from_intent(&goap_plan, goap_time_ms);

        let differences = PlanDiff::compute(&rule_summary, &goap_summary);
        let metrics =
            ComparisonMetrics::compute(&rule_summary, &goap_summary, rule_time_ms, goap_time_ms);

        Self {
            timestamp: snap.t,
            tactical_summary,
            rule_plan: rule_summary,
            goap_plan: goap_summary,
            differences,
            metrics,
        }
    }

    /// Format as human-readable log entry
    pub fn to_log_entry(&self) -> String {
        let mut log = String::new();

        log.push_str(&format!(
            "\n═══ Shadow Mode Comparison @ t={:.1}s ═══\n",
            self.timestamp
        ));
        log.push_str(&format!("Situation: {}\n\n", self.tactical_summary));

        log.push_str("🤖 RuleOrchestrator:\n");
        log.push_str(&format!("  Plan ID: {}\n", self.rule_plan.plan_id));
        log.push_str(&format!("  Steps: {}\n", self.rule_plan.step_count));
        log.push_str(&format!("  Actions: {:?}\n", self.rule_plan.action_types));
        log.push_str(&format!(
            "  Planning Time: {:.2}ms\n\n",
            self.rule_plan.planning_time_ms
        ));

        log.push_str("🧠 GOAP Planner:\n");
        log.push_str(&format!("  Plan ID: {}\n", self.goap_plan.plan_id));
        log.push_str(&format!("  Steps: {}\n", self.goap_plan.step_count));
        log.push_str(&format!("  Actions: {:?}\n", self.goap_plan.action_types));
        log.push_str(&format!(
            "  Planning Time: {:.2}ms\n\n",
            self.goap_plan.planning_time_ms
        ));

        log.push_str("📊 Differences:\n");
        log.push_str(&format!(
            "  Similarity Score: {:.1}%\n",
            self.differences.similarity_score * 100.0
        ));
        log.push_str(&format!(
            "  Common Actions: {}\n",
            self.differences.actions_in_common
        ));

        if !self.differences.unique_to_rule.is_empty() {
            log.push_str(&format!(
                "  Only in Rule: {:?}\n",
                self.differences.unique_to_rule
            ));
        }
        if !self.differences.unique_to_goap.is_empty() {
            log.push_str(&format!(
                "  Only in GOAP: {:?}\n",
                self.differences.unique_to_goap
            ));
        }
        if self.differences.order_differs {
            log.push_str("  ⚠ Action order differs\n");
        }

        log.push_str("\n📈 Metrics:\n");
        if self.metrics.rule_faster {
            log.push_str(&format!(
                "  ✓ Rule faster by {:.2}ms\n",
                self.metrics.time_difference_ms
            ));
        } else {
            log.push_str(&format!(
                "  ✓ GOAP faster by {:.2}ms\n",
                self.metrics.time_difference_ms.abs()
            ));
        }

        if self.metrics.both_empty {
            log.push_str("  ℹ Both plans empty (no action needed)\n");
        } else if !self.metrics.both_non_empty {
            log.push_str("  ⚠ One planner failed to generate plan\n");
        }

        log.push_str("═══════════════════════════════════════\n");
        log
    }

    /// Export as JSON for offline analysis
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

impl PlanSummary {
    fn from_intent(intent: &PlanIntent, planning_time_ms: f64) -> Self {
        let action_types: Vec<String> = intent
            .steps
            .iter()
            .map(|step| {
                format!("{:?}", step)
                    .split_whitespace()
                    .next()
                    .unwrap_or("Unknown")
                    .to_string()
            })
            .collect();

        Self {
            plan_id: intent.plan_id.clone(),
            step_count: intent.steps.len(),
            action_types,
            planning_time_ms,
            empty: intent.steps.is_empty(),
        }
    }
}

impl PlanDiff {
    fn compute(rule: &PlanSummary, goap: &PlanSummary) -> Self {
        let step_count_diff = goap.step_count as i32 - rule.step_count as i32;

        // Find common actions (ignoring order)
        let rule_set: std::collections::HashSet<_> = rule.action_types.iter().collect();
        let goap_set: std::collections::HashSet<_> = goap.action_types.iter().collect();

        let actions_in_common = rule_set.intersection(&goap_set).count();

        let unique_to_rule: Vec<String> = rule
            .action_types
            .iter()
            .filter(|a| !goap_set.contains(a))
            .map(|s| s.clone())
            .collect();

        let unique_to_goap: Vec<String> = goap
            .action_types
            .iter()
            .filter(|a| !rule_set.contains(a))
            .map(|s| s.clone())
            .collect();

        // Check if order differs (for common actions)
        let order_differs = rule.action_types != goap.action_types;

        // Calculate similarity score
        let total_actions = (rule.step_count + goap.step_count).max(1);
        let similarity_score = (actions_in_common * 2) as f32 / total_actions as f32;

        Self {
            step_count_diff,
            actions_in_common,
            unique_to_rule,
            unique_to_goap,
            order_differs,
            similarity_score,
        }
    }
}

impl ComparisonMetrics {
    fn compute(
        rule: &PlanSummary,
        goap: &PlanSummary,
        rule_time_ms: f64,
        goap_time_ms: f64,
    ) -> Self {
        Self {
            rule_faster: rule_time_ms < goap_time_ms,
            time_difference_ms: rule_time_ms - goap_time_ms,
            goap_more_steps: goap.step_count > rule.step_count,
            both_empty: rule.empty && goap.empty,
            both_non_empty: !rule.empty && !goap.empty,
        }
    }
}

/// Shadow mode runner that executes both planners
pub struct ShadowModeRunner {
    comparisons: Vec<PlanComparison>,
    log_to_console: bool,
}

impl ShadowModeRunner {
    pub fn new(log_to_console: bool) -> Self {
        Self {
            comparisons: Vec::new(),
            log_to_console,
        }
    }

    /// Run both planners and compare results
    pub fn compare(
        &mut self,
        snap: &WorldSnapshot,
        rule_orchestrator: &crate::orchestrator::RuleOrchestrator,
        goap_orchestrator: &mut crate::goap::orchestrator::GOAPOrchestrator,
    ) -> &PlanComparison {
        // Execute rule-based planner
        let rule_start = Instant::now();
        let rule_plan = rule_orchestrator.propose_plan(snap);
        let rule_time_ms = rule_start.elapsed().as_secs_f64() * 1000.0;

        // Execute GOAP planner
        let goap_start = Instant::now();
        let goap_plan = goap_orchestrator.propose_plan(snap);
        let goap_time_ms = goap_start.elapsed().as_secs_f64() * 1000.0;

        // Create comparison
        let comparison =
            PlanComparison::new(snap, rule_plan, rule_time_ms, goap_plan, goap_time_ms);

        if self.log_to_console {
            println!("{}", comparison.to_log_entry());
        }

        self.comparisons.push(comparison);
        self.comparisons.last().unwrap()
    }

    /// Get all comparisons
    pub fn get_comparisons(&self) -> &[PlanComparison] {
        &self.comparisons
    }

    /// Generate aggregate statistics
    pub fn generate_report(&self) -> ShadowModeReport {
        ShadowModeReport::from_comparisons(&self.comparisons)
    }
}

/// Aggregate report from multiple comparisons
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowModeReport {
    pub total_comparisons: usize,
    pub avg_similarity: f32,
    pub goap_faster_count: usize,
    pub rule_faster_count: usize,
    pub avg_time_diff_ms: f64,
    pub both_empty_count: usize,
    pub avg_rule_steps: f32,
    pub avg_goap_steps: f32,
}

impl ShadowModeReport {
    fn from_comparisons(comparisons: &[PlanComparison]) -> Self {
        if comparisons.is_empty() {
            return Self::default();
        }

        let total = comparisons.len();

        let avg_similarity = comparisons
            .iter()
            .map(|c| c.differences.similarity_score)
            .sum::<f32>()
            / total as f32;

        let goap_faster_count = comparisons
            .iter()
            .filter(|c| !c.metrics.rule_faster)
            .count();

        let rule_faster_count = total - goap_faster_count;

        let avg_time_diff_ms = comparisons
            .iter()
            .map(|c| c.metrics.time_difference_ms)
            .sum::<f64>()
            / total as f64;

        let both_empty_count = comparisons.iter().filter(|c| c.metrics.both_empty).count();

        let avg_rule_steps = comparisons
            .iter()
            .map(|c| c.rule_plan.step_count)
            .sum::<usize>() as f32
            / total as f32;

        let avg_goap_steps = comparisons
            .iter()
            .map(|c| c.goap_plan.step_count)
            .sum::<usize>() as f32
            / total as f32;

        Self {
            total_comparisons: total,
            avg_similarity,
            goap_faster_count,
            rule_faster_count,
            avg_time_diff_ms,
            both_empty_count,
            avg_rule_steps,
            avg_goap_steps,
        }
    }

    pub fn print_report(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║     Shadow Mode Aggregate Report      ║");
        println!("╚════════════════════════════════════════╝\n");

        println!("📊 Total Comparisons: {}", self.total_comparisons);
        println!("🎯 Average Similarity: {:.1}%", self.avg_similarity * 100.0);
        println!("\n⏱️  Performance:");
        println!(
            "   • GOAP faster: {} times ({:.1}%)",
            self.goap_faster_count,
            (self.goap_faster_count as f32 / self.total_comparisons as f32) * 100.0
        );
        println!(
            "   • Rule faster: {} times ({:.1}%)",
            self.rule_faster_count,
            (self.rule_faster_count as f32 / self.total_comparisons as f32) * 100.0
        );
        println!(
            "   • Avg time difference: {:.2}ms",
            self.avg_time_diff_ms.abs()
        );

        println!("\n📈 Plan Characteristics:");
        println!("   • Avg Rule steps: {:.1}", self.avg_rule_steps);
        println!("   • Avg GOAP steps: {:.1}", self.avg_goap_steps);
        println!("   • Both empty: {} times", self.both_empty_count);

        println!("\n════════════════════════════════════════\n");
    }
}

impl Default for ShadowModeReport {
    fn default() -> Self {
        Self {
            total_comparisons: 0,
            avg_similarity: 0.0,
            goap_faster_count: 0,
            rule_faster_count: 0,
            avg_time_diff_ms: 0.0,
            both_empty_count: 0,
            avg_rule_steps: 0.0,
            avg_goap_steps: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{ActionStep, CompanionState, IVec2, PlayerState};
    use std::collections::BTreeMap;

    fn make_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 1.0,
            player: PlayerState {
                hp: 100,
                physics_context: None,
                pos: IVec2 { x: 0, y: 0 },
                stance: "stand".to_string(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 20,
                cooldowns: BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            enemies: vec![],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        }
    }

    #[test]
    fn test_plan_comparison() {
        let snap = make_test_snapshot();

        let rule_plan = PlanIntent {
            plan_id: "rule-1".to_string(),
            steps: vec![
                ActionStep::MoveTo {
                    x: 10,
                    y: 10,
                    speed: None,
                },
                ActionStep::Attack { target_id: 1 },
            ],
        };

        let goap_plan = PlanIntent {
            plan_id: "goap-1".to_string(),
            steps: vec![
                ActionStep::MoveTo {
                    x: 10,
                    y: 10,
                    speed: None,
                },
                ActionStep::Reload,
                ActionStep::Attack { target_id: 1 },
            ],
        };

        let comparison = PlanComparison::new(&snap, rule_plan, 1.5, goap_plan, 2.0);

        assert_eq!(comparison.differences.actions_in_common, 2); // MoveTo and Attack
        assert_eq!(comparison.differences.step_count_diff, 1); // GOAP has 1 more step
        assert!(comparison.metrics.rule_faster);
    }

    #[test]
    fn test_shadow_mode_report() {
        let mut comparisons = Vec::new();

        for i in 0..5 {
            let snap = make_test_snapshot();
            let rule_plan = PlanIntent {
                plan_id: format!("rule-{}", i),
                steps: vec![ActionStep::MoveTo {
                    x: i,
                    y: i,
                    speed: None,
                }],
            };
            let goap_plan = PlanIntent {
                plan_id: format!("goap-{}", i),
                steps: vec![ActionStep::MoveTo {
                    x: i,
                    y: i,
                    speed: None,
                }],
            };

            comparisons.push(PlanComparison::new(&snap, rule_plan, 1.0, goap_plan, 1.5));
        }

        let report = ShadowModeReport::from_comparisons(&comparisons);

        assert_eq!(report.total_comparisons, 5);
        assert_eq!(report.rule_faster_count, 5); // Rule is always faster in this test
        assert!(report.avg_similarity > 0.9); // Plans are very similar
    }
}
