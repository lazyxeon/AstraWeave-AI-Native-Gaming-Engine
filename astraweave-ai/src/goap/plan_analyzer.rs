use super::{Action, ActionHistory, WorldState};
use std::collections::HashMap;

/// Metrics for analyzing plan quality
#[derive(Debug, Clone)]
pub struct PlanMetrics {
    pub total_cost: f32,
    pub total_risk: f32,
    pub action_count: usize,
    pub estimated_duration: f32,
    pub success_probability: f32,
    pub bottlenecks: Vec<Bottleneck>,
    pub action_breakdown: HashMap<String, ActionMetrics>,
}

/// Metrics for individual actions
#[derive(Debug, Clone)]
pub struct ActionMetrics {
    pub cost: f32,
    pub risk: f32,
    pub success_rate: f32,
    pub avg_duration: f32,
    pub executions: u32,
}

/// Identified bottleneck in a plan
#[derive(Debug, Clone)]
pub struct Bottleneck {
    pub action_name: String,
    pub reason: BottleneckReason,
    pub severity: f32, // 0.0 - 1.0
}

#[derive(Debug, Clone, PartialEq)]
pub enum BottleneckReason {
    HighCost,
    HighRisk,
    LowSuccessRate,
    LongDuration,
}

/// Comparison report between two plans
#[derive(Debug, Clone)]
pub struct ComparisonReport {
    pub cost_diff: f32,
    pub risk_diff: f32,
    pub duration_diff: f32,
    pub success_prob_diff: f32,
    pub better_plan: PlanComparison,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlanComparison {
    Plan1Better,
    Plan2Better,
    Similar,
}

/// Optimization suggestion
#[derive(Debug, Clone)]
pub struct Suggestion {
    pub message: String,
    pub priority: SuggestionPriority,
    pub estimated_improvement: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SuggestionPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Plan analyzer for quality metrics and optimization
pub struct PlanAnalyzer;

impl PlanAnalyzer {
    /// Analyze a plan and produce metrics
    pub fn analyze(
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
        start_state: &WorldState,
    ) -> PlanMetrics {
        let mut total_cost = 0.0;
        let mut total_risk = 0.0;
        let mut total_duration = 0.0;
        let mut action_breakdown = HashMap::new();
        let mut current_state = start_state.clone();

        for action_name in plan {
            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                let cost = action.calculate_cost(&current_state, history);
                let success_prob = action.success_probability(&current_state, history);
                let risk = 1.0 - success_prob;

                let stats = history.get_action_stats(action_name);
                let duration = stats.map(|s| s.avg_duration).unwrap_or(1.0);
                let executions = stats.map(|s| s.executions).unwrap_or(0);
                let success_rate = stats.map(|s| s.success_rate()).unwrap_or(0.5);

                total_cost += cost;
                total_risk += risk;
                total_duration += duration;

                action_breakdown.insert(
                    action_name.clone(),
                    ActionMetrics {
                        cost,
                        risk,
                        success_rate,
                        avg_duration: duration,
                        executions,
                    },
                );

                current_state.apply_effects(action.effects());
            }
        }

        // Calculate overall success probability (product of individual probabilities)
        let success_probability = action_breakdown.values().map(|m| m.success_rate).product();

        // Identify bottlenecks
        let bottlenecks = Self::identify_bottlenecks(&action_breakdown);

        PlanMetrics {
            total_cost,
            total_risk,
            action_count: plan.len(),
            estimated_duration: total_duration,
            success_probability,
            bottlenecks,
            action_breakdown,
        }
    }

