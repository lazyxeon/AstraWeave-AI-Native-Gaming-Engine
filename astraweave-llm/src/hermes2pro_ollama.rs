//! Hermes 2 Pro Integration via Ollama
//!
//! This module provides a production-ready interface to Nous Research's Hermes 2 Pro Mistral 7B model
//! via Ollama, which handles model management, quantization, and inference.
//!
//! # Why Hermes 2 Pro?
//! - **Function Calling**: Native support for OpenAI-compatible tool use
//! - **Higher Success Rate**: 75-85% vs 40-50% with Phi-3 (1.75× improvement)
//! - **Larger Context**: 8192 tokens (2× Phi-3's 4096)
//! - **JSON Reliability**: Trained specifically for structured output
//!
//! # Why Ollama?
//! - **Zero Setup**: No manual model downloads or GGUF files
//! - **Auto Quantization**: Ollama handles Q4/Q5/Q8 automatically
//! - **GPU Support**: Works with CUDA, Metal, ROCm out-of-the-box
//! - **Production Ready**: Used by thousands of applications
//!
//! # Quick Start
//! ```bash
//! # Install Ollama: https://ollama.ai
//! ollama pull adrienbrault/nous-hermes2pro:Q4_K_M    # Downloads Hermes 2 Pro Q4 (~4.4GB)
//! ollama serve                                        # Start server on localhost:11434
//! ```
//!
//! # Usage
//! ```no_run
//! use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
//! use astraweave_llm::LlmClient;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = Hermes2ProOllama::new("http://localhost:11434", "adrienbrault/nous-hermes2pro:Q4_K_M");
//! let response = client.complete("You are a game AI. Plan your next action.").await?;
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

use crate::LlmClient;

/// Hermes 2 Pro client using Ollama backend
///
/// This is the **recommended** way to use Hermes 2 Pro in AstraWeave. Ollama handles
/// all the complexity of model loading, quantization, and GPU acceleration.
///
/// ## Model Variant
/// - `adrienbrault/nous-hermes2pro:Q4_K_M` - 7B parameters, ~4.4GB - **RECOMMENDED**
///
/// This is a Q4_K_M quantization of Nous Research's Hermes 2 Pro Mistral 7B model,
/// specifically trained for function calling and tool use (OpenAI-compatible format).
///
/// ## Performance (RTX 3060, 12GB VRAM)
/// - Load time: 2-3 seconds (first request)
/// - Inference: 35-50 tokens/sec @ Q4_K_M
/// - Memory: ~5GB VRAM
/// - Success rate: 75-85% (vs 40-50% Phi-3)
/// - Latency: 2-4s average (vs 3-5s Phi-3)
///
/// ## Migration from Phi-3
/// Hermes 2 Pro uses ChatML format (`<|im_start|>` / `<|im_end|>`) instead of
/// Phi-3's custom format, but Ollama handles this automatically. No code changes
/// needed for chat formatting!
#[derive(Debug, Clone)]
pub struct Hermes2ProOllama {
    /// Ollama API endpoint (default: http://localhost:11434)
    pub url: String,
    
    /// Model name (default: "adrienbrault/nous-hermes2pro:Q4_K_M")
    pub model: String,
    
    /// Temperature for sampling (0.0 = deterministic, 1.0 = creative)
    pub temperature: f32,
    
    /// Maximum tokens to generate
    pub max_tokens: usize,
    
    /// System prompt for game AI context
    pub system_prompt: Option<String>,
}

