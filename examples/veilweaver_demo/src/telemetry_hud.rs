// Telemetry HUD - Performance monitoring and metrics capture

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::fs;
use std::path::Path;
use std::time::Duration;

#[derive(Clone, Debug)]
pub struct TelemetryMetrics {
    pub fps: f32,
    pub frame_time: Duration,
    pub physics_time: Duration,
    pub ai_planning_time: Duration,
    pub memory_mb: f32,
}

impl Default for TelemetryMetrics {
    fn default() -> Self {
        Self {
            fps: 60.0,
            frame_time: Duration::from_millis(16),
            physics_time: Duration::from_millis(0),
            ai_planning_time: Duration::from_millis(0),
            memory_mb: 0.0,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MetricsSample {
    pub timestamp: f32,
    pub fps: f32,
    pub frame_time_ms: f32,
    pub physics_time_ms: f32,
    pub ai_planning_time_ms: f32,
    pub memory_mb: f32,
}

pub struct TelemetryHud {
    samples: VecDeque<MetricsSample>,
    max_samples: usize,
    start_time: std::time::Instant,
}

impl TelemetryHud {
    pub fn new() -> Self {
        Self {
            samples: VecDeque::with_capacity(1000),
            max_samples: 1000,
            start_time: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self, metrics: &TelemetryMetrics) {
        let timestamp = self.start_time.elapsed().as_secs_f32();

        let sample = MetricsSample {
            timestamp,
            fps: metrics.fps,
            frame_time_ms: metrics.frame_time.as_secs_f32() * 1000.0,
            physics_time_ms: metrics.physics_time.as_secs_f32() * 1000.0,
            ai_planning_time_ms: metrics.ai_planning_time.as_secs_f32() * 1000.0,
            memory_mb: metrics.memory_mb,
        };

        self.samples.push_back(sample);

        // Keep only recent samples
        while self.samples.len() > self.max_samples {
            self.samples.pop_front();
        }
    }

    pub fn get_stats(&self) -> TelemetryStats {
        if self.samples.is_empty() {
            return TelemetryStats::default();
        }

        let fps_values: Vec<f32> = self.samples.iter().map(|s| s.fps).collect();
        let frame_time_values: Vec<f32> = self.samples.iter().map(|s| s.frame_time_ms).collect();

        TelemetryStats {
            fps_avg: avg(&fps_values),
            fps_min: min(&fps_values),
            fps_p50: percentile(&fps_values, 0.5),
            fps_p95: percentile(&fps_values, 0.95),
            fps_p99: percentile(&fps_values, 0.99),
            frame_time_avg: avg(&frame_time_values),
            frame_time_max: max(&frame_time_values),
            frame_time_p95: percentile(&frame_time_values, 0.95),
            sample_count: self.samples.len(),
        }
    }

    pub fn export_to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let export = TelemetryExport {
            stats: self.get_stats(),
            samples: self.samples.iter().cloned().collect(),
        };

        let json = serde_json::to_string_pretty(&export)?;
        fs::write(path, json)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct TelemetryStats {
    pub fps_avg: f32,
    pub fps_min: f32,
    pub fps_p50: f32,
    pub fps_p95: f32,
    pub fps_p99: f32,
    pub frame_time_avg: f32,
    pub frame_time_max: f32,
    pub frame_time_p95: f32,
    pub sample_count: usize,
}

#[derive(Serialize, Deserialize)]
struct TelemetryExport {
    stats: TelemetryStats,
    samples: Vec<MetricsSample>,
}

// Statistical helper functions
fn avg(values: &[f32]) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    values.iter().sum::<f32>() / values.len() as f32
}

fn min(values: &[f32]) -> f32 {
    values.iter().fold(f32::INFINITY, |a, &b| a.min(b))
}

fn max(values: &[f32]) -> f32 {
    values.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b))
}

fn percentile(values: &[f32], p: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }

    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    let idx = ((sorted.len() - 1) as f32 * p) as usize;
    sorted[idx.min(sorted.len() - 1)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_hud_basic() {
        let mut hud = TelemetryHud::new();

        let metrics = TelemetryMetrics {
            fps: 60.0,
            frame_time: Duration::from_millis(16),
            physics_time: Duration::from_millis(2),
            ai_planning_time: Duration::from_millis(1),
            memory_mb: 100.0,
        };

        hud.update(&metrics);

        let stats = hud.get_stats();
        assert_eq!(stats.sample_count, 1);
        assert_eq!(stats.fps_avg, 60.0);
    }

    #[test]
    fn test_stats_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        assert_eq!(avg(&values), 30.0);
        assert_eq!(min(&values), 10.0);
        assert_eq!(max(&values), 50.0);
        assert_eq!(percentile(&values, 0.5), 30.0);
    }
}