    /// Compare two plans
    pub fn compare(metrics1: &PlanMetrics, metrics2: &PlanMetrics) -> ComparisonReport {
        let cost_diff = metrics2.total_cost - metrics1.total_cost;
        let risk_diff = metrics2.total_risk - metrics1.total_risk;
        let duration_diff = metrics2.estimated_duration - metrics1.estimated_duration;
        let success_prob_diff = metrics2.success_probability - metrics1.success_probability;

        // Scoring: lower cost, risk, duration and higher success prob is better
        let score1 =
            -metrics1.total_cost - metrics1.total_risk * 2.0 - metrics1.estimated_duration * 0.1
                + metrics1.success_probability * 10.0;
        let score2 =
            -metrics2.total_cost - metrics2.total_risk * 2.0 - metrics2.estimated_duration * 0.1
                + metrics2.success_probability * 10.0;

        let better_plan = if (score1 - score2).abs() < 0.5 {
            PlanComparison::Similar
        } else if score1 > score2 {
            PlanComparison::Plan1Better
        } else {
            PlanComparison::Plan2Better
        };

        let mut recommendations = Vec::new();

        if cost_diff.abs() > 1.0 {
            if cost_diff > 0.0 {
                recommendations.push(format!("Plan 1 is {:.1} cost units cheaper", cost_diff));
            } else {
                recommendations.push(format!("Plan 2 is {:.1} cost units cheaper", -cost_diff));
            }
        }

        if risk_diff.abs() > 0.1 {
            if risk_diff > 0.0 {
                recommendations.push(format!("Plan 1 has {:.2} less risk", risk_diff));
            } else {
                recommendations.push(format!("Plan 2 has {:.2} less risk", -risk_diff));
            }
        }

        if duration_diff.abs() > 2.0 {
            if duration_diff > 0.0 {
                recommendations.push(format!("Plan 1 is {:.1}s faster", duration_diff));
            } else {
                recommendations.push(format!("Plan 2 is {:.1}s faster", -duration_diff));
            }
        }

        ComparisonReport {
            cost_diff,
            risk_diff,
            duration_diff,
            success_prob_diff,
            better_plan,
            recommendations,
        }
    }

    /// Suggest optimizations for a plan
    pub fn suggest_optimizations(metrics: &PlanMetrics) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // Check for high overall cost
        if metrics.total_cost > 20.0 {
            suggestions.push(Suggestion {
                message: format!(
                    "Plan has high total cost ({:.1}). Consider shorter alternative paths.",
                    metrics.total_cost
                ),
                priority: SuggestionPriority::High,
                estimated_improvement: Some(metrics.total_cost * 0.2),
            });
        }

        // Check for high overall risk
        if metrics.total_risk > 2.0 {
            suggestions.push(Suggestion {
                message: format!(
                    "Plan has high cumulative risk ({:.2}). Consider more reliable actions.",
                    metrics.total_risk
                ),
                priority: SuggestionPriority::Critical,
                estimated_improvement: Some(metrics.total_risk * 0.3),
            });
        }

        // Check for low success probability
        if metrics.success_probability < 0.5 {
            suggestions.push(Suggestion {
                message: format!(
                    "Plan has low success probability ({:.1}%). Consider adding fallback actions.",
                    metrics.success_probability * 100.0
                ),
                priority: SuggestionPriority::Critical,
                estimated_improvement: None,
            });
        }

        // Check for long duration
        if metrics.estimated_duration > 30.0 {
            suggestions.push(Suggestion {
                message: format!(
                    "Plan takes a long time ({:.1}s). Look for faster action sequences.",
                    metrics.estimated_duration
                ),
                priority: SuggestionPriority::Medium,
                estimated_improvement: Some(metrics.estimated_duration * 0.15),
            });
        }

        // Analyze bottlenecks
        for bottleneck in &metrics.bottlenecks {
            match bottleneck.reason {
                BottleneckReason::HighCost => {
                    suggestions.push(Suggestion {
                        message: format!(
                            "Action '{}' is expensive. Consider alternatives or optimize preconditions.",
                            bottleneck.action_name
                        ),
                        priority: if bottleneck.severity > 0.7 {
                            SuggestionPriority::High
                        } else {
                            SuggestionPriority::Medium
                        },
                        estimated_improvement: Some(bottleneck.severity * 5.0),
                    });
                }
                BottleneckReason::HighRisk => {
                    suggestions.push(Suggestion {
                        message: format!(
                            "Action '{}' is risky. Add supporting actions to improve success rate.",
                            bottleneck.action_name
                        ),
                        priority: SuggestionPriority::Critical,
                        estimated_improvement: None,
                    });
                }
                BottleneckReason::LowSuccessRate => {
                    suggestions.push(Suggestion {
                        message: format!(
                            "Action '{}' has low historical success rate. Review preconditions or avoid.",
                            bottleneck.action_name
                        ),
                        priority: SuggestionPriority::High,
                        estimated_improvement: None,
                    });
                }
                BottleneckReason::LongDuration => {
                    suggestions.push(Suggestion {
                        message: format!(
                            "Action '{}' takes a long time. Consider faster alternatives.",
                            bottleneck.action_name
                        ),
                        priority: SuggestionPriority::Low,
                        estimated_improvement: Some(bottleneck.severity * 2.0),
                    });
                }
            }
        }