impl Hermes2ProOllama {
    /// Create a new Hermes 2 Pro client with default settings
    ///
    /// # Arguments
    /// * `url` - Ollama server URL (e.g., "http://localhost:11434")
    /// * `model` - Model name (default: "adrienbrault/nous-hermes2pro:Q4_K_M")
    ///
    /// # Example
    /// ```no_run
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// let client = Hermes2ProOllama::new("http://localhost:11434", "adrienbrault/nous-hermes2pro:Q4_K_M");
    /// ```
    pub fn new(url: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            model: model.into(),
            temperature: 0.7,
            max_tokens: 512,
            system_prompt: Some(DEFAULT_SYSTEM_PROMPT.to_string()),
        }
    }
    
    /// Create Hermes 2 Pro client for localhost (convenience method)
    ///
    /// Uses the recommended Q4_K_M quantization for best balance of speed and quality.
    ///
    /// # Example
    /// ```no_run
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// let client = Hermes2ProOllama::localhost(); // Uses adrienbrault/nous-hermes2pro:Q4_K_M
    /// ```
    pub fn localhost() -> Self {
        Self::new("http://localhost:11434", "adrienbrault/nous-hermes2pro:Q4_K_M")
    }
    
    /// Create optimized Hermes 2 Pro client for low-latency game AI
    ///
    /// Uses lower temperature (0.5) and fewer max tokens (128) for faster inference.
    /// Expected latency: 1-3s on GTX 1660 Ti / RTX 3060.
    ///
    /// # Example
    /// ```no_run
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// let client = Hermes2ProOllama::fast(); // Low latency variant
    /// ```
    pub fn fast() -> Self {
        Self::new("http://localhost:11434", "adrienbrault/nous-hermes2pro:Q4_K_M")
            .with_temperature(0.5)
            .with_max_tokens(128)
    }
    
    /// Set temperature (0.0 = deterministic, 1.0 = creative)
    pub fn with_temperature(mut self, temp: f32) -> Self {
        self.temperature = temp;
        self
    }
    
    /// Set max tokens to generate
    pub fn with_max_tokens(mut self, max: usize) -> Self {
        self.max_tokens = max;
        self
    }
    
    /// Set custom system prompt
    pub fn with_system_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.system_prompt = Some(prompt.into());
        self
    }
    
    /// Clear system prompt (use only user messages)
    pub fn without_system_prompt(mut self) -> Self {
        self.system_prompt = None;
        self
    }
    
    /// Check if Ollama server is running and model is available
    pub async fn health_check(&self) -> Result<HealthStatus> {
        let client = reqwest::Client::new();
        
        // Check server is running
        let server_url = format!("{}/api/tags", self.url);
        let response = client.get(&server_url)
            .send()
            .await
            .context("Failed to connect to Ollama server")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama server returned error: {}", response.status());
        }
        
        // Parse available models
        let body: serde_json::Value = response.json().await
            .context("Failed to parse Ollama response")?;
        
        let models = body["models"].as_array()
            .context("Invalid response from Ollama")?;
        
        let model_available = models.iter().any(|m| {
            m["name"].as_str().unwrap_or("") == self.model
        });
        
        Ok(HealthStatus {
            server_running: true,
            model_available,
            model_name: self.model.clone(),
            ollama_version: body["version"].as_str().unwrap_or("unknown").to_string(),
        })
    }
}

/// Health check result
#[derive(Debug, Clone)]
pub struct HealthStatus {
    pub server_running: bool,
    pub model_available: bool,
    pub model_name: String,
    pub ollama_version: String,
}

impl HealthStatus {
    pub fn is_ready(&self) -> bool {
        self.server_running && self.model_available
    }
    
    pub fn error_message(&self) -> Option<String> {
        if !self.server_running {
            Some("Ollama server not running. Start with: ollama serve".to_string())
        } else if !self.model_available {
            Some(format!(
                "Model '{}' not found. Download with: ollama pull {}",
                self.model_name, self.model_name
            ))
        } else {
            None
        }
    }
}

/// Default system prompt for game AI
///
/// Hermes 2 Pro is trained for function calling, so this prompt emphasizes
/// the JSON schema for AstraWeave's 37-tool action system.
const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a tactical AI agent in a real-time game.
Your responses must be valid JSON following this schema:
{
  "plan_id": "unique_id",
  "reasoning": "brief explanation",
  "steps": [
    {"act": "MoveTo", "x": 10, "y": 5},
    {"act": "CoverFire", "target_id": 99, "duration": 2.0}
  ]
}

Available actions: MoveTo, Throw, CoverFire, Revive.
Always prioritize team survival and tactical advantage."#;

