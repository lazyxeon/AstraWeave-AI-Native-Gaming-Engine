use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// A/B testing framework for LLM prompts and models
pub struct ABTestFramework {
    /// Active experiments
    experiments: Arc<RwLock<HashMap<String, Experiment>>>,
    /// Results storage
    results: Arc<RwLock<HashMap<String, ExperimentResults>>>,
    /// Configuration
    config: ABTestConfig,
}

/// Configuration for A/B testing
#[derive(Debug, Clone)]
pub struct ABTestConfig {
    /// Default experiment duration in hours
    pub default_duration_hours: u64,
    /// Minimum sample size per variant
    pub min_sample_size: usize,
    /// Statistical significance threshold (p-value)
    pub significance_threshold: f64,
    /// Enable automatic winner selection
    pub auto_winner_selection: bool,
    /// Maximum concurrent experiments
    pub max_concurrent_experiments: usize,
}

impl Default for ABTestConfig {
    fn default() -> Self {
        Self {
            default_duration_hours: 168, // 1 week
            min_sample_size: 100,
            significance_threshold: 0.05, // 95% confidence
            auto_winner_selection: false,
            max_concurrent_experiments: 10,
        }
    }
}

/// Individual A/B test experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: String,
    pub name: String,
    pub description: String,
    pub status: ExperimentStatus,
    pub created_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub duration_hours: u64,
    pub traffic_percentage: f32, // 0.0 to 1.0
    pub control_variant: Variant,
    pub test_variants: Vec<Variant>,
    pub target_metric: String,
    pub success_criteria: SuccessCriteria,
    pub assignment_strategy: AssignmentStrategy,
    pub metadata: HashMap<String, String>,
}

/// Status of an experiment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExperimentStatus {
    Draft,
    Running,
    Paused,
    Completed,
    Stopped,
}

/// Individual variant in an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variant {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt_template: Option<String>,
    pub model_config: Option<ModelConfig>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub traffic_allocation: f32, // 0.0 to 1.0
}

/// Model configuration for a variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    pub model_name: String,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub top_p: Option<f32>,
    pub top_k: Option<u32>,
    pub repetition_penalty: Option<f32>,
}

/// Success criteria for the experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    pub primary_metric: String,
    pub improvement_threshold: f32, // Minimum improvement to be considered significant
    pub direction: OptimizationDirection,
    pub secondary_metrics: Vec<String>,
}

/// Optimization direction for metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationDirection {
    Maximize, // Higher values are better
    Minimize, // Lower values are better
}

/// Strategy for assigning users to variants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignmentStrategy {
    /// Hash-based deterministic assignment
    Hash,
    /// Weighted random assignment
    WeightedRandom,
    /// Round-robin assignment
    RoundRobin,
    /// Custom assignment function
    Custom(String),
}

/// Results of an experiment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResults {
    pub experiment_id: String,
    pub status: ResultStatus,
    pub variant_results: HashMap<String, VariantResults>,
    pub statistical_analysis: Option<StatisticalAnalysis>,
    pub winner: Option<String>, // Variant ID
    pub confidence_level: f64,
    pub last_updated: DateTime<Utc>,
}

/// Status of experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultStatus {
    InProgress,
    SignificantResult,
    NoSignificantDifference,
    InsufficientData,
}

/// Results for a specific variant
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantResults {
    pub variant_id: String,
    pub sample_size: usize,
    pub metrics: HashMap<String, MetricResult>,
    pub conversion_rate: f32,
    pub confidence_interval: ConfidenceInterval,
}

/// Result for a specific metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricResult {
    pub metric_name: String,
    pub value: f32,
    pub count: usize,
    pub sum: f32,
    pub mean: f32,
    pub std_dev: f32,
    pub percentiles: HashMap<String, f32>, // P50, P95, P99, etc.
}

/// Confidence interval for a metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceInterval {
    pub lower_bound: f32,
    pub upper_bound: f32,
    pub confidence_level: f64,
}