        // Check for very long plans
        if metrics.action_count > 10 {
            suggestions.push(Suggestion {
                message: format!(
                    "Plan has {} actions. Consider hierarchical decomposition or simpler goals.",
                    metrics.action_count
                ),
                priority: SuggestionPriority::Medium,
                estimated_improvement: None,
            });
        }

        // Sort by priority
        suggestions.sort_by(|a, b| b.priority.cmp(&a.priority));

        suggestions
    }

    /// Identify bottlenecks in a plan
    fn identify_bottlenecks(action_breakdown: &HashMap<String, ActionMetrics>) -> Vec<Bottleneck> {
        let mut bottlenecks = Vec::new();

        // Calculate averages
        let avg_cost: f32 =
            action_breakdown.values().map(|m| m.cost).sum::<f32>() / action_breakdown.len() as f32;
        let avg_risk: f32 =
            action_breakdown.values().map(|m| m.risk).sum::<f32>() / action_breakdown.len() as f32;
        let avg_duration: f32 = action_breakdown
            .values()
            .map(|m| m.avg_duration)
            .sum::<f32>()
            / action_breakdown.len() as f32;

        for (action_name, metrics) in action_breakdown {
            // Check for high cost (>2x average)
            if metrics.cost > avg_cost * 2.0 {
                bottlenecks.push(Bottleneck {
                    action_name: action_name.clone(),
                    reason: BottleneckReason::HighCost,
                    severity: (metrics.cost / (avg_cost * 2.0)).min(1.0),
                });
            }

            // Check for high risk (>2x average)
            if metrics.risk > avg_risk * 2.0 && metrics.risk > 0.3 {
                bottlenecks.push(Bottleneck {
                    action_name: action_name.clone(),
                    reason: BottleneckReason::HighRisk,
                    severity: (metrics.risk / (avg_risk * 2.0)).min(1.0),
                });
            }

            // Check for low success rate (<50%)
            if metrics.success_rate < 0.5 && metrics.executions > 3 {
                bottlenecks.push(Bottleneck {
                    action_name: action_name.clone(),
                    reason: BottleneckReason::LowSuccessRate,
                    severity: 1.0 - metrics.success_rate,
                });
            }

            // Check for long duration (>2x average)
            if metrics.avg_duration > avg_duration * 2.0 && metrics.avg_duration > 2.0 {
                bottlenecks.push(Bottleneck {
                    action_name: action_name.clone(),
                    reason: BottleneckReason::LongDuration,
                    severity: (metrics.avg_duration / (avg_duration * 2.0)).min(1.0),
                });
            }
        }

        // Sort by severity
        bottlenecks.sort_by(|a, b| b.severity.partial_cmp(&a.severity).unwrap());

        bottlenecks
    }

