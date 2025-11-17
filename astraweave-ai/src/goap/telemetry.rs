// Telemetry hooks for GOAP plan quality metrics and execution outcomes
// Phase 2: Engine Integration

use std::collections::VecDeque;
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

/// Telemetry event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TelemetryEvent {
    /// A plan was generated
    PlanGenerated {
        plan_id: String,
        timestamp: f32,
        step_count: usize,
        planning_time_ms: f64,
        action_names: Vec<String>,
    },
    /// A plan step was executed
    StepExecuted {
        plan_id: String,
        action_name: String,
        success: bool,
        duration_ms: f64,
    },
    /// A plan was completed
    PlanCompleted {
        plan_id: String,
        total_duration_ms: f64,
        steps_executed: usize,
        steps_failed: usize,
    },
    /// A plan was abandoned
    PlanAbandoned {
        plan_id: String,
        reason: String,
        steps_completed: usize,
    },
    /// Planning failed
    PlanningFailed {
        timestamp: f32,
        goal_description: String,
        reason: String,
    },
}

/// Aggregate metrics for reporting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanMetrics {
    pub total_plans_generated: usize,
    pub total_plans_completed: usize,
    pub total_plans_abandoned: usize,
    pub total_planning_failures: usize,
    pub avg_planning_time_ms: f64,
    pub avg_steps_per_plan: f32,
    pub avg_plan_success_rate: f32,
    pub fastest_plan_ms: f64,
    pub slowest_plan_ms: f64,
}

impl Default for PlanMetrics {
    fn default() -> Self {
        Self {
            total_plans_generated: 0,
            total_plans_completed: 0,
            total_plans_abandoned: 0,
            total_planning_failures: 0,
            avg_planning_time_ms: 0.0,
            avg_steps_per_plan: 0.0,
            avg_plan_success_rate: 0.0,
            fastest_plan_ms: f64::MAX,
            slowest_plan_ms: 0.0,
        }
    }
}

/// Telemetry collector with ring buffer
pub struct TelemetryCollector {
    events: VecDeque<TelemetryEvent>,
    max_events: usize,
    metrics: PlanMetrics,
    current_plan_start: Option<Instant>,
}

impl TelemetryCollector {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
            metrics: PlanMetrics::default(),
            current_plan_start: None,
        }
    }

    /// Record a telemetry event
    pub fn record(&mut self, event: TelemetryEvent) {
        // Update metrics based on event type
        match &event {
            TelemetryEvent::PlanGenerated { planning_time_ms, step_count, .. } => {
                self.metrics.total_plans_generated += 1;
                
                // Update averages
                let n = self.metrics.total_plans_generated as f64;
                self.metrics.avg_planning_time_ms = 
                    (self.metrics.avg_planning_time_ms * (n - 1.0) + planning_time_ms) / n;
                
                let n = self.metrics.total_plans_generated as f32;
                self.metrics.avg_steps_per_plan = 
                    (self.metrics.avg_steps_per_plan * (n - 1.0) + *step_count as f32) / n;
                
                // Update min/max
                if *planning_time_ms < self.metrics.fastest_plan_ms {
                    self.metrics.fastest_plan_ms = *planning_time_ms;
                }
                if *planning_time_ms > self.metrics.slowest_plan_ms {
                    self.metrics.slowest_plan_ms = *planning_time_ms;
                }
                
                self.current_plan_start = Some(Instant::now());
            }
            TelemetryEvent::PlanCompleted { .. } => {
                self.metrics.total_plans_completed += 1;
                self.update_success_rate();
                self.current_plan_start = None;
            }
            TelemetryEvent::PlanAbandoned { .. } => {
                self.metrics.total_plans_abandoned += 1;
                self.update_success_rate();
                self.current_plan_start = None;
            }
            TelemetryEvent::PlanningFailed { .. } => {
                self.metrics.total_planning_failures += 1;
            }
            _ => {}
        }

        // Add to ring buffer
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    fn update_success_rate(&mut self) {
        let total_attempted = self.metrics.total_plans_completed + self.metrics.total_plans_abandoned;
        if total_attempted > 0 {
            self.metrics.avg_plan_success_rate = 
                self.metrics.total_plans_completed as f32 / total_attempted as f32;
        }
    }

    /// Get current metrics
    pub fn metrics(&self) -> &PlanMetrics {
        &self.metrics
    }

    /// Get all recorded events
    pub fn events(&self) -> &VecDeque<TelemetryEvent> {
        &self.events
    }

    /// Clear all events (keeps metrics)
    pub fn clear_events(&mut self) {
        self.events.clear();
    }

    /// Reset everything
    pub fn reset(&mut self) {
        self.events.clear();
        self.metrics = PlanMetrics::default();
        self.current_plan_start = None;
    }

    /// Export events as JSON
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.events)
    }

    /// Print metrics summary
    pub fn print_metrics(&self) {
        println!("\n╔════════════════════════════════════════╗");
        println!("║       GOAP Telemetry Metrics           ║");
        println!("╚════════════════════════════════════════╝\n");

        println!("📊 Plan Generation:");
        println!("   • Total plans: {}", self.metrics.total_plans_generated);
        println!("   • Completed: {}", self.metrics.total_plans_completed);
        println!("   • Abandoned: {}", self.metrics.total_plans_abandoned);
        println!("   • Failures: {}", self.metrics.total_planning_failures);
        println!("   • Success rate: {:.1}%", self.metrics.avg_plan_success_rate * 100.0);

        println!("\n⏱️  Performance:");
        println!("   • Avg planning time: {:.2}ms", self.metrics.avg_planning_time_ms);
        println!("   • Fastest: {:.2}ms", self.metrics.fastest_plan_ms);
        println!("   • Slowest: {:.2}ms", self.metrics.slowest_plan_ms);

        println!("\n📈 Plan Characteristics:");
        println!("   • Avg steps: {:.1}", self.metrics.avg_steps_per_plan);

        println!("\n════════════════════════════════════════\n");
    }
}

