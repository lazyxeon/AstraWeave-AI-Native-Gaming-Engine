use anyhow::{Result, anyhow};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tracing::debug;

/// Advanced rate limiting system for LLM API calls
pub struct RateLimiter {
    /// Per-model rate limits
    model_limits: Arc<RwLock<HashMap<String, ModelRateLimit>>>,
    /// Per-user rate limits
    user_limits: Arc<RwLock<HashMap<String, UserRateLimit>>>,
    /// Global rate limit
    global_limit: Arc<GlobalRateLimit>,
    /// Configuration
    config: RateLimiterConfig,
}

#[derive(Debug, Clone)]
pub struct RateLimiterConfig {
    /// Default requests per minute per model
    pub default_rpm: u32,
    /// Default tokens per minute per model
    pub default_tpm: u32,
    /// Default requests per minute per user
    pub user_rpm: u32,
    /// Global requests per minute
    pub global_rpm: u32,
    /// Enable burst allowance
    pub allow_burst: bool,
    /// Burst multiplier (e.g., 2.0 = 2x normal rate for short periods)
    pub burst_multiplier: f32,
    /// Window size for rate calculations
    pub window_duration: Duration,
    /// Enable adaptive rate limiting based on success rate
    pub adaptive_limiting: bool,
}

impl Default for RateLimiterConfig {
    fn default() -> Self {
        Self {
            default_rpm: 1000,
            default_tpm: 50000,
            user_rpm: 100,
            global_rpm: 10000,
            allow_burst: true,
            burst_multiplier: 1.5,
            window_duration: Duration::from_secs(60),
            adaptive_limiting: true,
        }
    }
}

/// Per-model rate limiting
struct ModelRateLimit {
    model_name: String,
    requests_per_minute: u32,
    tokens_per_minute: u32,
    current_requests: u32,
    current_tokens: u32,
    window_start: Instant,
    request_semaphore: Arc<Semaphore>,
    token_semaphore: Arc<Semaphore>,
    success_rate: f32,
    adaptive_multiplier: f32,
}

impl ModelRateLimit {
    fn new(model_name: String, rpm: u32, tpm: u32) -> Self {
        Self {
            model_name,
            requests_per_minute: rpm,
            tokens_per_minute: tpm,
            current_requests: 0,
            current_tokens: 0,
            window_start: Instant::now(),
            request_semaphore: Arc::new(Semaphore::new(rpm as usize)),
            token_semaphore: Arc::new(Semaphore::new(tpm as usize)),
            success_rate: 1.0,
            adaptive_multiplier: 1.0,
        }
    }

    async fn can_proceed(&mut self, token_estimate: u32) -> bool {
        self.reset_window_if_needed();

        // Check if we have capacity
        let has_request_capacity = self.current_requests < (self.requests_per_minute as f32 * self.adaptive_multiplier) as u32;
        let has_token_capacity = self.current_tokens + token_estimate < (self.tokens_per_minute as f32 * self.adaptive_multiplier) as u32;

        has_request_capacity && has_token_capacity
    }

    async fn acquire(&mut self, token_estimate: u32) -> Result<RateLimit> {
        self.reset_window_if_needed();

        // Try to acquire semaphore permits
        let request_permit = self.request_semaphore.clone().try_acquire_owned()
            .map_err(|_| anyhow!("Request rate limit exceeded for model {}", self.model_name))?;

        let token_permits = if token_estimate > 0 {
            Some(self.token_semaphore.clone().try_acquire_many_owned(token_estimate)
                .map_err(|_| anyhow!("Token rate limit exceeded for model {}", self.model_name))?)
        } else {
            None
        };

        self.current_requests += 1;
        self.current_tokens += token_estimate;

        Ok(RateLimit {
            _request_permit: request_permit,
            _token_permits: token_permits,
        })
    }

