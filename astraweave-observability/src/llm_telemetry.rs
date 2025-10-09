use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, Timelike, Utc};
use dashmap::DashMap;
use hdrhistogram::Histogram;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Comprehensive LLM telemetry system for production observability
pub struct LlmTelemetry {
    /// Trace storage
    traces: Arc<RwLock<VecDeque<LlmTrace>>>,
    /// Real-time metrics
    metrics: Arc<RwLock<LlmMetrics>>,
    /// Performance histograms
    histograms: Arc<RwLock<PerformanceHistograms>>,
    /// Cost tracking
    cost_tracker: Arc<RwLock<CostTracker>>,
    /// Alert system
    alert_manager: Arc<RwLock<AlertManager>>,
    /// Configuration
    config: TelemetryConfig,
    /// Active requests tracking
    active_requests: Arc<DashMap<String, ActiveRequest>>,
    /// Error tracking
    error_tracker: Arc<RwLock<ErrorTracker>>,
}

/// Configuration for telemetry system
#[derive(Debug, Clone)]
pub struct TelemetryConfig {
    /// Maximum number of traces to keep in memory
    pub max_traces: usize,
    /// Enable detailed prompt/response logging
    pub log_content: bool,
    /// Enable cost tracking
    pub enable_cost_tracking: bool,
    /// Export to Prometheus
    pub enable_prometheus: bool,
    /// Export to OpenTelemetry
    pub enable_opentelemetry: bool,
    /// Alert thresholds
    pub alert_thresholds: AlertThresholds,
    /// Sampling rate for detailed traces (0.0 to 1.0)
    pub sampling_rate: f32,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            max_traces: 10000,
            log_content: false, // Default to false for privacy
            enable_cost_tracking: true,
            enable_prometheus: false,
            enable_opentelemetry: false,
            alert_thresholds: AlertThresholds::default(),
            sampling_rate: 1.0, // Sample all requests by default
        }
    }
}

/// Alert threshold configuration
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Alert if P95 latency exceeds this (ms)
    pub latency_p95_ms: u64,
    /// Alert if error rate exceeds this (0.0 to 1.0)
    pub error_rate: f32,
    /// Alert if cost per hour exceeds this (USD)
    pub cost_per_hour_usd: f32,
    /// Alert if queue depth exceeds this
    pub queue_depth: usize,
    /// Alert if token rate exceeds this (tokens/sec)
    pub token_rate: u64,
}

impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            latency_p95_ms: 5000, // 5 seconds
            error_rate: 0.1,      // 10%
            cost_per_hour_usd: 10.0,
            queue_depth: 100,
            token_rate: 10000,
        }
    }
}

/// Individual LLM request trace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmTrace {
    pub request_id: String,
    pub session_id: Option<String>,
    pub user_id: Option<String>,
    pub prompt: Option<String>,   // Optional for privacy
    pub response: Option<String>, // Optional for privacy
    pub prompt_hash: Option<u64>, // Hash for deduplication
    pub model: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub latency_ms: u64,
    pub tokens_prompt: usize,
    pub tokens_response: usize,
    pub total_tokens: usize,
    pub cost_usd: f64,
    pub success: bool,
    pub error_message: Option<String>,
    pub error_type: Option<String>,
    pub request_source: String, // Which system made the request
    pub tags: HashMap<String, String>,
}

/// Real-time LLM metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LlmMetrics {
    pub total_requests: u64,
    pub successful_requests: u64,
    pub failed_requests: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub average_latency_ms: f32,
    pub p50_latency_ms: u64,
    pub p95_latency_ms: u64,
    pub p99_latency_ms: u64,
    pub error_rate: f32,
    pub requests_per_second: f32,
    pub tokens_per_second: f32,
    pub cost_per_hour_usd: f32,
    pub active_requests: usize,
    pub queue_depth: usize,
    pub cache_hit_rate: f32,
    pub last_updated: DateTime<Utc>,
    pub model_usage: HashMap<String, ModelMetrics>,
    pub source_metrics: HashMap<String, SourceMetrics>,
}

/// Metrics per model
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMetrics {
    pub requests: u64,
    pub total_tokens: u64,
    pub total_cost_usd: f64,
    pub average_latency_ms: f32,
    pub error_rate: f32,
}

/// Metrics per request source
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SourceMetrics {
    pub requests: u64,
    pub total_tokens: u64,
    pub average_latency_ms: f32,
    pub error_rate: f32,
}

