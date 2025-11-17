// Learning algorithms for adaptive success probability estimation
// Phase 3: Learning & Persistence

use super::{ActionHistory, ActionStats};
use super::config::{GOAPConfig, SmoothingMethod};

/// EWMA (Exponentially Weighted Moving Average) smoothing for success probability
pub struct EWMASmoothing {
    alpha: f32, // Smoothing factor (0.0-1.0, higher = more weight on recent data)
}

impl EWMASmoothing {
    pub fn new(alpha: f32) -> Self {
        Self {
            alpha: alpha.clamp(0.0, 1.0),
        }
    }

    /// Calculate EWMA-smoothed success probability
    pub fn estimate(&self, stats: &ActionStats, previous_estimate: Option<f32>) -> f32 {
        let raw_rate = stats.success_rate();
        
        match previous_estimate {
            Some(prev) => {
                // EWMA formula: new_estimate = alpha * raw_rate + (1 - alpha) * previous_estimate
                self.alpha * raw_rate + (1.0 - self.alpha) * prev
            }
            None => {
                // No previous estimate, use raw rate
                raw_rate
            }
        }
    }
}

/// Bayesian estimation with prior knowledge
pub struct BayesianSmoothing {
    prior_successes: u32,
    prior_failures: u32,
}

impl BayesianSmoothing {
    pub fn new(prior_successes: u32, prior_failures: u32) -> Self {
        Self {
            prior_successes,
            prior_failures,
        }
    }

    /// Calculate Bayesian posterior success probability
    /// Uses Beta distribution: P(success) = (successes + prior_successes) / (total + prior_total)
    pub fn estimate(&self, stats: &ActionStats) -> f32 {
        let posterior_successes = stats.successes + self.prior_successes;
        let posterior_failures = stats.failures + self.prior_failures;
        let posterior_total = posterior_successes + posterior_failures;

        if posterior_total == 0 {
            // Should never happen with proper priors, but handle gracefully
            return 0.5;
        }

        posterior_successes as f32 / posterior_total as f32
    }

    /// Calculate confidence interval width (95% confidence)
    /// Smaller interval = more confidence in estimate
    pub fn confidence_interval_width(&self, stats: &ActionStats) -> f32 {
        let posterior_successes = stats.successes + self.prior_successes;
        let posterior_failures = stats.failures + self.prior_failures;
        let n = posterior_successes + posterior_failures;

        if n == 0 {
            return 1.0; // Maximum uncertainty
        }

        let p = posterior_successes as f32 / n as f32;
        let variance = p * (1.0 - p) / n as f32;
        
        // 95% CI is approximately ± 1.96 * sqrt(variance)
        2.0 * 1.96 * variance.sqrt()
    }
}

/// Enhanced ActionStats with smoothed estimates
pub struct SmoothedStats {
    pub raw_stats: ActionStats,
    pub smoothed_probability: f32,
    pub confidence: f32, // 0.0 = no confidence, 1.0 = full confidence
}

impl SmoothedStats {
    /// Create from raw stats using config
    pub fn from_stats(stats: ActionStats, config: &GOAPConfig, previous_estimate: Option<f32>) -> Self {
        let (smoothed_probability, confidence) = match config.learning.smoothing.method {
            SmoothingMethod::Ewma => {
                let ewma = EWMASmoothing::new(config.learning.smoothing.ewma_alpha);
                let prob = ewma.estimate(&stats, previous_estimate);
                
                // Confidence based on sample size
                let conf = (stats.executions as f32 / 20.0).min(1.0);
                (prob, conf)
            }
            SmoothingMethod::Bayesian => {
                let bayesian = BayesianSmoothing::new(
                    config.learning.smoothing.bayesian_prior_successes,
                    config.learning.smoothing.bayesian_prior_failures,
                );
                let prob = bayesian.estimate(&stats);
                
                // Confidence based on interval width (narrower = more confident)
                let interval_width = bayesian.confidence_interval_width(&stats);
                let conf = (1.0 - interval_width).max(0.0);
                (prob, conf)
            }
        };

        // Clamp to configured bounds
        let clamped_prob = smoothed_probability
            .max(config.learning.min_success_rate)
            .min(config.learning.max_success_rate);

        Self {
            raw_stats: stats,
            smoothed_probability: clamped_prob,
            confidence,
        }
    }
}