impl Default for TelemetryCollector {
    fn default() -> Self {
        Self::new(1000) // Default: keep last 1000 events
    }
}

/// Helper to track execution of a single plan
pub struct PlanExecutionTracker {
    plan_id: String,
    start_time: Instant,
    steps_executed: usize,
    steps_failed: usize,
}

impl PlanExecutionTracker {
    pub fn new(plan_id: String) -> Self {
        Self {
            plan_id,
            start_time: Instant::now(),
            steps_executed: 0,
            steps_failed: 0,
        }
    }

    pub fn record_step(&mut self, action_name: String, success: bool, duration: Duration) -> TelemetryEvent {
        if success {
            self.steps_executed += 1;
        } else {
            self.steps_failed += 1;
        }

        TelemetryEvent::StepExecuted {
            plan_id: self.plan_id.clone(),
            action_name,
            success,
            duration_ms: duration.as_secs_f64() * 1000.0,
        }
    }

    pub fn complete(self) -> TelemetryEvent {
        let total_duration_ms = self.start_time.elapsed().as_secs_f64() * 1000.0;
        TelemetryEvent::PlanCompleted {
            plan_id: self.plan_id,
            total_duration_ms,
            steps_executed: self.steps_executed,
            steps_failed: self.steps_failed,
        }
    }

    pub fn abandon(self, reason: String) -> TelemetryEvent {
        TelemetryEvent::PlanAbandoned {
            plan_id: self.plan_id,
            reason,
            steps_completed: self.steps_executed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_collector() {
        let mut collector = TelemetryCollector::new(10);

        collector.record(TelemetryEvent::PlanGenerated {
            plan_id: "plan-1".to_string(),
            timestamp: 1.0,
            step_count: 3,
            planning_time_ms: 2.5,
            action_names: vec!["move".to_string(), "attack".to_string()],
        });

        collector.record(TelemetryEvent::PlanCompleted {
            plan_id: "plan-1".to_string(),
            total_duration_ms: 100.0,
            steps_executed: 3,
            steps_failed: 0,
        });

        assert_eq!(collector.metrics().total_plans_generated, 1);
        assert_eq!(collector.metrics().total_plans_completed, 1);
        assert_eq!(collector.metrics().avg_plan_success_rate, 1.0);
    }

    #[test]
    fn test_ring_buffer() {
        let mut collector = TelemetryCollector::new(3);

        for i in 0..5 {
            collector.record(TelemetryEvent::PlanGenerated {
                plan_id: format!("plan-{}", i),
                timestamp: i as f32,
                step_count: 1,
                planning_time_ms: 1.0,
                action_names: vec![],
            });
        }

        // Should only have last 3 events
        assert_eq!(collector.events().len(), 3);
        // But total count should be 5
        assert_eq!(collector.metrics().total_plans_generated, 5);
    }

    #[test]
    fn test_plan_execution_tracker() {
        let mut tracker = PlanExecutionTracker::new("test-plan".to_string());

        let event = tracker.record_step("move".to_string(), true, Duration::from_millis(50));
        
        match event {
            TelemetryEvent::StepExecuted { success, .. } => {
                assert!(success);
            }
            _ => panic!("Wrong event type"),
        }

        let complete_event = tracker.complete();
        match complete_event {
            TelemetryEvent::PlanCompleted { steps_executed, .. } => {
                assert_eq!(steps_executed, 1);
            }
            _ => panic!("Wrong event type"),
        }
    }
}