/// Performance histograms for detailed analysis
#[derive(Debug)]
pub struct PerformanceHistograms {
    pub latency_histogram: Histogram<u64>,
    pub token_histogram: Histogram<u64>,
    pub cost_histogram: Histogram<u64>, // Cost in cents
}

impl Default for PerformanceHistograms {
    fn default() -> Self {
        Self {
            latency_histogram: Histogram::new(3).unwrap(), // 3 significant digits
            token_histogram: Histogram::new(3).unwrap(),
            cost_histogram: Histogram::new(3).unwrap(),
        }
    }
}

/// Cost tracking with budgets and alerts
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CostTracker {
    pub hourly_costs: VecDeque<HourlyCost>,
    pub daily_budget_usd: f32,
    pub monthly_budget_usd: f32,
    pub current_day_spend: f32,
    pub current_month_spend: f32,
    pub cost_by_model: HashMap<String, f32>,
    pub cost_by_source: HashMap<String, f32>,
    pub budget_alerts_sent: Vec<BudgetAlert>,
}

/// Hourly cost tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyCost {
    pub hour: DateTime<Utc>,
    pub cost_usd: f32,
    pub requests: u64,
    pub tokens: u64,
}

/// Budget alert record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAlert {
    pub alert_type: BudgetAlertType,
    pub threshold: f32,
    pub actual: f32,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetAlertType {
    DailyBudget,
    MonthlyBudget,
    HourlyRate,
}

/// Alert management system
#[derive(Debug, Default)]
pub struct AlertManager {
    pub active_alerts: HashMap<String, Alert>,
    pub alert_history: VecDeque<Alert>,
    pub notification_channels: Vec<NotificationChannel>,
}

/// Individual alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub value: f32,
    pub threshold: f32,
    pub first_triggered: DateTime<Utc>,
    pub last_triggered: DateTime<Utc>,
    pub acknowledged: bool,
    pub resolved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertType {
    HighLatency,
    HighErrorRate,
    HighCost,
    QueueBacklog,
    ModelFailure,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Notification channels for alerts
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    Log,
    Webhook(String),
    Email(String),
}

/// Currently active request tracking
#[derive(Debug, Clone)]
pub struct ActiveRequest {
    pub request_id: String,
    pub start_time: Instant,
    pub model: String,
    pub source: String,
    pub prompt_tokens: usize,
}