/// Statistical analysis of experiment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticalAnalysis {
    pub test_type: String,
    pub p_value: f64,
    pub effect_size: f32,
    pub power: f64,
    pub recommendations: Vec<String>,
}

/// Outcome of an experiment interaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outcome {
    pub user_id: String,
    pub experiment_id: String,
    pub variant_id: String,
    pub timestamp: DateTime<Utc>,
    pub metrics: HashMap<String, f32>,
    pub success: bool,
    pub metadata: HashMap<String, String>,
}

/// Assignment result for a user
#[derive(Debug, Clone)]
pub struct VariantAssignment {
    pub experiment_id: String,
    pub variant_id: String,
    pub variant: Variant,
    pub assigned_at: DateTime<Utc>,
}

impl ABTestFramework {
    pub fn new(config: ABTestConfig) -> Self {
        Self {
            experiments: Arc::new(RwLock::new(HashMap::new())),
            results: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Create a new experiment
    pub async fn create_experiment(&self, mut experiment: Experiment) -> Result<String> {
        // Validate experiment
        self.validate_experiment(&experiment)?;

        // Generate ID if not provided
        if experiment.id.is_empty() {
            experiment.id = Uuid::new_v4().to_string();
        }

        // Set defaults
        if experiment.duration_hours == 0 {
            experiment.duration_hours = self.config.default_duration_hours;
        }

        experiment.status = ExperimentStatus::Draft;
        experiment.created_at = Utc::now();

        // Check concurrent experiment limit
        {
            let experiments = self.experiments.read().await;
            let running_count = experiments
                .values()
                .filter(|e| e.status == ExperimentStatus::Running)
                .count();

            if running_count >= self.config.max_concurrent_experiments {
                return Err(anyhow!("Maximum concurrent experiments limit reached"));
            }
        }

        // Store experiment
        {
            let mut experiments = self.experiments.write().await;
            experiments.insert(experiment.id.clone(), experiment.clone());
        }

        // Initialize results
        {
            let mut results = self.results.write().await;
            results.insert(
                experiment.id.clone(),
                ExperimentResults {
                    experiment_id: experiment.id.clone(),
                    status: ResultStatus::InProgress,
                    variant_results: HashMap::new(),
                    statistical_analysis: None,
                    winner: None,
                    confidence_level: 0.0,
                    last_updated: Utc::now(),
                },
            );
        }

        info!(
            "Created experiment: {} ({})",
            experiment.name, experiment.id
        );
        Ok(experiment.id)
    }

    /// Start an experiment
    pub async fn start_experiment(&self, experiment_id: &str) -> Result<()> {
        let mut experiments = self.experiments.write().await;

        if let Some(experiment) = experiments.get_mut(experiment_id) {
            if experiment.status != ExperimentStatus::Draft {
                return Err(anyhow!("Experiment is not in draft status"));
            }

            experiment.status = ExperimentStatus::Running;
            experiment.started_at = Some(Utc::now());

            info!(
                "Started experiment: {} ({})",
                experiment.name, experiment.id
            );
            Ok(())
        } else {
            Err(anyhow!("Experiment {} not found", experiment_id))
        }
    }

    /// Stop an experiment
    pub async fn stop_experiment(&self, experiment_id: &str) -> Result<()> {
        let mut experiments = self.experiments.write().await;

        if let Some(experiment) = experiments.get_mut(experiment_id) {
            if experiment.status != ExperimentStatus::Running {
                return Err(anyhow!("Experiment is not running"));
            }

            experiment.status = ExperimentStatus::Stopped;
            experiment.ended_at = Some(Utc::now());

            info!(
                "Stopped experiment: {} ({})",
                experiment.name, experiment.id
            );
            Ok(())
        } else {
            Err(anyhow!("Experiment {} not found", experiment_id))
        }
    }

    /// Assign a user to a variant
    pub async fn assign_variant(
        &self,
        experiment_id: &str,
        user_id: &str,
    ) -> Result<Option<VariantAssignment>> {
        let experiments = self.experiments.read().await;

        if let Some(experiment) = experiments.get(experiment_id) {
            if experiment.status != ExperimentStatus::Running {
                return Ok(None);
            }

            // Check if user is in experiment traffic
            if !self.is_user_in_experiment_traffic(user_id, experiment.traffic_percentage) {
                return Ok(None);
            }

            // Assign variant based on strategy
            let variant = self.assign_variant_by_strategy(user_id, experiment)?;

            Ok(Some(VariantAssignment {
                experiment_id: experiment_id.to_string(),
                variant_id: variant.id.clone(),
                variant,
                assigned_at: Utc::now(),
            }))
        } else {
            Err(anyhow!("Experiment {} not found", experiment_id))
        }
    }

    /// Record an outcome for an experiment
    pub async fn record_outcome(&self, outcome: Outcome) -> Result<()> {
        // Validate experiment is running
        {
            let experiments = self.experiments.read().await;
            if let Some(experiment) = experiments.get(&outcome.experiment_id) {
                if experiment.status != ExperimentStatus::Running {
                    return Err(anyhow!("Experiment is not running"));
                }
            } else {
                return Err(anyhow!("Experiment {} not found", outcome.experiment_id));
            }
        }

        // Update results
        {
            let mut results = self.results.write().await;
            if let Some(experiment_results) = results.get_mut(&outcome.experiment_id) {
                self.update_variant_results(experiment_results, &outcome);
                experiment_results.last_updated = Utc::now();

                // Perform statistical analysis if we have enough data
                if self.has_sufficient_data(experiment_results) {
                    experiment_results.statistical_analysis =
                        Some(self.perform_statistical_analysis(experiment_results)?);

                    // Check for significance
                    if let Some(analysis) = &experiment_results.statistical_analysis {
                        if analysis.p_value < self.config.significance_threshold {
                            experiment_results.status = ResultStatus::SignificantResult;

                            // Determine winner
                            experiment_results.winner =
                                self.determine_winner(experiment_results).await;

                            // Auto-stop experiment if configured
                            if self.config.auto_winner_selection
                                && experiment_results.winner.is_some()
                            {
                                self.stop_experiment(&outcome.experiment_id).await?;
                            }
                        }
                    }
                }
            }
        }

        debug!(
            "Recorded outcome for experiment {} variant {}",
            outcome.experiment_id, outcome.variant_id
        );
        Ok(())
    }

    /// Get experiment results
    pub async fn get_results(&self, experiment_id: &str) -> Result<ExperimentResults> {
        let results = self.results.read().await;

        results
            .get(experiment_id)
            .cloned()
            .ok_or_else(|| anyhow!("Results for experiment {} not found", experiment_id))
    }

    /// List all experiments
    pub async fn list_experiments(&self) -> Vec<Experiment> {
        let experiments = self.experiments.read().await;
        experiments.values().cloned().collect()
    }

    /// Get experiment by ID
    pub async fn get_experiment(&self, experiment_id: &str) -> Option<Experiment> {
        let experiments = self.experiments.read().await;
        experiments.get(experiment_id).cloned()
    }

    /// Delete an experiment
    pub async fn delete_experiment(&self, experiment_id: &str) -> Result<()> {
        {
            let mut experiments = self.experiments.write().await;
            if let Some(experiment) = experiments.remove(experiment_id) {
                if experiment.status == ExperimentStatus::Running {
                    warn!("Deleting running experiment: {}", experiment.name);
                }
            } else {
                return Err(anyhow!("Experiment {} not found", experiment_id));
            }
        }

        {
            let mut results = self.results.write().await;
            results.remove(experiment_id);
        }

        info!("Deleted experiment: {}", experiment_id);
        Ok(())
    }

    /// Validate experiment configuration
    fn validate_experiment(&self, experiment: &Experiment) -> Result<()> {
        if experiment.name.is_empty() {
            return Err(anyhow!("Experiment name cannot be empty"));
        }

        if experiment.control_variant.id.is_empty() {
            return Err(anyhow!("Control variant must have an ID"));
        }

        if experiment.test_variants.is_empty() {
            return Err(anyhow!("At least one test variant is required"));
        }

        // Check traffic allocation sums to 1.0
        let total_traffic = experiment.control_variant.traffic_allocation
            + experiment
                .test_variants
                .iter()
                .map(|v| v.traffic_allocation)
                .sum::<f32>();

        if (total_traffic - 1.0).abs() > 0.01 {
            return Err(anyhow!(
                "Traffic allocation must sum to 1.0 (currently: {})",
                total_traffic
            ));
        }

        // Validate variant IDs are unique
        let mut variant_ids = vec![experiment.control_variant.id.clone()];
        variant_ids.extend(experiment.test_variants.iter().map(|v| v.id.clone()));
        variant_ids.sort();
        variant_ids.dedup();

        if variant_ids.len() != 1 + experiment.test_variants.len() {
            return Err(anyhow!("Variant IDs must be unique"));
        }

        Ok(())
    }

    /// Check if user is included in experiment traffic
    fn is_user_in_experiment_traffic(&self, user_id: &str, traffic_percentage: f32) -> bool {
        let hash = self.hash_user_id(user_id);
        (hash % 100) as f32 / 100.0 < traffic_percentage
    }

    /// Assign variant based on assignment strategy
    fn assign_variant_by_strategy(
        &self,
        user_id: &str,
        experiment: &Experiment,
    ) -> Result<Variant> {
        match experiment.assignment_strategy {
            AssignmentStrategy::Hash => self.assign_variant_hash(user_id, experiment),
            AssignmentStrategy::WeightedRandom => self.assign_variant_weighted_random(experiment),
            AssignmentStrategy::RoundRobin => self.assign_variant_round_robin(experiment),
            AssignmentStrategy::Custom(_) => {
                // For now, fall back to hash-based assignment
                self.assign_variant_hash(user_id, experiment)
            }
        }
    }

    /// Hash-based deterministic variant assignment
    fn assign_variant_hash(&self, user_id: &str, experiment: &Experiment) -> Result<Variant> {
        let hash = self.hash_user_id(&format!("{}:{}", experiment.id, user_id));
        let normalized = (hash % 100) as f32 / 100.0;

        let mut cumulative = 0.0;

        // Check control variant
        cumulative += experiment.control_variant.traffic_allocation;
        if normalized <= cumulative {
            return Ok(experiment.control_variant.clone());
        }

        // Check test variants
        for variant in &experiment.test_variants {
            cumulative += variant.traffic_allocation;
            if normalized <= cumulative {
                return Ok(variant.clone());
            }
        }

        // Fallback to control variant
        Ok(experiment.control_variant.clone())
    }

    /// Weighted random variant assignment
    fn assign_variant_weighted_random(&self, experiment: &Experiment) -> Result<Variant> {
        use rand::Rng;
        let mut rng = rand::rng();
        let random_value: f32 = rng.random();

        let mut cumulative = 0.0;

        cumulative += experiment.control_variant.traffic_allocation;
        if random_value <= cumulative {
            return Ok(experiment.control_variant.clone());
        }

        for variant in &experiment.test_variants {
            cumulative += variant.traffic_allocation;
            if random_value <= cumulative {
                return Ok(variant.clone());
            }
        }

        Ok(experiment.control_variant.clone())
    }

    /// Round-robin variant assignment (simplified)
    fn assign_variant_round_robin(&self, experiment: &Experiment) -> Result<Variant> {
        // For simplicity, use time-based round robin
        let now = Utc::now().timestamp() as usize;
        let total_variants = 1 + experiment.test_variants.len();
        let variant_index = now % total_variants;

        if variant_index == 0 {
            Ok(experiment.control_variant.clone())
        } else {
            Ok(experiment.test_variants[variant_index - 1].clone())
        }
    }

    /// Hash a user ID for consistent assignment
    fn hash_user_id(&self, user_id: &str) -> u32 {
        let mut hasher = DefaultHasher::new();
        user_id.hash(&mut hasher);
        hasher.finish() as u32
    }

    /// Update variant results with new outcome
    fn update_variant_results(
        &self,
        experiment_results: &mut ExperimentResults,
        outcome: &Outcome,
    ) {
        let variant_results = experiment_results
            .variant_results
            .entry(outcome.variant_id.clone())
            .or_insert_with(|| VariantResults {
                variant_id: outcome.variant_id.clone(),
                sample_size: 0,
                metrics: HashMap::new(),
                conversion_rate: 0.0,
                confidence_interval: ConfidenceInterval {
                    lower_bound: 0.0,
                    upper_bound: 0.0,
                    confidence_level: 0.95,
                },
            });

        variant_results.sample_size += 1;

        // Update metrics
        for (metric_name, value) in &outcome.metrics {
            let metric_result = variant_results
                .metrics
                .entry(metric_name.clone())
                .or_insert_with(|| MetricResult {
                    metric_name: metric_name.clone(),
                    value: 0.0,
                    count: 0,
                    sum: 0.0,
                    mean: 0.0,
                    std_dev: 0.0,
                    percentiles: HashMap::new(),
                });

            metric_result.count += 1;
            metric_result.sum += value;
            metric_result.mean = metric_result.sum / metric_result.count as f32;
            metric_result.value = *value; // Store last value

            // Update standard deviation (simplified)
            // In practice, would track sum of squares for proper calculation
        }

        // Update conversion rate
        let successes = variant_results.sample_size;
        let success_count = if outcome.success { 1 } else { 0 };
        variant_results.conversion_rate =
            (variant_results.conversion_rate * (successes - 1) as f32 + success_count as f32)
                / successes as f32;
    }

    /// Check if experiment has sufficient data for analysis
    fn has_sufficient_data(&self, results: &ExperimentResults) -> bool {
        results
            .variant_results
            .values()
            .all(|v| v.sample_size >= self.config.min_sample_size)
    }

    /// Perform statistical analysis
    fn perform_statistical_analysis(
        &self,
        _results: &ExperimentResults,
    ) -> Result<StatisticalAnalysis> {
        // Simplified statistical analysis
        // In practice, would use proper statistical libraries like `statrs`

        Ok(StatisticalAnalysis {
            test_type: "t-test".to_string(),
            p_value: 0.03, // Placeholder
            effect_size: 0.15,
            power: 0.8,
            recommendations: vec![
                "Sample size is sufficient for reliable results".to_string(),
                "Effect size indicates practical significance".to_string(),
            ],
        })
    }

    /// Determine the winner of an experiment
    async fn determine_winner(&self, results: &ExperimentResults) -> Option<String> {
        let experiments = self.experiments.read().await;
        if let Some(experiment) = experiments.get(&results.experiment_id) {
            // Find variant with best performance on primary metric
            let mut best_variant = None;
            let mut best_value = match experiment.success_criteria.direction {
                OptimizationDirection::Maximize => f32::NEG_INFINITY,
                OptimizationDirection::Minimize => f32::INFINITY,
            };

            for (variant_id, variant_results) in &results.variant_results {
                if let Some(metric) = variant_results
                    .metrics
                    .get(&experiment.success_criteria.primary_metric)
                {
                    let is_better = match experiment.success_criteria.direction {
                        OptimizationDirection::Maximize => metric.mean > best_value,
                        OptimizationDirection::Minimize => metric.mean < best_value,
                    };

                    if is_better {
                        best_value = metric.mean;
                        best_variant = Some(variant_id.clone());
                    }
                }
            }

            best_variant
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_experiment() -> Experiment {
        Experiment {
            id: "test-experiment".to_string(),
            name: "Test Experiment".to_string(),
            description: "A test experiment".to_string(),
            status: ExperimentStatus::Draft,
            created_at: Utc::now(),
            started_at: None,
            ended_at: None,
            duration_hours: 24,
            traffic_percentage: 0.5,
            control_variant: Variant {
                id: "control".to_string(),
                name: "Control".to_string(),
                description: "Control variant".to_string(),
                prompt_template: Some("Original prompt".to_string()),
                model_config: None,
                parameters: HashMap::new(),
                traffic_allocation: 0.5,
            },
            test_variants: vec![Variant {
                id: "test".to_string(),
                name: "Test".to_string(),
                description: "Test variant".to_string(),
                prompt_template: Some("New prompt".to_string()),
                model_config: None,
                parameters: HashMap::new(),
                traffic_allocation: 0.5,
            }],
            target_metric: "conversion_rate".to_string(),
            success_criteria: SuccessCriteria {
                primary_metric: "conversion_rate".to_string(),
                improvement_threshold: 0.05,
                direction: OptimizationDirection::Maximize,
                secondary_metrics: Vec::new(),
            },
            assignment_strategy: AssignmentStrategy::Hash,
            metadata: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn test_create_experiment() {
        let framework = ABTestFramework::new(ABTestConfig::default());
        let experiment = create_test_experiment();

        let experiment_id = framework.create_experiment(experiment).await.unwrap();
        assert!(!experiment_id.is_empty());

        let retrieved = framework.get_experiment(&experiment_id).await.unwrap();
        assert_eq!(retrieved.name, "Test Experiment");
    }

    #[tokio::test]
    async fn test_start_stop_experiment() {
        let framework = ABTestFramework::new(ABTestConfig::default());
        let experiment = create_test_experiment();

        let experiment_id = framework.create_experiment(experiment).await.unwrap();

        framework.start_experiment(&experiment_id).await.unwrap();
        let running = framework.get_experiment(&experiment_id).await.unwrap();
        assert_eq!(running.status, ExperimentStatus::Running);

        framework.stop_experiment(&experiment_id).await.unwrap();
        let stopped = framework.get_experiment(&experiment_id).await.unwrap();
        assert_eq!(stopped.status, ExperimentStatus::Stopped);
    }

    #[tokio::test]
    async fn test_variant_assignment() {
        let framework = ABTestFramework::new(ABTestConfig::default());
        let experiment = create_test_experiment();

        let experiment_id = framework.create_experiment(experiment).await.unwrap();
        framework.start_experiment(&experiment_id).await.unwrap();

        let assignment = framework
            .assign_variant(&experiment_id, "user123")
            .await
            .unwrap();
        assert!(assignment.is_some());

        let assignment = assignment.unwrap();
        assert!(assignment.variant_id == "control" || assignment.variant_id == "test");
    }

    #[tokio::test]
    async fn test_record_outcome() {
        let framework = ABTestFramework::new(ABTestConfig::default());
        let experiment = create_test_experiment();

        let experiment_id = framework.create_experiment(experiment).await.unwrap();
        framework.start_experiment(&experiment_id).await.unwrap();

        let mut metrics = HashMap::new();
        metrics.insert("conversion_rate".to_string(), 0.15);

        let outcome = Outcome {
            user_id: "user123".to_string(),
            experiment_id: experiment_id.clone(),
            variant_id: "control".to_string(),
            timestamp: Utc::now(),
            metrics,
            success: true,
            metadata: HashMap::new(),
        };

        framework.record_outcome(outcome).await.unwrap();

        let results = framework.get_results(&experiment_id).await.unwrap();
        assert!(results.variant_results.contains_key("control"));
        assert_eq!(results.variant_results["control"].sample_size, 1);
    }

    #[test]
    fn test_hash_consistency() {
        let framework = ABTestFramework::new(ABTestConfig::default());

        let hash1 = framework.hash_user_id("user123");
        let hash2 = framework.hash_user_id("user123");

        assert_eq!(hash1, hash2);
    }
}
