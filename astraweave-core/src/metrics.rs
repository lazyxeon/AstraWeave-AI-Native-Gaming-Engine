use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

/// Centralized metrics registry for tracking system performance and reliability.
///
/// Provides a thread-safe singleton-like access pattern (via lazy_static or just shared Arc)
/// for recording counters, gauges, and histograms.
#[derive(Debug, Clone, Default)]
pub struct MetricsRegistry {
    counters: Arc<Mutex<HashMap<String, u64>>>,
    gauges: Arc<Mutex<HashMap<String, f64>>>,
    histograms: Arc<Mutex<HashMap<String, Vec<f64>>>>,
}

impl MetricsRegistry {
    pub fn new() -> Self {
        Self {
            counters: Arc::new(Mutex::new(HashMap::new())),
            gauges: Arc::new(Mutex::new(HashMap::new())),
            histograms: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Increment a counter by 1.
    pub fn increment(&self, key: &str) {
        self.add(key, 1);
    }

    /// Add a value to a counter.
    pub fn add(&self, key: &str, value: u64) {
        let mut counters = self.counters.lock().unwrap();
        *counters.entry(key.to_string()).or_insert(0) += value;
    }

    /// Set a gauge value.
    pub fn gauge(&self, key: &str, value: f64) {
        let mut gauges = self.gauges.lock().unwrap();
        gauges.insert(key.to_string(), value);
    }

    /// Record a value in a histogram.
    pub fn histogram(&self, key: &str, value: f64) {
        let mut histograms = self.histograms.lock().unwrap();
        histograms
            .entry(key.to_string())
            .or_default()
            .push(value);
    }

    /// Record a duration in milliseconds.
    pub fn record_duration(&self, key: &str, duration: Duration) {
        self.histogram(key, duration.as_secs_f64() * 1000.0);
    }

    /// Get a snapshot of all counters.
    pub fn get_counters(&self) -> HashMap<String, u64> {
        self.counters.lock().unwrap().clone()
    }

    /// Get a snapshot of all gauges.
    pub fn get_gauges(&self) -> HashMap<String, f64> {
        self.gauges.lock().unwrap().clone()
    }

    /// Get summary statistics for a histogram.
    /// Returns (count, min, max, avg).
    pub fn get_histogram_stats(&self, key: &str) -> Option<(usize, f64, f64, f64)> {
        let histograms = self.histograms.lock().unwrap();
        let values = histograms.get(key)?;

        if values.is_empty() {
            return None;
        }

        let count = values.len();
        let min = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let max = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let sum: f64 = values.iter().sum();
        let avg = sum / count as f64;

        Some((count, min, max, avg))
    }
}

// Global singleton instance
use std::sync::OnceLock;

static GLOBAL_REGISTRY: OnceLock<MetricsRegistry> = OnceLock::new();

/// Get the global metrics registry.
pub fn global_metrics() -> &'static MetricsRegistry {
    GLOBAL_REGISTRY.get_or_init(MetricsRegistry::new)
}

/// Helper: Increment a global counter.
pub fn increment(key: &str) {
    global_metrics().increment(key);
}

/// Helper: Set a global gauge.
pub fn gauge(key: &str, value: f64) {
    global_metrics().gauge(key, value);
}

/// Helper: Record a global histogram value.
pub fn histogram(key: &str, value: f64) {
    global_metrics().histogram(key, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_counters() {
        let metrics = MetricsRegistry::new();
        metrics.increment("test.counter");
        metrics.add("test.counter", 5);

        let counters = metrics.get_counters();
        assert_eq!(counters.get("test.counter"), Some(&6));
    }

    #[test]
    fn test_gauges() {
        let metrics = MetricsRegistry::new();
        metrics.gauge("test.gauge", 42.5);

        let gauges = metrics.get_gauges();
        assert_eq!(gauges.get("test.gauge"), Some(&42.5));
    }

    #[test]
    fn test_histograms() {
        let metrics = MetricsRegistry::new();
        metrics.histogram("test.hist", 10.0);
        metrics.histogram("test.hist", 20.0);
        metrics.histogram("test.hist", 30.0);

        let (count, min, max, avg) = metrics.get_histogram_stats("test.hist").unwrap();
        assert_eq!(count, 3);
        assert_eq!(min, 10.0);
        assert_eq!(max, 30.0);
        assert_eq!(avg, 20.0);
    }

    // ===== Additional Coverage Tests =====

    #[test]
    fn test_record_duration() {
        let metrics = MetricsRegistry::new();
        metrics.record_duration("test.duration", Duration::from_millis(100));
        metrics.record_duration("test.duration", Duration::from_millis(200));

        let (count, min, max, avg) = metrics.get_histogram_stats("test.duration").unwrap();
        assert_eq!(count, 2);
        assert!((min - 100.0).abs() < 0.1);
        assert!((max - 200.0).abs() < 0.1);
        assert!((avg - 150.0).abs() < 0.1);
    }

    #[test]
    fn test_histogram_stats_nonexistent() {
        let metrics = MetricsRegistry::new();
        let stats = metrics.get_histogram_stats("nonexistent");
        assert!(stats.is_none());
    }

    #[test]
    fn test_histogram_stats_empty() {
        let metrics = MetricsRegistry::new();
        // Force create empty histogram
        {
            let mut histograms = metrics.histograms.lock().unwrap();
            histograms.insert("empty.hist".to_string(), Vec::new());
        }
        let stats = metrics.get_histogram_stats("empty.hist");
        assert!(stats.is_none());
    }

    #[test]
    fn test_gauge_overwrite() {
        let metrics = MetricsRegistry::new();
        metrics.gauge("cpu", 50.0);
        metrics.gauge("cpu", 75.0);

        let gauges = metrics.get_gauges();
        assert_eq!(gauges.get("cpu"), Some(&75.0));
    }

    #[test]
    fn test_multiple_counters() {
        let metrics = MetricsRegistry::new();
        metrics.increment("a");
        metrics.increment("b");
        metrics.add("a", 10);

        let counters = metrics.get_counters();
        assert_eq!(counters.get("a"), Some(&11));
        assert_eq!(counters.get("b"), Some(&1));
    }

    #[test]
    fn test_global_metrics_increment() {
        // Note: Uses global state, but tests should still run
        increment("global.test.counter");
        let counters = global_metrics().get_counters();
        assert!(counters.contains_key("global.test.counter"));
    }

    #[test]
    fn test_global_metrics_gauge() {
        gauge("global.test.gauge", 123.456);
        let gauges = global_metrics().get_gauges();
        assert_eq!(gauges.get("global.test.gauge"), Some(&123.456));
    }

    #[test]
    fn test_global_metrics_histogram() {
        histogram("global.test.hist", 1.0);
        histogram("global.test.hist", 2.0);
        let stats = global_metrics().get_histogram_stats("global.test.hist");
        assert!(stats.is_some());
    }

    #[test]
    fn test_metrics_registry_clone() {
        let metrics = MetricsRegistry::new();
        metrics.increment("cloned.counter");
        
        let cloned = metrics.clone();
        cloned.increment("cloned.counter");
        
        // Both share the same Arc'd data
        let counters = metrics.get_counters();
        assert_eq!(counters.get("cloned.counter"), Some(&2));
    }

    #[test]
    fn test_metrics_registry_default() {
        let metrics = MetricsRegistry::default();
        assert!(metrics.get_counters().is_empty());
        assert!(metrics.get_gauges().is_empty());
    }

    #[test]
    fn test_histogram_single_value() {
        let metrics = MetricsRegistry::new();
        metrics.histogram("single", 42.0);

        let (count, min, max, avg) = metrics.get_histogram_stats("single").unwrap();
        assert_eq!(count, 1);
        assert_eq!(min, 42.0);
        assert_eq!(max, 42.0);
        assert_eq!(avg, 42.0);
    }

    #[test]
    fn test_record_duration_microseconds() {
        let metrics = MetricsRegistry::new();
        metrics.record_duration("micro", Duration::from_micros(500));

        let (count, _, _, avg) = metrics.get_histogram_stats("micro").unwrap();
        assert_eq!(count, 1);
        assert!((avg - 0.5).abs() < 0.01); // 500µs = 0.5ms
    }
}