    fn reset_window_if_needed(&mut self) {
        if self.window_start.elapsed() >= Duration::from_secs(60) {
            self.current_requests = 0;
            self.current_tokens = 0;
            self.window_start = Instant::now();
        }
    }

    fn update_success_rate(&mut self, success: bool) {
        let alpha = 0.1; // Exponential moving average factor
        let new_sample = if success { 1.0 } else { 0.0 };
        self.success_rate = alpha * new_sample + (1.0 - alpha) * self.success_rate;

        // Adjust rate based on success rate
        if self.success_rate < 0.8 {
            // Reduce rate if success rate is low
            self.adaptive_multiplier = (self.adaptive_multiplier * 0.9).max(0.1);
        } else if self.success_rate > 0.95 {
            // Increase rate if success rate is high
            self.adaptive_multiplier = (self.adaptive_multiplier * 1.1).min(2.0);
        }
    }
}

/// Per-user rate limiting
struct UserRateLimit {
    user_id: String,
    requests_per_minute: u32,
    current_requests: u32,
    window_start: Instant,
    semaphore: Arc<Semaphore>,
}

impl UserRateLimit {
    fn new(user_id: String, rpm: u32) -> Self {
        Self {
            user_id,
            requests_per_minute: rpm,
            current_requests: 0,
            window_start: Instant::now(),
            semaphore: Arc::new(Semaphore::new(rpm as usize)),
        }
    }

    async fn can_proceed(&mut self) -> bool {
        self.reset_window_if_needed();
        self.current_requests < self.requests_per_minute
    }

    async fn acquire(&mut self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        self.reset_window_if_needed();

        let permit = self.semaphore.clone().try_acquire_owned()
            .map_err(|_| anyhow!("User rate limit exceeded for user {}", self.user_id))?;

        self.current_requests += 1;
        Ok(permit)
    }

    fn reset_window_if_needed(&mut self) {
        if self.window_start.elapsed() >= Duration::from_secs(60) {
            self.current_requests = 0;
            self.window_start = Instant::now();
        }
    }
}

/// Global rate limiting
struct GlobalRateLimit {
    requests_per_minute: u32,
    current_requests: Arc<RwLock<u32>>,
    window_start: Arc<RwLock<Instant>>,
    semaphore: Arc<Semaphore>,
}

impl GlobalRateLimit {
    fn new(rpm: u32) -> Self {
        Self {
            requests_per_minute: rpm,
            current_requests: Arc::new(RwLock::new(0)),
            window_start: Arc::new(RwLock::new(Instant::now())),
            semaphore: Arc::new(Semaphore::new(rpm as usize)),
        }
    }

    async fn can_proceed(&self) -> bool {
        self.reset_window_if_needed().await;
        let current = *self.current_requests.read().await;
        current < self.requests_per_minute
    }

    async fn acquire(&self) -> Result<tokio::sync::OwnedSemaphorePermit> {
        self.reset_window_if_needed().await;

        let permit = self.semaphore.clone().try_acquire_owned()
            .map_err(|_| anyhow!("Global rate limit exceeded"))?;

        let mut current = self.current_requests.write().await;
        *current += 1;

        Ok(permit)
    }

    async fn reset_window_if_needed(&self) {
        let mut window_start = self.window_start.write().await;
        if window_start.elapsed() >= Duration::from_secs(60) {
            let mut current = self.current_requests.write().await;
            *current = 0;
            *window_start = Instant::now();
        }
    }
}

/// Rate limit permit that releases resources when dropped
pub struct RateLimit {
    _request_permit: tokio::sync::OwnedSemaphorePermit,
    _token_permits: Option<tokio::sync::OwnedSemaphorePermit>,
}

/// Rate limit check result
#[derive(Debug)]
pub struct RateLimitResult {
    pub allowed: bool,
    pub retry_after: Option<Duration>,
    pub reason: Option<String>,
}