    /// Generate a human-readable report
    pub fn generate_report(metrics: &PlanMetrics) -> String {
        let mut report = String::new();

        report.push_str("=== Plan Analysis Report ===\n\n");

        report.push_str(&format!("Total Actions: {}\n", metrics.action_count));
        report.push_str(&format!("Total Cost: {:.2}\n", metrics.total_cost));
        report.push_str(&format!("Total Risk: {:.2}\n", metrics.total_risk));
        report.push_str(&format!(
            "Estimated Duration: {:.1}s\n",
            metrics.estimated_duration
        ));
        report.push_str(&format!(
            "Success Probability: {:.1}%\n",
            metrics.success_probability * 100.0
        ));

        if !metrics.bottlenecks.is_empty() {
            report.push_str("\n=== Bottlenecks ===\n");
            for bottleneck in &metrics.bottlenecks {
                report.push_str(&format!(
                    "- {}: {:?} (severity: {:.1}%)\n",
                    bottleneck.action_name,
                    bottleneck.reason,
                    bottleneck.severity * 100.0
                ));
            }
        }

        let suggestions = Self::suggest_optimizations(metrics);
        if !suggestions.is_empty() {
            report.push_str("\n=== Optimization Suggestions ===\n");
            for (i, suggestion) in suggestions.iter().take(5).enumerate() {
                report.push_str(&format!(
                    "{}. [{:?}] {}\n",
                    i + 1,
                    suggestion.priority,
                    suggestion.message
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goap::SimpleAction;
    use std::collections::BTreeMap;

    fn create_test_action(name: &str, cost: f32) -> Box<dyn Action> {
        Box::new(SimpleAction::new(
            name,
            BTreeMap::new(),
            BTreeMap::new(),
            cost,
        ))
    }

    #[test]
    fn test_analyze_simple_plan() {
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
            create_test_action("action2", 2.0),
        ];
        let plan = vec!["action1".to_string(), "action2".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let metrics = PlanAnalyzer::analyze(&plan, &actions, &history, &start);

        assert_eq!(metrics.action_count, 2);
        assert_eq!(metrics.total_cost, 3.0);
    }

    #[test]
    fn test_identify_high_cost_bottleneck() {
        let mut action_breakdown = HashMap::new();

        action_breakdown.insert(
            "cheap".to_string(),
            ActionMetrics {
                cost: 1.0,
                risk: 0.1,
                success_rate: 0.9,
                avg_duration: 1.0,
                executions: 10,
            },
        );

        action_breakdown.insert(
            "expensive".to_string(),
            ActionMetrics {
                cost: 20.0, // Much higher - avg is 10.5, 2x avg is 21.0, so 20.0 is close but we need >2x
                risk: 0.1,
                success_rate: 0.9,
                avg_duration: 1.0,
                executions: 10,
            },
        );

        action_breakdown.insert(
            "very_cheap".to_string(),
            ActionMetrics {
                cost: 0.5, // Add third action to bring avg down
                risk: 0.1,
                success_rate: 0.9,
                avg_duration: 1.0,
                executions: 10,
            },
        );

        let bottlenecks = PlanAnalyzer::identify_bottlenecks(&action_breakdown);

        // avg = (1.0 + 20.0 + 0.5) / 3 = 7.17, 2x = 14.33
        // 20.0 > 14.33, so should identify bottleneck
        assert!(!bottlenecks.is_empty());
        assert!(bottlenecks
            .iter()
            .any(|b| b.reason == BottleneckReason::HighCost));
    }

    #[test]
    fn test_compare_plans() {
        let metrics1 = PlanMetrics {
            total_cost: 5.0,
            total_risk: 0.5,
            action_count: 3,
            estimated_duration: 10.0,
            success_probability: 0.8,
            bottlenecks: vec![],
            action_breakdown: HashMap::new(),
        };

        let metrics2 = PlanMetrics {
            total_cost: 8.0, // Higher cost
            total_risk: 0.3, // Lower risk
            action_count: 4,
            estimated_duration: 12.0,
            success_probability: 0.9, // Higher success
            bottlenecks: vec![],
            action_breakdown: HashMap::new(),
        };

        let comparison = PlanAnalyzer::compare(&metrics1, &metrics2);

        assert!(comparison.cost_diff > 0.0); // Plan 2 costs more
        assert!(comparison.risk_diff < 0.0); // Plan 2 has less risk
    }

    #[test]
    fn test_suggest_optimizations_high_cost() {
        let metrics = PlanMetrics {
            total_cost: 25.0, // High cost
            total_risk: 0.2,
            action_count: 5,
            estimated_duration: 10.0,
            success_probability: 0.9,
            bottlenecks: vec![],
            action_breakdown: HashMap::new(),
        };

        let suggestions = PlanAnalyzer::suggest_optimizations(&metrics);

        assert!(!suggestions.is_empty());
        assert!(suggestions
            .iter()
            .any(|s| s.message.contains("high total cost")));
    }

    #[test]
    fn test_suggest_optimizations_low_success() {
        let metrics = PlanMetrics {
            total_cost: 5.0,
            total_risk: 0.2,
            action_count: 3,
            estimated_duration: 10.0,
            success_probability: 0.3, // Low success
            bottlenecks: vec![],
            action_breakdown: HashMap::new(),
        };

        let suggestions = PlanAnalyzer::suggest_optimizations(&metrics);

        assert!(!suggestions.is_empty());
        assert!(suggestions
            .iter()
            .any(|s| s.priority == SuggestionPriority::Critical));
    }

    #[test]
    fn test_generate_report() {
        let metrics = PlanMetrics {
            total_cost: 10.0,
            total_risk: 0.5,
            action_count: 5,
            estimated_duration: 15.0,
            success_probability: 0.85,
            bottlenecks: vec![],
            action_breakdown: HashMap::new(),
        };

        let report = PlanAnalyzer::generate_report(&metrics);

        assert!(report.contains("Plan Analysis Report"));
        assert!(report.contains("Total Actions: 5"));
        assert!(report.contains("Total Cost"));
    }
}
