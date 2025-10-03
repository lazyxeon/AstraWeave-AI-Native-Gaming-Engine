//! Security and Sandboxing for AstraWeave
//!
//! This crate provides security features including:
//! - LLM prompt sanitization and sandboxing
//! - Script execution sandboxing with Rhai
//! - Input validation and anti-cheat measures
//! - Telemetry and monitoring systems

use anyhow::Result;
use astraweave_ecs::{App, Plugin, World};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Security configuration resource
#[derive(Clone, Debug)]
pub struct SecurityConfig {
    pub enable_sandboxing: bool,
    pub enable_llm_validation: bool,
    pub enable_script_sandbox: bool,
    pub max_script_execution_time_ms: u64,
    pub max_memory_usage_mb: usize,
}

/// Telemetry data collection
#[derive(Clone, Debug)]
pub struct TelemetryData {
    pub events: Vec<TelemetryEvent>,
    pub session_start: std::time::Instant,
    pub anomaly_count: u64,
}

/// Individual telemetry event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TelemetryEvent {
    pub timestamp: u64,
    pub event_type: String,
    pub severity: TelemetrySeverity,
    pub data: serde_json::Value,
}

/// Telemetry severity levels
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum TelemetrySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Script execution sandbox
#[derive(Clone)]
pub struct ScriptSandbox {
    pub engine: Arc<Mutex<rhai::Engine>>,
    pub allowed_functions: HashMap<String, String>,
    pub execution_limits: ExecutionLimits,
}

/// Execution limits for sandboxed scripts
#[derive(Clone, Debug)]
pub struct ExecutionLimits {
    pub max_operations: u64,
    pub max_memory_bytes: usize,
    pub timeout_ms: u64,
}

/// LLM validation and sanitization
#[derive(Clone, Debug)]
pub struct LLMValidator {
    pub banned_patterns: Vec<String>,
    pub allowed_domains: Vec<String>,
    pub max_prompt_length: usize,
    pub enable_content_filtering: bool,
}

/// Anti-cheat component for entities
#[derive(Clone, Debug)]
pub struct CAntiCheat {
    pub player_id: String,
    pub trust_score: f32,
    pub last_validation: u64,
    pub anomaly_flags: Vec<String>,
}

/// Input validation result
#[derive(Clone, Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub trust_score: f32,
    pub warnings: Vec<String>,
    pub anomalies: Vec<String>,
}

/// Security plugin for ECS integration
pub struct SecurityPlugin {
    config: SecurityConfig,
}

impl SecurityPlugin {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: SecurityConfig {
                enable_sandboxing: true,
                enable_llm_validation: true,
                enable_script_sandbox: true,
                max_script_execution_time_ms: 1000,
                max_memory_usage_mb: 50,
            },
        }
    }
}

impl Plugin for SecurityPlugin {
    fn build(&self, app: &mut App) {
        // Initialize security resources
        app.world.insert_resource(self.config.clone());
        app.world.insert_resource(TelemetryData {
            events: Vec::new(),
            session_start: std::time::Instant::now(),
            anomaly_count: 0,
        });

        // Initialize script sandbox
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(10000);
        engine.set_max_string_size(1000);

        let sandbox = ScriptSandbox {
            engine: Arc::new(Mutex::new(engine)),
            allowed_functions: HashMap::new(),
            execution_limits: ExecutionLimits {
                max_operations: 10000,
                max_memory_bytes: 1024 * 1024, // 1MB
                timeout_ms: self.config.max_script_execution_time_ms,
            },
        };

        app.world.insert_resource(sandbox);

        // Initialize LLM validator
        let llm_validator = LLMValidator {
            banned_patterns: vec![
                "system(".to_string(),
                "exec(".to_string(),
                "eval(".to_string(),
                "import ".to_string(),
            ],
            allowed_domains: vec![
                "openai.com".to_string(),
                "anthropic.com".to_string(),
                "localhost".to_string(),
            ],
            max_prompt_length: 10000,
            enable_content_filtering: true,
        };

        app.world.insert_resource(llm_validator);

        // Add security systems
        app.add_system("pre_simulation", input_validation_system);
        app.add_system("post_simulation", telemetry_collection_system);
        app.add_system("post_simulation", anomaly_detection_system);
    }
}