/// Request context for rate limiting
#[derive(Debug, Clone)]
pub struct RateLimitContext {
    pub user_id: Option<String>,
    pub model: String,
    pub estimated_tokens: u32,
    pub priority: RequestPriority,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RequestPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl RateLimiter {
    pub fn new(config: RateLimiterConfig) -> Self {
        Self {
            model_limits: Arc::new(RwLock::new(HashMap::new())),
            user_limits: Arc::new(RwLock::new(HashMap::new())),
            global_limit: Arc::new(GlobalRateLimit::new(config.global_rpm)),
            config,
        }
    }

    /// Check if request can proceed without acquiring permits
    pub async fn check_rate_limit(&self, context: &RateLimitContext) -> RateLimitResult {
        // Check global limit first
        if !self.global_limit.can_proceed().await {
            return RateLimitResult {
                allowed: false,
                retry_after: Some(Duration::from_secs(1)),
                reason: Some("Global rate limit exceeded".to_string()),
            };
        }

        // Check user limit if user_id provided
        if let Some(user_id) = &context.user_id {
            let mut user_limits = self.user_limits.write().await;
            let user_limit = user_limits.entry(user_id.clone())
                .or_insert_with(|| UserRateLimit::new(user_id.clone(), self.config.user_rpm));

            if !user_limit.can_proceed().await {
                return RateLimitResult {
                    allowed: false,
                    retry_after: Some(Duration::from_secs(1)),
                    reason: Some(format!("User rate limit exceeded for user {}", user_id)),
                };
            }
        }

        // Check model limit
        let mut model_limits = self.model_limits.write().await;
        let model_limit = model_limits.entry(context.model.clone())
            .or_insert_with(|| ModelRateLimit::new(
                context.model.clone(),
                self.config.default_rpm,
                self.config.default_tpm,
            ));

        if !model_limit.can_proceed(context.estimated_tokens).await {
            return RateLimitResult {
                allowed: false,
                retry_after: Some(Duration::from_secs(1)),
                reason: Some(format!("Model rate limit exceeded for model {}", context.model)),
            };
        }

        RateLimitResult {
            allowed: true,
            retry_after: None,
            reason: None,
        }
    }

    /// Acquire rate limit permits - blocks until permits are available or returns error
    pub async fn acquire(&self, context: &RateLimitContext) -> Result<RateLimitPermits> {
        // Acquire global permit
        let global_permit = self.global_limit.acquire().await?;

        // Acquire user permit if needed
        let user_permit = if let Some(user_id) = &context.user_id {
            let mut user_limits = self.user_limits.write().await;
            let user_limit = user_limits.entry(user_id.clone())
                .or_insert_with(|| UserRateLimit::new(user_id.clone(), self.config.user_rpm));
            Some(user_limit.acquire().await?)
        } else {
            None
        };

        // Acquire model permit
        let model_permit = {
            let mut model_limits = self.model_limits.write().await;
            let model_limit = model_limits.entry(context.model.clone())
                .or_insert_with(|| ModelRateLimit::new(
                    context.model.clone(),
                    self.config.default_rpm,
                    self.config.default_tpm,
                ));
            model_limit.acquire(context.estimated_tokens).await?
        };

        debug!("Acquired rate limit permits for model {} (tokens: {})",
               context.model, context.estimated_tokens);

        Ok(RateLimitPermits {
            _global_permit: global_permit,
            _user_permit: user_permit,
            _model_permit: model_permit,
        })
    }

    /// Update success/failure for adaptive rate limiting
    pub async fn report_result(&self, context: &RateLimitContext, success: bool) {
        if !self.config.adaptive_limiting {
            return;
        }

        let mut model_limits = self.model_limits.write().await;
        if let Some(model_limit) = model_limits.get_mut(&context.model) {
            model_limit.update_success_rate(success);
            debug!("Updated success rate for model {}: {} (multiplier: {})",
                   context.model, model_limit.success_rate, model_limit.adaptive_multiplier);
        }
    }

    /// Get current rate limit status
    pub async fn get_status(&self) -> RateLimitStatus {
        let model_limits = self.model_limits.read().await;
        let user_limits = self.user_limits.read().await;

        let model_status: HashMap<String, ModelLimitStatus> = model_limits.iter()
            .map(|(name, limit)| {
                (name.clone(), ModelLimitStatus {
                    current_requests: limit.current_requests,
                    max_requests: limit.requests_per_minute,
                    current_tokens: limit.current_tokens,
                    max_tokens: limit.tokens_per_minute,
                    success_rate: limit.success_rate,
                    adaptive_multiplier: limit.adaptive_multiplier,
                })
            })
            .collect();

        let user_status: HashMap<String, UserLimitStatus> = user_limits.iter()
            .map(|(id, limit)| {
                (id.clone(), UserLimitStatus {
                    current_requests: limit.current_requests,
                    max_requests: limit.requests_per_minute,
                })
            })
            .collect();

        RateLimitStatus {
            global_current: *self.global_limit.current_requests.read().await,
            global_max: self.global_limit.requests_per_minute,
            model_status,
            user_status,
        }
    }

    /// Clear all rate limit data
    pub async fn clear(&self) {
        let mut model_limits = self.model_limits.write().await;
        model_limits.clear();

        let mut user_limits = self.user_limits.write().await;
        user_limits.clear();

        let mut global_current = self.global_limit.current_requests.write().await;
        *global_current = 0;

        let mut global_start = self.global_limit.window_start.write().await;
        *global_start = Instant::now();
    }
}

/// Combined rate limit permits
pub struct RateLimitPermits {
    _global_permit: tokio::sync::OwnedSemaphorePermit,
    _user_permit: Option<tokio::sync::OwnedSemaphorePermit>,
    _model_permit: RateLimit,
}

/// Current rate limit status
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct RateLimitStatus {
    pub global_current: u32,
    pub global_max: u32,
    pub model_status: HashMap<String, ModelLimitStatus>,
    pub user_status: HashMap<String, UserLimitStatus>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct ModelLimitStatus {
    pub current_requests: u32,
    pub max_requests: u32,
    pub current_tokens: u32,
    pub max_tokens: u32,
    pub success_rate: f32,
    pub adaptive_multiplier: f32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserLimitStatus {
    pub current_requests: u32,
    pub max_requests: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiter_creation() {
        let config = RateLimiterConfig::default();
        let rate_limiter = RateLimiter::new(config);

        let context = RateLimitContext {
            user_id: Some("test_user".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            priority: RequestPriority::Normal,
        };

        let result = rate_limiter.check_rate_limit(&context).await;
        assert!(result.allowed);
    }

    #[tokio::test]
    async fn test_rate_limit_enforcement() {
        let config = RateLimiterConfig {
            default_rpm: 2,
            default_tpm: 200,
            user_rpm: 1,
            global_rpm: 10,
            ..Default::default()
        };
        let rate_limiter = RateLimiter::new(config);

        let context = RateLimitContext {
            user_id: Some("test_user".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            priority: RequestPriority::Normal,
        };

        // First request should succeed
        let _permit1 = rate_limiter.acquire(&context).await.unwrap();

        // Second request should fail due to user limit
        let result = rate_limiter.acquire(&context).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_adaptive_rate_limiting() {
        let config = RateLimiterConfig {
            adaptive_limiting: true,
            ..Default::default()
        };
        let rate_limiter = RateLimiter::new(config);

        let context = RateLimitContext {
            user_id: None,
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            priority: RequestPriority::Normal,
        };

        // Report several failures
        for _ in 0..10 {
            rate_limiter.report_result(&context, false).await;
        }

        let status = rate_limiter.get_status().await;
        let model_status = status.model_status.get("gpt-3.5-turbo").unwrap();

        // Adaptive multiplier should be reduced due to failures
        assert!(model_status.adaptive_multiplier < 1.0);
    }
}