/// Error tracking and analysis
#[derive(Debug, Default)]
pub struct ErrorTracker {
    pub error_counts: HashMap<String, u64>,
    pub error_patterns: HashMap<String, ErrorPattern>,
    pub recent_errors: VecDeque<ErrorEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorPattern {
    pub error_type: String,
    pub frequency: u64,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub sample_messages: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub timestamp: DateTime<Utc>,
    pub error_type: String,
    pub error_message: String,
    pub request_id: String,
    pub model: String,
}

/// Dashboard data for UI
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub current_metrics: LlmMetrics,
    pub cost_summary: CostSummary,
    pub active_alerts: Vec<Alert>,
    pub top_errors: Vec<ErrorPattern>,
    pub model_breakdown: Vec<ModelBreakdown>,
    pub hourly_stats: Vec<HourlyStats>,
    pub performance_percentiles: PerformancePercentiles,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostSummary {
    pub current_hour_cost: f32,
    pub today_cost: f32,
    pub month_cost: f32,
    pub daily_budget_remaining: f32,
    pub monthly_budget_remaining: f32,
    pub projected_monthly_cost: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelBreakdown {
    pub model: String,
    pub requests: u64,
    pub cost: f32,
    pub avg_latency: f32,
    pub error_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyStats {
    pub hour: DateTime<Utc>,
    pub requests: u64,
    pub cost: f32,
    pub avg_latency: f32,
    pub error_rate: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformancePercentiles {
    pub latency_p50: u64,
    pub latency_p75: u64,
    pub latency_p90: u64,
    pub latency_p95: u64,
    pub latency_p99: u64,
    pub token_p50: u64,
    pub token_p95: u64,
    pub cost_p50: f32,
    pub cost_p95: f32,
}

impl LlmTelemetry {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            traces: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_traces))),
            metrics: Arc::new(RwLock::new(LlmMetrics::default())),
            histograms: Arc::new(RwLock::new(PerformanceHistograms::default())),
            cost_tracker: Arc::new(RwLock::new(CostTracker::default())),
            alert_manager: Arc::new(RwLock::new(AlertManager::default())),
            config,
            active_requests: Arc::new(DashMap::new()),
            error_tracker: Arc::new(RwLock::new(ErrorTracker::default())),
        }
    }

    /// Start tracking an LLM request
    pub fn start_request(
        &self,
        request_id: String,
        model: String,
        source: String,
        prompt_tokens: usize,
    ) -> RequestTracker {
        let active_request = ActiveRequest {
            request_id: request_id.clone(),
            start_time: Instant::now(),
            model: model.clone(),
            source: source.clone(),
            prompt_tokens,
        };

        self.active_requests
            .insert(request_id.clone(), active_request);

        RequestTracker {
            request_id,
            start_time: Instant::now(),
            telemetry: Arc::new(self.clone()),
        }
    }

    /// Record a completed LLM request
    pub async fn record_request(&self, trace: LlmTrace) -> Result<()> {
        // Sample based on configuration
        if self.should_sample() {
            // Store trace
            {
                let mut traces = self.traces.write().await;
                traces.push_back(trace.clone());

                // Maintain trace buffer size
                if traces.len() > self.config.max_traces {
                    traces.pop_front();
                }
            }
        }

        // Update metrics
        self.update_metrics(&trace).await?;

        // Update histograms
        self.update_histograms(&trace).await?;

        // Track costs
        if self.config.enable_cost_tracking {
            self.track_cost(&trace).await?;
        }

        // Check for alerts
        self.check_alerts().await?;

        // Remove from active requests
        self.active_requests.remove(&trace.request_id);

        // Track errors if applicable
        if !trace.success {
            self.track_error(&trace).await?;
        }

        debug!(
            "Recorded LLM request: {} ({}ms, {} tokens, ${:.4})",
            trace.request_id, trace.latency_ms, trace.total_tokens, trace.cost_usd
        );

        Ok(())
    }

    /// Get real-time dashboard data
    pub async fn get_dashboard_data(&self) -> Result<DashboardData> {
        let metrics = self.metrics.read().await.clone();
        let cost_tracker = self.cost_tracker.read().await;
        let alert_manager = self.alert_manager.read().await;
        let error_tracker = self.error_tracker.read().await;
        let histograms = self.histograms.read().await;

        let cost_summary = CostSummary {
            current_hour_cost: cost_tracker
                .hourly_costs
                .back()
                .map(|h| h.cost_usd)
                .unwrap_or(0.0),
            today_cost: cost_tracker.current_day_spend,
            month_cost: cost_tracker.current_month_spend,
            daily_budget_remaining: (cost_tracker.daily_budget_usd
                - cost_tracker.current_day_spend)
                .max(0.0),
            monthly_budget_remaining: (cost_tracker.monthly_budget_usd
                - cost_tracker.current_month_spend)
                .max(0.0),
            projected_monthly_cost: self.calculate_projected_monthly_cost(&cost_tracker),
        };

        let model_breakdown: Vec<_> = metrics
            .model_usage
            .iter()
            .map(|(model, model_metrics)| ModelBreakdown {
                model: model.clone(),
                requests: model_metrics.requests,
                cost: model_metrics.total_cost_usd as f32,
                avg_latency: model_metrics.average_latency_ms,
                error_rate: model_metrics.error_rate,
            })
            .collect();

        let hourly_stats: Vec<_> = cost_tracker
            .hourly_costs
            .iter()
            .map(|hourly| {
                HourlyStats {
                    hour: hourly.hour,
                    requests: hourly.requests,
                    cost: hourly.cost_usd,
                    avg_latency: 0.0, // Would calculate from stored data
                    error_rate: 0.0,  // Would calculate from stored data
                }
            })
            .collect();

        let performance_percentiles = PerformancePercentiles {
            latency_p50: histograms.latency_histogram.value_at_quantile(0.5),
            latency_p75: histograms.latency_histogram.value_at_quantile(0.75),
            latency_p90: histograms.latency_histogram.value_at_quantile(0.90),
            latency_p95: histograms.latency_histogram.value_at_quantile(0.95),
            latency_p99: histograms.latency_histogram.value_at_quantile(0.99),
            token_p50: histograms.token_histogram.value_at_quantile(0.5),
            token_p95: histograms.token_histogram.value_at_quantile(0.95),
            cost_p50: (histograms.cost_histogram.value_at_quantile(0.5) as f32) / 100.0,
            cost_p95: (histograms.cost_histogram.value_at_quantile(0.95) as f32) / 100.0,
        };

        Ok(DashboardData {
            current_metrics: metrics,
            cost_summary,
            active_alerts: alert_manager.active_alerts.values().cloned().collect(),
            top_errors: error_tracker.error_patterns.values().cloned().collect(),
            model_breakdown,
            hourly_stats,
            performance_percentiles,
        })
    }

    /// Export traces in various formats
    pub async fn export_traces(
        &self,
        format: ExportFormat,
        filter: Option<TraceFilter>,
    ) -> Result<String> {
        let traces = self.traces.read().await;

        let filtered_traces: Vec<_> = if let Some(filter) = filter {
            traces
                .iter()
                .filter(|trace| self.matches_filter(trace, &filter))
                .cloned()
                .collect()
        } else {
            traces.iter().cloned().collect()
        };

        match format {
            ExportFormat::Json => Ok(serde_json::to_string_pretty(&filtered_traces)?),
            ExportFormat::Csv => self.export_traces_csv(&filtered_traces),
            ExportFormat::OpenTelemetry => self.export_opentelemetry(&filtered_traces),
        }
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> LlmMetrics {
        self.metrics.read().await.clone()
    }

    /// Clear all stored data
    pub async fn clear_data(&self) -> Result<()> {
        {
            let mut traces = self.traces.write().await;
            traces.clear();
        }

        {
            let mut metrics = self.metrics.write().await;
            *metrics = LlmMetrics::default();
        }

        {
            let mut histograms = self.histograms.write().await;
            *histograms = PerformanceHistograms::default();
        }

        {
            let mut cost_tracker = self.cost_tracker.write().await;
            *cost_tracker = CostTracker::default();
        }

        {
            let mut alert_manager = self.alert_manager.write().await;
            alert_manager.active_alerts.clear();
            alert_manager.alert_history.clear();
        }

        {
            let mut error_tracker = self.error_tracker.write().await;
            *error_tracker = ErrorTracker::default();
        }

        self.active_requests.clear();

        info!("Cleared all telemetry data");
        Ok(())
    }

    /// Determine if we should sample this request
    fn should_sample(&self) -> bool {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        rng.gen::<f32>() < self.config.sampling_rate
    }

    /// Update real-time metrics
    async fn update_metrics(&self, trace: &LlmTrace) -> Result<()> {
        let mut metrics = self.metrics.write().await;

        metrics.total_requests += 1;
        if trace.success {
            metrics.successful_requests += 1;
        } else {
            metrics.failed_requests += 1;
        }

        metrics.total_tokens += trace.total_tokens as u64;
        metrics.total_cost_usd += trace.cost_usd;

        // Update averages
        let total_requests = metrics.total_requests as f32;
        metrics.average_latency_ms = (metrics.average_latency_ms * (total_requests - 1.0)
            + trace.latency_ms as f32)
            / total_requests;
        metrics.error_rate = metrics.failed_requests as f32 / metrics.total_requests as f32;

        // Update model-specific metrics
        let model_metrics = metrics
            .model_usage
            .entry(trace.model.clone())
            .or_insert_with(ModelMetrics::default);
        model_metrics.requests += 1;
        model_metrics.total_tokens += trace.total_tokens as u64;
        model_metrics.total_cost_usd += trace.cost_usd;
        model_metrics.average_latency_ms = (model_metrics.average_latency_ms
            * (model_metrics.requests - 1) as f32
            + trace.latency_ms as f32)
            / model_metrics.requests as f32;
        if !trace.success {
            model_metrics.error_rate =
                (model_metrics.error_rate * (model_metrics.requests - 1) as f32 + 1.0)
                    / model_metrics.requests as f32;
        } else {
            model_metrics.error_rate = (model_metrics.error_rate
                * (model_metrics.requests - 1) as f32)
                / model_metrics.requests as f32;
        }

        // Update source-specific metrics
        let source_metrics = metrics
            .source_metrics
            .entry(trace.request_source.clone())
            .or_insert_with(SourceMetrics::default);
        source_metrics.requests += 1;
        source_metrics.total_tokens += trace.total_tokens as u64;
        source_metrics.average_latency_ms = (source_metrics.average_latency_ms
            * (source_metrics.requests - 1) as f32
            + trace.latency_ms as f32)
            / source_metrics.requests as f32;
        if !trace.success {
            source_metrics.error_rate =
                (source_metrics.error_rate * (source_metrics.requests - 1) as f32 + 1.0)
                    / source_metrics.requests as f32;
        } else {
            source_metrics.error_rate = (source_metrics.error_rate
                * (source_metrics.requests - 1) as f32)
                / source_metrics.requests as f32;
        }

        metrics.active_requests = self.active_requests.len();
        metrics.last_updated = Utc::now();

        Ok(())
    }

    /// Update performance histograms
    async fn update_histograms(&self, trace: &LlmTrace) -> Result<()> {
        let mut histograms = self.histograms.write().await;

        histograms.latency_histogram.record(trace.latency_ms)?;
        histograms
            .token_histogram
            .record(trace.total_tokens as u64)?;
        histograms
            .cost_histogram
            .record((trace.cost_usd * 100.0) as u64)?; // Convert to cents

        Ok(())
    }

    /// Track costs and check budgets
    async fn track_cost(&self, trace: &LlmTrace) -> Result<()> {
        let mut cost_tracker = self.cost_tracker.write().await;

        // Update hourly costs
        let current_hour = trace
            .start_time
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        if let Some(hourly) = cost_tracker.hourly_costs.back_mut() {
            if hourly.hour == current_hour {
                hourly.cost_usd += trace.cost_usd as f32;
                hourly.requests += 1;
                hourly.tokens += trace.total_tokens as u64;
            } else {
                cost_tracker.hourly_costs.push_back(HourlyCost {
                    hour: current_hour,
                    cost_usd: trace.cost_usd as f32,
                    requests: 1,
                    tokens: trace.total_tokens as u64,
                });
            }
        } else {
            cost_tracker.hourly_costs.push_back(HourlyCost {
                hour: current_hour,
                cost_usd: trace.cost_usd as f32,
                requests: 1,
                tokens: trace.total_tokens as u64,
            });
        }

        // Keep only recent hourly data
        while cost_tracker.hourly_costs.len() > 168 {
            // Keep 1 week
            cost_tracker.hourly_costs.pop_front();
        }

        // Update daily/monthly spend
        cost_tracker.current_day_spend += trace.cost_usd as f32;
        cost_tracker.current_month_spend += trace.cost_usd as f32;

        // Update cost by model
        *cost_tracker
            .cost_by_model
            .entry(trace.model.clone())
            .or_insert(0.0) += trace.cost_usd as f32;

        // Update cost by source
        *cost_tracker
            .cost_by_source
            .entry(trace.request_source.clone())
            .or_insert(0.0) += trace.cost_usd as f32;

        Ok(())
    }

    /// Check for alert conditions
    async fn check_alerts(&self) -> Result<()> {
        let metrics = self.metrics.read().await;
        let cost_tracker = self.cost_tracker.read().await;
        let mut alert_manager = self.alert_manager.write().await;

        // Check latency alerts
        if metrics.p95_latency_ms > self.config.alert_thresholds.latency_p95_ms {
            self.trigger_alert(
                &mut alert_manager,
                AlertType::HighLatency,
                AlertSeverity::Warning,
                format!(
                    "P95 latency is {}ms (threshold: {}ms)",
                    metrics.p95_latency_ms, self.config.alert_thresholds.latency_p95_ms
                ),
                metrics.p95_latency_ms as f32,
                self.config.alert_thresholds.latency_p95_ms as f32,
            );
        }

        // Check error rate alerts
        if metrics.error_rate > self.config.alert_thresholds.error_rate {
            self.trigger_alert(
                &mut alert_manager,
                AlertType::HighErrorRate,
                AlertSeverity::Critical,
                format!(
                    "Error rate is {:.2}% (threshold: {:.2}%)",
                    metrics.error_rate * 100.0,
                    self.config.alert_thresholds.error_rate * 100.0
                ),
                metrics.error_rate,
                self.config.alert_thresholds.error_rate,
            );
        }

        // Check cost alerts
        if let Some(hourly) = cost_tracker.hourly_costs.back() {
            if hourly.cost_usd > self.config.alert_thresholds.cost_per_hour_usd {
                self.trigger_alert(
                    &mut alert_manager,
                    AlertType::HighCost,
                    AlertSeverity::Warning,
                    format!(
                        "Hourly cost is ${:.2} (threshold: ${:.2})",
                        hourly.cost_usd, self.config.alert_thresholds.cost_per_hour_usd
                    ),
                    hourly.cost_usd,
                    self.config.alert_thresholds.cost_per_hour_usd,
                );
            }
        }

        // Check queue depth alerts
        if metrics.queue_depth > self.config.alert_thresholds.queue_depth {
            self.trigger_alert(
                &mut alert_manager,
                AlertType::QueueBacklog,
                AlertSeverity::Warning,
                format!(
                    "Queue depth is {} (threshold: {})",
                    metrics.queue_depth, self.config.alert_thresholds.queue_depth
                ),
                metrics.queue_depth as f32,
                self.config.alert_thresholds.queue_depth as f32,
            );
        }

        Ok(())
    }

    /// Trigger an alert
    fn trigger_alert(
        &self,
        alert_manager: &mut AlertManager,
        alert_type: AlertType,
        severity: AlertSeverity,
        message: String,
        value: f32,
        threshold: f32,
    ) {
        let alert_key = format!("{:?}", alert_type);
        let now = Utc::now();

        if let Some(existing_alert) = alert_manager.active_alerts.get_mut(&alert_key) {
            existing_alert.last_triggered = now;
            existing_alert.value = value;
        } else {
            let alert = Alert {
                id: Uuid::new_v4().to_string(),
                alert_type,
                severity,
                message: message.clone(),
                value,
                threshold,
                first_triggered: now,
                last_triggered: now,
                acknowledged: false,
                resolved: false,
            };

            alert_manager.active_alerts.insert(alert_key, alert.clone());
            alert_manager.alert_history.push_back(alert);

            // Keep alert history manageable
            if alert_manager.alert_history.len() > 1000 {
                alert_manager.alert_history.pop_front();
            }

            warn!("Alert triggered: {}", message);
        }
    }

    /// Track error patterns
    async fn track_error(&self, trace: &LlmTrace) -> Result<()> {
        if let Some(error_type) = &trace.error_type {
            let mut error_tracker = self.error_tracker.write().await;

            *error_tracker
                .error_counts
                .entry(error_type.clone())
                .or_insert(0) += 1;

            let error_pattern = error_tracker
                .error_patterns
                .entry(error_type.clone())
                .or_insert_with(|| ErrorPattern {
                    error_type: error_type.clone(),
                    frequency: 0,
                    first_seen: trace.start_time,
                    last_seen: trace.start_time,
                    sample_messages: Vec::new(),
                });

            error_pattern.frequency += 1;
            error_pattern.last_seen = trace.start_time;

            if let Some(error_message) = &trace.error_message {
                if error_pattern.sample_messages.len() < 5
                    && !error_pattern.sample_messages.contains(error_message)
                {
                    error_pattern.sample_messages.push(error_message.clone());
                }
            }

            let error_event = ErrorEvent {
                timestamp: trace.start_time,
                error_type: error_type.clone(),
                error_message: trace.error_message.clone().unwrap_or_default(),
                request_id: trace.request_id.clone(),
                model: trace.model.clone(),
            };

            error_tracker.recent_errors.push_back(error_event);

            // Keep recent errors manageable
            if error_tracker.recent_errors.len() > 1000 {
                error_tracker.recent_errors.pop_front();
            }
        }

        Ok(())
    }

    /// Calculate projected monthly cost
    fn calculate_projected_monthly_cost(&self, cost_tracker: &CostTracker) -> f32 {
        let days_in_month = 30.0;
        let current_day = Utc::now().day() as f32;

        if current_day > 0.0 {
            (cost_tracker.current_month_spend / current_day) * days_in_month
        } else {
            0.0
        }
    }

    /// Export traces to CSV format
    fn export_traces_csv(&self, traces: &[LlmTrace]) -> Result<String> {
        let mut csv = String::new();
        csv.push_str("request_id,model,start_time,latency_ms,tokens_prompt,tokens_response,cost_usd,success,error_message,request_source\n");

        for trace in traces {
            csv.push_str(&format!(
                "{},{},{},{},{},{},{},{},{},{}\n",
                trace.request_id,
                trace.model,
                trace.start_time.format("%Y-%m-%d %H:%M:%S UTC"),
                trace.latency_ms,
                trace.tokens_prompt,
                trace.tokens_response,
                trace.cost_usd,
                trace.success,
                trace.error_message.as_deref().unwrap_or(""),
                trace.request_source
            ));
        }

        Ok(csv)
    }

    /// Export to OpenTelemetry format
    fn export_opentelemetry(&self, _traces: &[LlmTrace]) -> Result<String> {
        // Placeholder for OpenTelemetry export
        // Would implement actual OTLP format here
        Ok("OpenTelemetry export not yet implemented".to_string())
    }

    /// Check if trace matches filter
    fn matches_filter(&self, trace: &LlmTrace, filter: &TraceFilter) -> bool {
        if let Some(model) = &filter.model {
            if &trace.model != model {
                return false;
            }
        }

        if let Some(success) = filter.success {
            if trace.success != success {
                return false;
            }
        }

        if let Some(start_time) = filter.start_time {
            if trace.start_time < start_time {
                return false;
            }
        }

        if let Some(end_time) = filter.end_time {
            if trace.start_time > end_time {
                return false;
            }
        }

        true
    }
}