/// Input validation system
fn input_validation_system(world: &mut World) {
    let entities: Vec<_> = world.entities_with::<CAntiCheat>();
    
    for entity in entities {
        // Get anti_cheat component to read
        let validation_result = if let Some(anti_cheat) = world.get::<CAntiCheat>(entity) {
            validate_player_input(anti_cheat)
        } else {
            continue;
        };

        // Update trust score based on validation
        if let Some(anti_cheat) = world.get_mut::<CAntiCheat>(entity) {
            anti_cheat.trust_score =
                (anti_cheat.trust_score * 0.9) + (validation_result.trust_score * 0.1);
            anti_cheat.last_validation = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();

            // Record anomalies
            if !validation_result.anomalies.is_empty() {
                anti_cheat
                    .anomaly_flags
                    .extend(validation_result.anomalies.clone());

                // Collect data for telemetry
                let player_id = anti_cheat.player_id.clone();
                let trust_score = anti_cheat.trust_score;
                let timestamp = anti_cheat.last_validation;

                // Log telemetry event
                if let Some(telemetry) = world.get_resource_mut::<TelemetryData>() {
                    telemetry.events.push(TelemetryEvent {
                        timestamp,
                        event_type: "input_anomaly".to_string(),
                        severity: TelemetrySeverity::Warning,
                        data: serde_json::json!({
                            "player_id": player_id,
                            "anomalies": validation_result.anomalies,
                            "trust_score": trust_score
                        }),
                    });
                }
            }
        }
    }
}

/// Telemetry collection system
fn telemetry_collection_system(world: &mut World) {
    if let Some(telemetry) = world.get_resource_mut::<TelemetryData>() {
        // Clean up old events (keep last 1000)
        if telemetry.events.len() > 1000 {
            telemetry.events = telemetry.events.split_off(telemetry.events.len() - 1000);
        }

        // Log periodic telemetry summary
        let session_duration = telemetry.session_start.elapsed().as_secs();
        if session_duration % 60 == 0 && !telemetry.events.is_empty() {
            println!(
                "Telemetry: {} events in {} seconds, {} anomalies",
                telemetry.events.len(),
                session_duration,
                telemetry.anomaly_count
            );
        }
    }
}

/// Anomaly detection system
fn anomaly_detection_system(world: &mut World) {
    let mut total_anomalies = 0;
    let mut low_trust_players = 0;
    let mut total_players = 0;

    let entities: Vec<_> = world.entities_with::<CAntiCheat>();
    
    for entity in &entities {
        if let Some(anti_cheat) = world.get::<CAntiCheat>(*entity) {
            total_anomalies += anti_cheat.anomaly_flags.len() as u64;
            if anti_cheat.trust_score < 0.5 {
                low_trust_players += 1;
            }
            total_players += 1;
        }
    }

    if let Some(telemetry) = world.get_resource_mut::<TelemetryData>() {
        telemetry.anomaly_count = total_anomalies;

        // Detect systemic anomalies
        if low_trust_players > total_players / 2 {
            telemetry.events.push(TelemetryEvent {
                timestamp: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                event_type: "systemic_anomaly".to_string(),
                severity: TelemetrySeverity::Critical,
                data: serde_json::json!({
                    "low_trust_players": low_trust_players,
                    "total_players": total_players,
                    "total_anomalies": total_anomalies
                }),
            });
        }
    }
}

/// Validate player input for cheating
pub fn validate_player_input(anti_cheat: &CAntiCheat) -> ValidationResult {
    let mut warnings = Vec::new();
    let mut anomalies = Vec::new();
    let mut trust_score = 1.0;

    // Check for rapid input patterns (potential botting)
    if anti_cheat
        .anomaly_flags
        .contains(&"rapid_input".to_string())
    {
        anomalies.push("rapid_input".to_string());
        trust_score *= 0.8;
        warnings.push("Unusually rapid input detected".to_string());
    }

    // Check for impossible movement patterns
    if anti_cheat
        .anomaly_flags
        .contains(&"impossible_movement".to_string())
    {
        anomalies.push("impossible_movement".to_string());
        trust_score *= 0.5;
        warnings.push("Impossible movement pattern detected".to_string());
    }

    // Check for memory tampering indicators
    if anti_cheat
        .anomaly_flags
        .contains(&"memory_tamper".to_string())
    {
        anomalies.push("memory_tamper".to_string());
        trust_score *= 0.3;
        warnings.push("Memory tampering indicators detected".to_string());
    }

    ValidationResult {
        is_valid: trust_score > 0.2,
        trust_score,
        warnings,
        anomalies,
    }
}