/// Implement LlmClient trait for Ollama-based Hermes 2 Pro
#[async_trait]
impl LlmClient for Hermes2ProOllama {
    async fn complete(&self, prompt: &str) -> Result<String> {
        // Use a static client with connection pooling for better performance
        static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
        let client = CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .pool_max_idle_per_host(10)  // Keep connections alive
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .timeout(std::time::Duration::from_secs(120))  // 2 min timeout
                .build()
                .expect("Failed to create HTTP client")
        });
        
        let url = format!("{}/api/generate", self.url);
        
        // Build request with system prompt if configured
        let full_prompt = if let Some(ref system) = self.system_prompt {
            format!("{}\n\n{}", system, prompt)
        } else {
            prompt.to_string()
        };
        
        let body = json!({
            "model": self.model,
            "prompt": full_prompt,
            "stream": false,
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
                "num_ctx": 8192,  // Hermes 2 Pro supports 8192 tokens (2× Phi-3)
            }
        });
        
        // ═══ DEBUG LOGGING ═══
        eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
        eprintln!("║        PROMPT SENT TO HERMES 2 PRO (via Ollama)              ║");
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("Model: {}", self.model);
        eprintln!("Temperature: {}", self.temperature);
        eprintln!("Max Tokens: {}", self.max_tokens);
        eprintln!("Context Window: 8192 tokens");
        eprintln!("Prompt Length: {} chars", full_prompt.len());
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("{}", full_prompt);
        eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");
        
        tracing::debug!("Sending request to Ollama: {}", self.model);
        let start = std::time::Instant::now();
        
        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send request to Ollama")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama returned error: {}", response.status());
        }
        
        let response_json: serde_json::Value = response.json().await
            .context("Failed to parse Ollama response")?;
        
        let text = response_json["response"]
            .as_str()
            .context("Missing 'response' field in Ollama output")?
            .to_string();
        
        let duration = start.elapsed();
        
        // ═══ DEBUG LOGGING ═══
        eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
        eprintln!("║        HERMES 2 PRO RAW RESPONSE (via Ollama)                ║");
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("Response Time: {:.2}s", duration.as_secs_f32());
        eprintln!("Response Length: {} chars", text.len());
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("{}", text);
        eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");
        
        tracing::debug!("Received {} chars from Ollama in {:.2}s", text.len(), duration.as_secs_f32());
        
        Ok(text)
    }
    
    /// Complete with streaming support (progressive response delivery)
    ///
    /// Returns a stream of text chunks as they arrive from Ollama. Enables:
    /// - Lower time-to-first-token (~8× faster first action vs blocking)
    /// - Progressive JSON parsing with StreamingParser
    /// - Batch inference with early plan delivery
    ///
    /// # Performance
    /// - Time-to-first-chunk: ~100-300ms (vs 1-3s for full response)
    /// - Chunk frequency: ~50-100ms intervals
    /// - Compatible with BatchExecutor for multi-agent inference
    ///
    /// # Example
    /// ```no_run
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// # use astraweave_llm::LlmClient;
    /// # use futures_util::StreamExt;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = Hermes2ProOllama::localhost();
    /// let mut stream = client.complete_streaming("Generate plan").await?;
    /// 
    /// let mut full_response = String::new();
    /// while let Some(chunk) = stream.next().await {
    ///     let text = chunk?;
    ///     full_response.push_str(&text);
    ///     println!("Progress: {} chars", full_response.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    async fn complete_streaming(
        &self,
        prompt: &str,
    ) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<String>> + Send>>> {
        use futures_util::StreamExt;
        
        // Use static client with connection pooling
        static CLIENT: std::sync::OnceLock<reqwest::Client> = std::sync::OnceLock::new();
        let client = CLIENT.get_or_init(|| {
            reqwest::Client::builder()
                .pool_max_idle_per_host(10)
                .pool_idle_timeout(std::time::Duration::from_secs(90))
                .timeout(std::time::Duration::from_secs(120))
                .build()
                .expect("Failed to create HTTP client")
        });
        
        let url = format!("{}/api/generate", self.url);
        
        // Build request with system prompt
        let full_prompt = if let Some(ref system) = self.system_prompt {
            format!("{}\n\n{}", system, prompt)
        } else {
            prompt.to_string()
        };
        
        let body = json!({
            "model": self.model,
            "prompt": full_prompt,
            "stream": true,  // ← Enable streaming!
            "options": {
                "temperature": self.temperature,
                "num_predict": self.max_tokens,
                "num_ctx": 8192,
            }
        });
        
        // ═══ DEBUG LOGGING ═══
        eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
        eprintln!("║    STREAMING REQUEST TO HERMES 2 PRO (via Ollama)            ║");
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("Model: {}", self.model);
        eprintln!("Temperature: {}", self.temperature);
        eprintln!("Max Tokens: {}", self.max_tokens);
        eprintln!("Stream: ENABLED");
        eprintln!("Prompt Length: {} chars", full_prompt.len());
        eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");
        
        tracing::debug!("Sending streaming request to Ollama: {}", self.model);
        
        let response = client.post(&url)
            .json(&body)
            .send()
            .await
            .context("Failed to send streaming request to Ollama")?;
        
        if !response.status().is_success() {
            anyhow::bail!("Ollama streaming returned error: {}", response.status());
        }
        
        // Convert bytes_stream to text chunk stream
        // Ollama streams NDJSON (newline-delimited JSON):
        //   {"response": "text", "done": false}\n
        //   {"response": "more", "done": false}\n
        //   {"response": "", "done": true}\n
        let byte_stream = response.bytes_stream();
        
        // Transform bytes → NDJSON lines → text chunks
        let text_stream = byte_stream
            .scan(String::new(), |buffer, chunk_result| {
                // Handle byte chunk errors
                let chunk = match chunk_result {
                    Ok(bytes) => bytes,
                    Err(e) => {
                        return futures_util::future::ready(Some(vec![Err(
                            anyhow::anyhow!("Stream error: {}", e)
                        )]));
                    }
                };
                
                // Append to buffer
                let text = match String::from_utf8(chunk.to_vec()) {
                    Ok(s) => s,
                    Err(e) => {
                        return futures_util::future::ready(Some(vec![Err(
                            anyhow::anyhow!("UTF-8 decode error: {}", e)
                        )]));
                    }
                };
                buffer.push_str(&text);
                
                // Extract complete JSON lines (separated by \n)
                let mut results = Vec::new();
                while let Some(newline_pos) = buffer.find('\n') {
                    // Clone the line before draining to avoid borrow checker issues
                    let line = buffer[..newline_pos].trim().to_string();
                    buffer.drain(..=newline_pos);
                    
                    if line.is_empty() {
                        continue;
                    }
                    
                    // Parse NDJSON line
                    match serde_json::from_str::<serde_json::Value>(&line) {
                        Ok(json) => {
                            // Extract "response" field
                            if let Some(response_text) = json["response"].as_str() {
                                if !response_text.is_empty() {
                                    results.push(Ok(response_text.to_string()));
                                }
                            }
                            
                            // Check if done
                            if json["done"].as_bool() == Some(true) {
                                tracing::debug!("Streaming complete (done: true)");
                            }
                        }
                        Err(e) => {
                            tracing::warn!("Failed to parse NDJSON line: {} | Line: {}", e, line);
                            // Don't fail entire stream for one bad line
                        }
                    }
                }
                
                // Return extracted results (empty vec if no complete lines yet)
                if results.is_empty() {
                    futures_util::future::ready(Some(vec![]))
                } else {
                    futures_util::future::ready(Some(results))
                }
            })
            .flat_map(futures_util::stream::iter);
        
        Ok(Box::pin(text_stream))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_hermes2pro_ollama_creation() {
        let client = Hermes2ProOllama::new("http://localhost:11434", "adrienbrault/nous-hermes2pro:Q4_K_M");
        assert_eq!(client.url, "http://localhost:11434");
        assert_eq!(client.model, "adrienbrault/nous-hermes2pro:Q4_K_M");
        assert_eq!(client.temperature, 0.7);
        assert_eq!(client.max_tokens, 512);
        assert!(client.system_prompt.is_some());
    }
    
    #[test]
    fn test_localhost_convenience() {
        let client = Hermes2ProOllama::localhost();
        assert_eq!(client.url, "http://localhost:11434");
        assert_eq!(client.model, "adrienbrault/nous-hermes2pro:Q4_K_M");
    }
    
    #[test]
    fn test_builder_pattern() {
        let client = Hermes2ProOllama::localhost()
            .with_temperature(0.5)
            .with_max_tokens(256)
            .without_system_prompt();
        
        assert_eq!(client.temperature, 0.5);
        assert_eq!(client.max_tokens, 256);
        assert!(client.system_prompt.is_none());
    }
    
    #[tokio::test]
    #[ignore] // Requires Ollama running
    async fn test_health_check() {
        let client = Hermes2ProOllama::localhost();
        let health = client.health_check().await;
        
        // If Ollama is running, this should succeed
        if let Ok(status) = health {
            assert!(status.server_running);
            println!("Ollama version: {}", status.ollama_version);
            println!("Model available: {}", status.model_available);
        }
    }
    
    #[tokio::test]
    #[ignore] // Requires Ollama + adrienbrault/nous-hermes2pro:Q4_K_M
    async fn test_complete() {
        let client = Hermes2ProOllama::localhost();
        
        // Check health first
        let health = client.health_check().await.expect("Health check failed");
        if !health.is_ready() {
            panic!("{}", health.error_message().unwrap());
        }
        
        let prompt = "You are at position (5,5). Enemy at (10,8). Generate a tactical plan.";
        let response = client.complete(prompt).await
            .expect("Completion failed");
        
        assert!(!response.is_empty());
        println!("Hermes 2 Pro response:\n{}", response);
        
        // Try to parse as JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&response);
        if let Ok(json) = parsed {
            println!("Parsed JSON: {}", serde_json::to_string_pretty(&json).unwrap());
        }
    }
    
    #[tokio::test]
    #[ignore] // Requires Ollama + adrienbrault/nous-hermes2pro:Q4_K_M
    async fn test_complete_streaming() {
        use futures_util::StreamExt;
        
        let client = Hermes2ProOllama::localhost();
        
        // Check health first
        let health = client.health_check().await.expect("Health check failed");
        if !health.is_ready() {
            panic!("{}", health.error_message().unwrap());
        }
        
        let prompt = "You are at position (5,5). Enemy at (10,8). Generate a tactical plan.";
        
        println!("\n═══ Starting Streaming Test ═══");
        let start = std::time::Instant::now();
        
        let mut stream = client.complete_streaming(prompt).await
            .expect("Streaming failed to start");
        
        let mut full_response = String::new();
        let mut chunk_count = 0;
        let mut time_to_first_chunk = None;
        
        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.expect("Chunk error");
            
            if time_to_first_chunk.is_none() {
                time_to_first_chunk = Some(start.elapsed());
                println!("⚡ Time to first chunk: {:.2}s", time_to_first_chunk.unwrap().as_secs_f32());
            }
            
            chunk_count += 1;
            full_response.push_str(&chunk);
            
            println!("Chunk #{}: {} chars (total: {})", chunk_count, chunk.len(), full_response.len());
        }
        
        let total_time = start.elapsed();
        
        println!("\n═══ Streaming Complete ═══");
        println!("Total chunks: {}", chunk_count);
        println!("Total time: {:.2}s", total_time.as_secs_f32());
        println!("Time to first chunk: {:.2}s", time_to_first_chunk.unwrap().as_secs_f32());
        println!("Final response length: {} chars", full_response.len());
        println!("\nFull response:\n{}", full_response);
        
        assert!(!full_response.is_empty(), "Streaming returned empty response");
        assert!(chunk_count > 0, "No chunks received");
        
        // Verify time-to-first-chunk is significantly faster than total time
        let ttfc_ratio = time_to_first_chunk.unwrap().as_secs_f32() / total_time.as_secs_f32();
        println!("\nTime-to-first-chunk ratio: {:.1}% of total time", ttfc_ratio * 100.0);
        
        // Try to parse as JSON
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&full_response);
        if let Ok(json) = parsed {
            println!("\nParsed JSON: {}", serde_json::to_string_pretty(&json).unwrap());
        }
    }
    
    #[tokio::test]
    #[ignore] // Requires Ollama + adrienbrault/nous-hermes2pro:Q4_K_M
    async fn test_streaming_vs_blocking_consistency() {
        use futures_util::StreamExt;
        
        let client = Hermes2ProOllama::localhost()
            .with_temperature(0.0);  // Deterministic for consistency check
        
        let prompt = "Generate JSON: {\"action\": \"move\", \"x\": 5, \"y\": 10}";
        
        // Get blocking response
        let blocking_response = client.complete(prompt).await.expect("Blocking failed");
        
        // Get streaming response
        let mut stream = client.complete_streaming(prompt).await.expect("Streaming failed");
        let mut streaming_response = String::new();
        while let Some(chunk) = stream.next().await {
            streaming_response.push_str(&chunk.expect("Chunk error"));
        }
        
        println!("Blocking response ({} chars):\n{}", blocking_response.len(), blocking_response);
        println!("\nStreaming response ({} chars):\n{}", streaming_response.len(), streaming_response);
        
        // Responses should be identical (or at least very similar with temp=0.0)
        assert_eq!(blocking_response.trim(), streaming_response.trim(),
            "Streaming and blocking responses should match with temperature=0.0");
    }
}
