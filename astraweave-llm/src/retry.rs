// Retry logic with exponential backoff for LLM operations
// Implements resilience patterns for transient failures

use std::time::Duration;

/// Retry configuration for LLM operations
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries)
    pub max_attempts: u32,
    
    /// Initial backoff duration
    pub initial_backoff_ms: u64,
    
    /// Backoff multiplier for each retry (exponential)
    pub backoff_multiplier: f64,
    
    /// Maximum backoff duration (cap)
    pub max_backoff_ms: u64,
    
    /// Whether to add jitter to backoff (reduces thundering herd)
    pub jitter: bool,
}

impl RetryConfig {
    /// Production-grade retry config (3 attempts, exponential backoff)
    pub fn production() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 50,
            backoff_multiplier: 2.0,
            max_backoff_ms: 500,
            jitter: true,
        }
    }

    /// Aggressive retry config (5 attempts, faster backoff)
    pub fn aggressive() -> Self {
        Self {
            max_attempts: 5,
            initial_backoff_ms: 25,
            backoff_multiplier: 1.5,
            max_backoff_ms: 300,
            jitter: true,
        }
    }

    /// Disabled (no retries)
    pub fn disabled() -> Self {
        Self {
            max_attempts: 0,
            initial_backoff_ms: 0,
            backoff_multiplier: 1.0,
            max_backoff_ms: 0,
            jitter: false,
        }
    }

    /// Calculate backoff duration for a given attempt
    pub fn backoff_for_attempt(&self, attempt: u32) -> Duration {
        if self.max_attempts == 0 {
            return Duration::from_millis(0);
        }

        // Exponential backoff: initial * multiplier^attempt
        let base_backoff = self.initial_backoff_ms as f64 
            * self.backoff_multiplier.powi(attempt as i32);
        
        // Apply cap
        let capped_backoff = base_backoff.min(self.max_backoff_ms as f64) as u64;

        // Add jitter (random ±25%) to reduce thundering herd
        let final_backoff = if self.jitter {
            let jitter_range = (capped_backoff as f64 * 0.25) as u64;
            let jitter = (rand::random::<u64>() % (jitter_range * 2)).saturating_sub(jitter_range);
            capped_backoff.saturating_add(jitter as u64)
        } else {
            capped_backoff
        };

        Duration::from_millis(final_backoff)
    }

    /// Check if retry should be attempted for this error
    pub fn should_retry(&self, error: &RetryableError) -> bool {
        match error {
            RetryableError::Timeout => true,
            RetryableError::NetworkError => true,
            RetryableError::RateLimited => true,
            RetryableError::ServerError(_) => true,
            RetryableError::Permanent(_) => false, // Never retry permanent errors
        }
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::production()
    }
}

/// Errors that can be retried
#[derive(Debug, Clone)]
pub enum RetryableError {
    /// Request timed out (transient)
    Timeout,
    
    /// Network connectivity issue (transient)
    NetworkError,
    
    /// Rate limit exceeded (transient, wait then retry)
    RateLimited,
    
    /// Server error 5xx (transient)
    ServerError(u16),
    
    /// Permanent error (4xx, invalid input, etc.)
    Permanent(String),
}

impl std::fmt::Display for RetryableError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RetryableError::Timeout => write!(f, "Request timeout"),
            RetryableError::NetworkError => write!(f, "Network error"),
            RetryableError::RateLimited => write!(f, "Rate limited"),
            RetryableError::ServerError(code) => write!(f, "Server error {}", code),
            RetryableError::Permanent(msg) => write!(f, "Permanent error: {}", msg),
        }
    }
}

impl std::error::Error for RetryableError {}

/// Retry executor - wraps an async operation with retry logic
pub struct RetryExecutor {
    config: RetryConfig,
}