/// Sanitize LLM prompt for security
pub fn sanitize_llm_prompt(prompt: &str, validator: &LLMValidator) -> Result<String> {
    // Check prompt length
    if prompt.len() > validator.max_prompt_length {
        anyhow::bail!(
            "Prompt too long: {} > {}",
            prompt.len(),
            validator.max_prompt_length
        );
    }

    // Check for banned patterns
    for pattern in &validator.banned_patterns {
        if prompt.contains(pattern) {
            anyhow::bail!("Prompt contains banned pattern: {}", pattern);
        }
    }

    // Basic content filtering
    if validator.enable_content_filtering {
        // TODO: Implement more sophisticated content filtering
        let suspicious_patterns = ["hack", "exploit", "cheat", "bypass"];
        for pattern in &suspicious_patterns {
            if prompt.to_lowercase().contains(pattern) {
                return Ok(format!("SAFE: {}", prompt)); // Prefix safe prompts
            }
        }
    }

    Ok(prompt.to_string())
}

/// Execute script in sandbox
pub async fn execute_script_sandboxed(
    script: &str,
    sandbox: &ScriptSandbox,
    context: HashMap<String, rhai::Dynamic>,
) -> Result<rhai::Dynamic> {
    let script = script.to_string();
    let engine = sandbox.engine.clone();
    let timeout_ms = sandbox.execution_limits.timeout_ms;

    // Execute with timeout in a blocking task
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(timeout_ms),
        tokio::task::spawn_blocking(move || -> Result<rhai::Dynamic> {
            let engine = engine.lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
            
            // Compile the script
            let ast = engine.compile(&script)?;

            // Create scope with context variables
            let mut scope = rhai::Scope::new();
            for (key, value) in context {
                scope.push(key, value);
            }

            // Execute the script
            let result = engine.eval_ast_with_scope::<rhai::Dynamic>(&mut scope, &ast)?;
            Ok(result)
        }),
    )
    .await??;

    result
}

/// Generate cryptographic signature for data integrity
pub fn generate_signature(data: &[u8], signing_key: &SigningKey) -> Signature {
    signing_key.sign(data)
}

/// Verify cryptographic signature
pub fn verify_signature(data: &[u8], signature: &Signature, verifying_key: &VerifyingKey) -> bool {
    verifying_key.verify(data, signature).is_ok()
}

/// Generate a new keypair for signing
pub fn generate_keypair() -> (SigningKey, VerifyingKey) {
    let signing_key = SigningKey::from_bytes(&rand::random());
    let verifying_key = signing_key.verifying_key();
    (signing_key, verifying_key)
}

/// Hash data for integrity checking
pub fn hash_data(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn llm_prompt_sanitization() {
        let validator = LLMValidator {
            banned_patterns: vec!["system(".to_string(), "exec(".to_string()],
            allowed_domains: vec![],
            max_prompt_length: 1000,
            enable_content_filtering: true,
        };

        // Valid prompt
        let result = sanitize_llm_prompt("Hello, how are you?", &validator).unwrap();
        assert_eq!(result, "Hello, how are you?");

        // Banned pattern
        let result = sanitize_llm_prompt("Please system(exit)", &validator);
        assert!(result.is_err());

        // Too long
        let long_prompt = "x".repeat(2000);
        let result = sanitize_llm_prompt(&long_prompt, &validator);
        assert!(result.is_err());
    }

    #[test]
    fn input_validation() {
        let anti_cheat = CAntiCheat {
            player_id: "test_player".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string()],
        };

        let result = validate_player_input(&anti_cheat);
        assert!(result.is_valid);
        assert!(result.trust_score < 1.0);
        assert!(!result.anomalies.is_empty());
    }

    #[test]
    fn cryptographic_signing() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"Hello, world!";
        let signature = generate_signature(data, &signing_key);

        assert!(verify_signature(data, &signature, &verifying_key));

        // Tampered data should fail verification
        let tampered_data = b"Hello, world?";
        assert!(!verify_signature(tampered_data, &signature, &verifying_key));
    }

    #[test]
    fn data_hashing() {
        let data = b"Hello, world!";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);

        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 hex length
    }

    #[tokio::test]
    async fn script_sandbox_execution() {
        let mut engine = rhai::Engine::new();
        engine.set_max_operations(1000);
        
        let sandbox = ScriptSandbox {
            engine: Arc::new(Mutex::new(engine)),
            allowed_functions: HashMap::new(),
            execution_limits: ExecutionLimits {
                max_operations: 1000,
                max_memory_bytes: 1024 * 1024,
                timeout_ms: 100,
            },
        };

        let script = "40 + 2";
        let context = HashMap::new();
        let result = execute_script_sandboxed(script, &sandbox, context)
            .await
            .unwrap();

        assert_eq!(result.as_int().unwrap(), 42);
    }
}