impl Clone for LlmTelemetry {
    fn clone(&self) -> Self {
        Self {
            traces: self.traces.clone(),
            metrics: self.metrics.clone(),
            histograms: Arc::new(RwLock::new(PerformanceHistograms::default())), // Can't clone histograms
            cost_tracker: self.cost_tracker.clone(),
            alert_manager: self.alert_manager.clone(),
            config: self.config.clone(),
            active_requests: self.active_requests.clone(),
            error_tracker: self.error_tracker.clone(),
        }
    }
}

/// Request tracker for monitoring individual requests
pub struct RequestTracker {
    request_id: String,
    start_time: Instant,
    telemetry: Arc<LlmTelemetry>,
}

impl RequestTracker {
    /// Complete the request with results
    pub async fn complete(
        self,
        model: String,
        success: bool,
        tokens_response: usize,
        cost_usd: f64,
        error_message: Option<String>,
        error_type: Option<String>,
        request_source: String,
        prompt: Option<String>,
        response: Option<String>,
        tags: HashMap<String, String>,
    ) -> Result<()> {
        let end_time = Utc::now();
        let latency = self.start_time.elapsed();

        let active_request = self
            .telemetry
            .active_requests
            .get(&self.request_id)
            .ok_or_else(|| anyhow!("Active request not found"))?;

        let trace = LlmTrace {
            request_id: self.request_id.clone(),
            session_id: None,
            user_id: None,
            prompt,
            response,
            prompt_hash: None,
            model,
            start_time: end_time - chrono::Duration::from_std(latency).unwrap(),
            end_time,
            latency_ms: latency.as_millis() as u64,
            tokens_prompt: active_request.prompt_tokens,
            tokens_response,
            total_tokens: active_request.prompt_tokens + tokens_response,
            cost_usd,
            success,
            error_message,
            error_type,
            request_source,
            tags,
        };

        self.telemetry.record_request(trace).await
    }
}