/// Learning manager that tracks smoothed estimates
pub struct LearningManager {
    config: GOAPConfig,
    /// Previous EWMA estimates per action (for EWMA continuity)
    ewma_estimates: std::collections::HashMap<String, f32>,
}

impl LearningManager {
    pub fn new(config: GOAPConfig) -> Self {
        Self {
            config,
            ewma_estimates: std::collections::HashMap::new(),
        }
    }

    /// Get smoothed probability for an action
    pub fn get_probability(&mut self, action_name: &str, history: &ActionHistory) -> f32 {
        if !self.config.learning.enabled {
            return self.config.learning.initial_success_rate;
        }

        match history.get_action_stats(action_name) {
            Some(stats) => {
                let previous_estimate = self.ewma_estimates.get(action_name).copied();
                let smoothed = SmoothedStats::from_stats(stats.clone(), &self.config, previous_estimate);
                
                // Update EWMA estimate for next time
                if self.config.learning.smoothing.method == SmoothingMethod::Ewma {
                    self.ewma_estimates.insert(action_name.to_string(), smoothed.smoothed_probability);
                }
                
                smoothed.smoothed_probability
            }
            None => {
                // No history for this action, use initial rate
                self.config.learning.initial_success_rate
            }
        }
    }

    /// Get detailed smoothed stats for an action
    pub fn get_smoothed_stats(&self, action_name: &str, history: &ActionHistory) -> Option<SmoothedStats> {
        if !self.config.learning.enabled {
            return None;
        }

        history.get_action_stats(action_name).map(|stats| {
            let previous_estimate = self.ewma_estimates.get(action_name).copied();
            SmoothedStats::from_stats(stats.clone(), &self.config, previous_estimate)
        })
    }