impl RetryExecutor {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    /// Execute an async operation with retry logic
    /// 
    /// Returns Ok(T) on success, Err(RetryableError) on final failure
    pub async fn execute<F, Fut, T>(&self, mut operation: F) -> Result<T, RetryableError>
    where
        F: FnMut() -> Fut,
        Fut: std::future::Future<Output = Result<T, RetryableError>>,
    {
        let mut attempt = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(error) => {
                    // Check if we should retry
                    if !self.config.should_retry(&error) {
                        tracing::debug!("Permanent error, no retry: {}", error);
                        return Err(error);
                    }

                    // Check if we've exhausted retries
                    if attempt >= self.config.max_attempts {
                        tracing::warn!(
                            "Max retries ({}) exhausted for error: {}",
                            self.config.max_attempts,
                            error
                        );
                        return Err(error);
                    }

                    // Calculate backoff and retry
                    let backoff = self.config.backoff_for_attempt(attempt);
                    tracing::debug!(
                        "Retry attempt {} after error: {} (backoff: {:?})",
                        attempt + 1,
                        error,
                        backoff
                    );

                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_production() {
        let config = RetryConfig::production();
        assert_eq!(config.max_attempts, 3);
        assert_eq!(config.initial_backoff_ms, 50);
        assert!(config.jitter);
    }

    #[test]
    fn test_retry_config_disabled() {
        let config = RetryConfig::disabled();
        assert_eq!(config.max_attempts, 0);
        
        let backoff = config.backoff_for_attempt(0);
        assert_eq!(backoff.as_millis(), 0);
    }

    #[test]
    fn test_exponential_backoff() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
            jitter: false,
        };

        // Attempt 0: 100ms
        assert_eq!(config.backoff_for_attempt(0).as_millis(), 100);
        
        // Attempt 1: 200ms
        assert_eq!(config.backoff_for_attempt(1).as_millis(), 200);
        
        // Attempt 2: 400ms
        assert_eq!(config.backoff_for_attempt(2).as_millis(), 400);
        
        // Attempt 3: 800ms
        assert_eq!(config.backoff_for_attempt(3).as_millis(), 800);
        
        // Attempt 4: 1600ms capped to 1000ms
        assert_eq!(config.backoff_for_attempt(4).as_millis(), 1000);
    }

    #[test]
    fn test_backoff_cap() {
        let config = RetryConfig {
            max_attempts: 10,
            initial_backoff_ms: 100,
            backoff_multiplier: 10.0,
            max_backoff_ms: 500,
            jitter: false,
        };

        // Even with high multiplier, cap at 500ms
        assert_eq!(config.backoff_for_attempt(3).as_millis(), 500);
    }

    #[test]
    fn test_jitter_adds_randomness() {
        let config = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
            jitter: true,
        };

        // With jitter, backoff should vary (run multiple times)
        let backoff1 = config.backoff_for_attempt(0).as_millis();
        let backoff2 = config.backoff_for_attempt(0).as_millis();
        
        // Should be within ±25% of 100ms = 75-125ms range
        assert!(backoff1 >= 75 && backoff1 <= 125);
        assert!(backoff2 >= 75 && backoff2 <= 125);
    }

    #[test]
    fn test_should_retry_transient_errors() {
        let config = RetryConfig::production();
        
        assert!(config.should_retry(&RetryableError::Timeout));
        assert!(config.should_retry(&RetryableError::NetworkError));
        assert!(config.should_retry(&RetryableError::RateLimited));
        assert!(config.should_retry(&RetryableError::ServerError(503)));
    }

    #[test]
    fn test_should_not_retry_permanent_errors() {
        let config = RetryConfig::production();
        
        assert!(!config.should_retry(&RetryableError::Permanent("bad input".into())));
    }

    #[tokio::test]
    async fn test_retry_executor_success_first_try() {
        let config = RetryConfig::production();
        let executor = RetryExecutor::new(config);
        
        let mut call_count = 0;
        let result = executor.execute(|| {
            call_count += 1;
            async { Ok::<_, RetryableError>(42) }
        }).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count, 1); // No retries needed
    }

    #[tokio::test]
    async fn test_retry_executor_success_after_retries() {
        let config = RetryConfig {
            max_attempts: 3,
            initial_backoff_ms: 1, // Fast test
            backoff_multiplier: 1.0,
            max_backoff_ms: 1,
            jitter: false,
        };
        let executor = RetryExecutor::new(config);
        
        let mut call_count = 0;
        let result = executor.execute(|| {
            call_count += 1;
            async move {
                if call_count < 3 {
                    Err(RetryableError::Timeout)
                } else {
                    Ok::<_, RetryableError>(42)
                }
            }
        }).await;
        
        assert_eq!(result.unwrap(), 42);
        assert_eq!(call_count, 3); // Succeeded on 3rd attempt
    }

    #[tokio::test]
    async fn test_retry_executor_exhausts_retries() {
        let config = RetryConfig {
            max_attempts: 2,
            initial_backoff_ms: 1,
            backoff_multiplier: 1.0,
            max_backoff_ms: 1,
            jitter: false,
        };
        let executor = RetryExecutor::new(config);
        
        let mut call_count = 0;
        let result = executor.execute(|| {
            call_count += 1;
            async { Err::<i32, _>(RetryableError::Timeout) }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(call_count, 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_executor_permanent_error_no_retry() {
        let config = RetryConfig::production();
        let executor = RetryExecutor::new(config);
        
        let mut call_count = 0;
        let result = executor.execute(|| {
            call_count += 1;
            async { Err::<i32, _>(RetryableError::Permanent("bad input".into())) }
        }).await;
        
        assert!(result.is_err());
        assert_eq!(call_count, 1); // No retries for permanent errors
    }
}