/// Export format options
#[derive(Debug, Clone)]
pub enum ExportFormat {
    Json,
    Csv,
    OpenTelemetry,
}

/// Filter for trace exports
#[derive(Debug, Clone)]
pub struct TraceFilter {
    pub model: Option<String>,
    pub success: Option<bool>,
    pub start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub min_latency_ms: Option<u64>,
    pub max_latency_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_telemetry_creation() {
        let config = TelemetryConfig::default();
        let telemetry = LlmTelemetry::new(config);

        let metrics = telemetry.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn test_request_tracking() {
        let telemetry = LlmTelemetry::new(TelemetryConfig::default());

        let tracker = telemetry.start_request(
            "test-request".to_string(),
            "gpt-3.5-turbo".to_string(),
            "test-source".to_string(),
            100,
        );

        assert_eq!(telemetry.active_requests.len(), 1);

        tracker
            .complete(
                "gpt-3.5-turbo".to_string(),
                true,
                200,
                0.01,
                None,
                None,
                "test-source".to_string(),
                None,
                None,
                HashMap::new(),
            )
            .await
            .unwrap();

        assert_eq!(telemetry.active_requests.len(), 0);

        let metrics = telemetry.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        assert_eq!(metrics.successful_requests, 1);
    }

    #[tokio::test]
    async fn test_dashboard_data() {
        let telemetry = LlmTelemetry::new(TelemetryConfig::default());

        let dashboard = telemetry.get_dashboard_data().await.unwrap();
        assert_eq!(dashboard.current_metrics.total_requests, 0);
        assert!(dashboard.active_alerts.is_empty());
    }
}