    /// Update configuration (useful for hot-reload)
    pub fn update_config(&mut self, config: GOAPConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_stats(successes: u32, failures: u32) -> ActionStats {
        ActionStats {
            executions: successes + failures,
            successes,
            failures,
            avg_duration: 0.1,
        }
    }

    #[test]
    fn test_ewma_smoothing() {
        let ewma = EWMASmoothing::new(0.2);
        
        // First estimate (no previous)
        let stats1 = create_test_stats(8, 2); // 80% success
        let est1 = ewma.estimate(&stats1, None);
        assert_eq!(est1, 0.8);

        // Second estimate (with previous)
        let stats2 = create_test_stats(6, 4); // 60% success
        let est2 = ewma.estimate(&stats2, Some(est1));
        
        // Should be: 0.2 * 0.6 + 0.8 * 0.8 = 0.12 + 0.64 = 0.76
        assert!((est2 - 0.76).abs() < 0.01);
    }

    #[test]
    fn test_ewma_adapts_to_recent_changes() {
        let ewma = EWMASmoothing::new(0.5); // Higher alpha = faster adaptation
        
        let stats_good = create_test_stats(9, 1); // 90% success
        let est1 = ewma.estimate(&stats_good, None);
        assert_eq!(est1, 0.9);

        let stats_bad = create_test_stats(2, 8); // 20% success (performance dropped)
        let est2 = ewma.estimate(&stats_bad, Some(est1));
        
        // Should adapt quickly: 0.5 * 0.2 + 0.5 * 0.9 = 0.55
        assert!((est2 - 0.55).abs() < 0.01);
    }

    #[test]
    fn test_bayesian_smoothing() {
        let bayesian = BayesianSmoothing::new(3, 1); // Prior: 75% success
        
        // Sparse data should be influenced by prior
        let sparse_stats = create_test_stats(1, 0); // 100% success, but only 1 sample
        let est_sparse = bayesian.estimate(&sparse_stats);
        
        // Posterior: (1+3) / (1+0+3+1) = 4/5 = 0.8
        assert!((est_sparse - 0.8).abs() < 0.01);
        
        // Lots of data should dominate prior
        let rich_stats = create_test_stats(50, 50); // 50% success, 100 samples
        let est_rich = bayesian.estimate(&rich_stats);
        
        // Posterior: (50+3) / (100+4) ≈ 0.51
        assert!((est_rich - 0.51).abs() < 0.02);
    }

    #[test]
    fn test_bayesian_confidence_interval() {
        let bayesian = BayesianSmoothing::new(3, 1);
        
        // Sparse data = wide interval (low confidence)
        let sparse = create_test_stats(2, 1);
        let ci_sparse = bayesian.confidence_interval_width(&sparse);
        assert!(ci_sparse > 0.3, "Sparse data should have wide CI");
        
        // Rich data = narrow interval (high confidence)
        let rich = create_test_stats(50, 50);
        let ci_rich = bayesian.confidence_interval_width(&rich);
        assert!(ci_rich < 0.2, "Rich data should have narrow CI");
    }

    #[test]
    fn test_smoothed_stats_with_ewma_config() {
        let mut config = GOAPConfig::default();
        config.learning.smoothing.method = SmoothingMethod::Ewma;
        config.learning.smoothing.ewma_alpha = 0.3;
        
        let stats = create_test_stats(7, 3); // 70% success
        let smoothed = SmoothedStats::from_stats(stats, &config, Some(0.5));
        
        // EWMA: 0.3 * 0.7 + 0.7 * 0.5 = 0.21 + 0.35 = 0.56
        assert!((smoothed.smoothed_probability - 0.56).abs() < 0.01);
        assert!(smoothed.confidence > 0.0);
    }

    #[test]
    fn test_smoothed_stats_with_bayesian_config() {
        let mut config = GOAPConfig::default();
        config.learning.smoothing.method = SmoothingMethod::Bayesian;
        config.learning.smoothing.bayesian_prior_successes = 5;
        config.learning.smoothing.bayesian_prior_failures = 5;
        
        let stats = create_test_stats(10, 0); // 100% success
        let smoothed = SmoothedStats::from_stats(stats, &config, None);
        
        // Posterior: (10+5) / (10+0+5+5) = 15/20 = 0.75
        assert!((smoothed.smoothed_probability - 0.75).abs() < 0.01);
    }

    #[test]
    fn test_probability_bounds_clamping() {
        let mut config = GOAPConfig::default();
        config.learning.min_success_rate = 0.2;
        config.learning.max_success_rate = 0.9;
        
        // Very low success rate should be clamped to min
        let low_stats = create_test_stats(1, 99); // 1% success
        let smoothed_low = SmoothedStats::from_stats(low_stats, &config, None);
        assert!(smoothed_low.smoothed_probability >= 0.2);
        
        // Very high success rate should be clamped to max
        let high_stats = create_test_stats(99, 1); // 99% success
        let smoothed_high = SmoothedStats::from_stats(high_stats, &config, None);
        assert!(smoothed_high.smoothed_probability <= 0.9);
    }

    #[test]
    fn test_learning_manager() {
        let config = GOAPConfig::default();
        let mut manager = LearningManager::new(config);
        
        let mut history = ActionHistory::new();
        history.record_success("attack", 0.1);
        history.record_success("attack", 0.1);
        history.record_failure("attack");
        
        // Should return smoothed probability
        let prob = manager.get_probability("attack", &history);
        assert!(prob > 0.0 && prob < 1.0);
        
        // Unknown action should return initial rate
        let unknown_prob = manager.get_probability("unknown_action", &history);
        assert_eq!(unknown_prob, 0.75); // default initial rate
    }

    #[test]
    fn test_learning_disabled() {
        let mut config = GOAPConfig::default();
        config.learning.enabled = false;
        
        let mut manager = LearningManager::new(config.clone());
        
        let mut history = ActionHistory::new();
        history.record_success("attack", 0.1);
        history.record_failure("attack");
        
        // Should always return initial rate when disabled
        let prob = manager.get_probability("attack", &history);
        assert_eq!(prob, config.learning.initial_success_rate);
    }
}

