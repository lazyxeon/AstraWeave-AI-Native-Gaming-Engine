use anyhow::{bail, Result};
use astraweave_core::{ActionStep, PlanIntent, ToolRegistry, WorldSnapshot};

#[cfg(feature = "llm_cache")]
pub mod cache;

#[cfg(feature = "llm_cache")]
use cache::{CachedPlan, PromptCache, PromptKey};
#[cfg(feature = "llm_cache")]
use std::sync::LazyLock;

#[cfg(feature = "llm_cache")]
static GLOBAL_CACHE: LazyLock<PromptCache> = LazyLock::new(|| {
    // Read capacity from environment, default to 4096
    let capacity = std::env::var("LLM_CACHE_CAP")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(4096);
    PromptCache::new(capacity)
});

/// Clear the global LLM cache (useful for testing)
#[cfg(feature = "llm_cache")]
pub fn clear_global_cache() {
    GLOBAL_CACHE.clear();
}

/// Enum to distinguish between LLM-generated plans and fallback plans with error reasons
#[derive(Debug, Clone)]
pub enum PlanSource {
    Llm(PlanIntent),
    Fallback { plan: PlanIntent, reason: String },
}

/// Trait for LLM clients (mock, Ollama, etc).
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Mock client (no model). Emits a basic plan using simple heuristics.
pub struct MockLlm;

#[async_trait::async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, _prompt: &str) -> Result<String> {
        // A minimal JSON that follows our schema
        let out = r#"{
          "plan_id":"llm-mock",
          "steps":[
            {"act":"Throw","item":"smoke","x":7,"y":2},
            {"act":"MoveTo","x":4,"y":2},
            {"act":"CoverFire","target_id":99,"duration":2.0}
          ]
        }"#;
        Ok(out.into())
    }
}

/// Mock client that always returns an error (for testing fallback behavior)
pub struct AlwaysErrMock;

#[async_trait::async_trait]
impl LlmClient for AlwaysErrMock {
    async fn complete(&self, _prompt: &str) -> Result<String> {
        bail!("AlwaysErrMock: simulated LLM failure")
    }
}

#[cfg(feature = "ollama")]
pub struct OllamaClient {
    pub url: String,
    pub model: String,
}

#[cfg(feature = "ollama")]
#[async_trait::async_trait]
impl LlmClient for OllamaClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        #[derive(serde::Serialize)]
        struct Req<'a> {
            model: &'a str,
            prompt: &'a str,
            stream: bool,
        }
        #[derive(serde::Deserialize)]
        struct Resp {
            response: String,
        }

        // ═══ PHASE 7 DEBUG LOGGING ═══
        eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
        eprintln!("║           PROMPT SENT TO PHI-3 (via Ollama)                  ║");
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("Model: {}", self.model);
        eprintln!("URL: {}", self.url);
        eprintln!("Prompt Length: {} chars", prompt.len());
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("{}", prompt);
        eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");

        let body = Req {
            model: &self.model,
            prompt,
            stream: false,
        };

        let client = reqwest::Client::new();
        let start = std::time::Instant::now();
        
        let response = client
            .post(format!("{}/api/generate", self.url))
            .json(&body)
            .timeout(std::time::Duration::from_secs(30))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to Ollama: {}", e))?;

        if !response.status().is_success() {
            bail!("Ollama API returned error status: {}", response.status());
        }

        let parsed: Resp = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse Ollama response: {}", e))?;

        let duration = start.elapsed();

        // ═══ PHASE 7 DEBUG LOGGING ═══
        eprintln!("\n╔═══════════════════════════════════════════════════════════════╗");
        eprintln!("║           PHI-3 RAW RESPONSE (via Ollama)                    ║");
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("Response Time: {:.2}s", duration.as_secs_f32());
        eprintln!("Response Length: {} chars", parsed.response.len());
        eprintln!("╠═══════════════════════════════════════════════════════════════╣");
        eprintln!("{}", parsed.response);
        eprintln!("╚═══════════════════════════════════════════════════════════════╝\n");

        Ok(parsed.response)
    }
}

// New Ollama chat client that targets the local Ollama chat endpoint (e.g. http://127.0.0.1:11434/api/chat)
// This client is resilient to a couple of response shapes returned by different Ollama versions.
#[cfg(feature = "ollama")]
#[derive(Clone)]
pub struct OllamaChatClient {
    pub url: String,
    pub model: String,
    client: reqwest::Client,
    // Low-latency tuning knobs
    low_latency: bool,
    keep_alive: Option<String>, // e.g. "5m" to keep model in RAM
    force_format_json: bool,    // add format: "json" to requests
    early_exit_on_json: bool,   // return as soon as a balanced JSON object is detected
}

#[cfg(feature = "ollama")]
impl OllamaChatClient {
    pub fn new(url: String, model: String) -> Self {
        // Build a tuned reqwest client for local low-latency usage
        let client = reqwest::Client::builder()
            .tcp_nodelay(true)
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .pool_max_idle_per_host(8)
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());

        let low_latency = std::env::var("OLLAMA_LOW_LATENCY")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);
        let keep_alive = std::env::var("OLLAMA_KEEP_ALIVE").ok(); // e.g., "5m" or "3600s"
        let force_format_json = std::env::var("OLLAMA_FORMAT_JSON")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);
        let early_exit_on_json = std::env::var("OLLAMA_EARLY_EXIT")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(true);

        Self {
            url,
            model,
            client,
            low_latency,
            keep_alive,
            force_format_json,
            early_exit_on_json,
        }
    }

    /// Warm up the model to minimize cold-start latency. Attempts a tiny generation and requests the model to remain in memory.
    pub async fn warmup(&self, timeout_secs: u64) -> Result<()> {
        #[derive(serde::Serialize)]
        struct Msg<'a> {
            role: &'a str,
            content: &'a str,
        }
        #[derive(serde::Serialize)]
        struct Req<'a> {
            model: &'a str,
            messages: Vec<Msg<'a>>,
            #[serde(skip_serializing_if = "Option::is_none")]
            format: Option<&'a str>,
            #[serde(skip_serializing_if = "Option::is_none")]
            keep_alive: Option<&'a str>,
            stream: bool,
            options: serde_json::Value,
        }

        let format_opt = if self.force_format_json {
            Some("json")
        } else {
            None
        };
        let keep_alive_opt = self.keep_alive.as_deref();
        let options = serde_json::json!({ "num_predict": 1, "temperature": 0.0 });
        let body = Req {
            model: &self.model,
            messages: vec![Msg {
                role: "user",
                content: "ping",
            }],
            format: format_opt,
            keep_alive: keep_alive_opt,
            stream: false,
            options,
        };

        let chat_url = format!("{}/api/chat", self.url.trim_end_matches('/'));
        let _ = self
            .client
            .post(&chat_url)
            .header("Accept", "application/json")
            .timeout(std::time::Duration::from_secs(timeout_secs))
            .json(&body)
            .send()
            .await?;
        Ok(())
    }

    /// Experimental: issue multiple prompts sequentially while reusing the same HTTP client and model residency.
    /// Returns the collected raw responses (not parsed into plans). Intended for micro-batching during warm stages.
    pub async fn complete_batch(&self, prompts: &[String]) -> Result<Vec<String>> {
        let mut out = Vec::with_capacity(prompts.len());
        for p in prompts {
            out.push(self.complete(p).await?);
        }
        Ok(out)
    }
}

#[cfg(feature = "ollama")]
#[async_trait::async_trait]
impl LlmClient for OllamaChatClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        #[derive(serde::Serialize)]
        struct Msg<'a> {
            role: &'a str,
            content: &'a str,
        }
        #[derive(serde::Serialize)]
        struct Req<'a> {
            model: &'a str,
            messages: Vec<Msg<'a>>,
            stream: bool,
        }

        let body = Req {
            model: &self.model,
            messages: vec![Msg {
                role: "user",
                content: prompt,
            }],
            stream: false,
        };

        let client = self.client.clone();
        // Build a non-chunked JSON body string and set Connection: close so the server
        // will finish the response and close the connection (avoids some streaming behaviors).
        // Build a JSON body for non-streaming attempts. Use `.json()` on the request so
        // reqwest sets proper headers and Content-Length.

        // Some Ollama setups can take a long time to load a model initially (minutes) or
        // will stream responses. Allow a long configurable timeout and fall back to a
        // streaming read if a non-stream response isn't available.
        let default_timeout = std::env::var("OLLAMA_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .unwrap_or(180u64);

        // First try a non-streaming request with a long timeout so we get a buffered JSON
        // response if the server supports it and the model responds within the window.
        let mut text = String::new();
        let chat_url = format!("{}/api/chat", self.url.trim_end_matches('/'));
        // Configure non-stream attempts/timeout via env with conservative defaults
        let max_nonstream_attempts: u32 = std::env::var("OLLAMA_NONSTREAM_ATTEMPTS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(1);
        let nonstream_timeout_secs: u64 = std::env::var("OLLAMA_NONSTREAM_TIMEOUT_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(20);

        let mut non_stream_ok = false;
        let mut attempt = 0u32;
        while attempt < max_nonstream_attempts && !non_stream_ok {
            attempt += 1;
            let backoff = std::time::Duration::from_millis(250 * (1 << (attempt - 1)));
            println!(
                "[ollama] non-stream attempt {}/{} to {} (timeout {}s)",
                attempt, max_nonstream_attempts, chat_url, nonstream_timeout_secs
            );
            let ns_attempt_start = std::time::Instant::now();
            // Use .json() so reqwest sets Content-Length and proper headers. Some servers
            // close the connection or reject raw bodies without length which can cause
            // "error sending request" failures; using .json() is more reliable here.
            match client
                .post(&chat_url)
                .header("Accept", "application/json")
                .timeout(std::time::Duration::from_secs(nonstream_timeout_secs))
                .json(&body)
                .send()
                .await
            {
                Ok(resp) => {
                    println!(
                        "[ollama] non-streaming request returned status {}",
                        resp.status()
                    );
                    if resp.status().is_success() {
                        // Try to read as text (this will complete if the server returns a full body)
                        match resp.text().await {
                            Ok(t) if !t.trim().is_empty() => {
                                println!(
                                    "[ollama] received non-empty buffered body ({} bytes)",
                                    t.len()
                                );
                                text = t;
                                non_stream_ok = true;
                                let total_ms = ns_attempt_start.elapsed().as_millis();
                                println!(
                                    "[ollama] timing nonstream attempt {}: total={}ms",
                                    attempt, total_ms
                                );
                            }
                            Ok(_) => {
                                println!("[ollama] buffered body was empty on attempt {}, will retry/fallback", attempt);
                            }
                            Err(e) => {
                                println!(
                                    "[ollama] error reading non-streaming response text: {:?}",
                                    e
                                );
                            }
                        }
                    } else {
                        // Read response body for diagnostics
                        let status = resp.status();
                        let b = resp.text().await.unwrap_or_default();
                        println!(
                            "[ollama] non-streaming request returned non-success status: {}: {}",
                            status, b
                        );
                        let total_ms = ns_attempt_start.elapsed().as_millis();
                        println!(
                            "[ollama] timing nonstream attempt {}: total={}ms (non-success)",
                            attempt, total_ms
                        );
                    }
                }
                Err(e) => {
                    // Print full debug info for the error to aid diagnosis
                    println!(
                        "[ollama] non-streaming request failed on attempt {}: {:?}",
                        attempt, e
                    );
                    let total_ms = ns_attempt_start.elapsed().as_millis();
                    println!(
                        "[ollama] timing nonstream attempt {}: total={}ms (error)",
                        attempt, total_ms
                    );
                }
            }

            if !non_stream_ok {
                // small backoff before retrying
                tokio::time::sleep(backoff).await;
            }
        }

        // If we didn't get a usable buffered body, or we're in low-latency mode, attempt a streaming read where we
        // accumulate incoming bytes and try to extract the assistant content as it arrives.
        if text.trim().is_empty() || self.low_latency {
            // Build a streaming request (request the server to stream if it supports it)
            let mut stream_body = serde_json::json!({
                "model": &self.model,
                "messages": [{ "role": "user", "content": prompt }],
                "stream": true,
                "options": { "temperature": 0.1, "num_predict": 512 }
            });
            if self.force_format_json {
                stream_body["format"] = serde_json::json!("json");
            }
            if let Some(ka) = &self.keep_alive {
                stream_body["keep_alive"] = serde_json::json!(ka);
            }

            // Total streaming timeout to avoid hangs when the model never responds
            let total_stream_timeout = std::time::Duration::from_secs(default_timeout * 2);
            let start = std::time::Instant::now();

            let stream_url = format!("{}/api/chat", self.url.trim_end_matches('/'));
            println!("[ollama] initiating streaming POST to {}", stream_url);
            let resp = client
                .post(&stream_url)
                .header("Accept", "application/json")
                .header("Content-Type", "application/json")
                .json(&stream_body)
                .send()
                .await
                .map_err(|e| {
                    anyhow::anyhow!("Failed to initiate streaming request to Ollama: {}", e)
                })?;

            println!(
                "[ollama] streaming request returned status {}",
                resp.status()
            );

            // Determine which response we'll stream from: prefer /api/chat, but fall back to /api/generate
            let resp_for_stream: Option<reqwest::Response> = if resp.status().is_success() {
                Some(resp)
            } else {
                let status = resp.status();
                if status.as_u16() == 404 || status.as_u16() == 405 {
                    println!(
                        "[ollama] /api/chat returned {}, attempting /api/generate as fallback",
                        status
                    );
                    let gen_url = format!("{}/api/generate", self.url.trim_end_matches('/'));
                    let gen_resp = client
                        .post(&gen_url)
                        .header("Accept", "application/json")
                        .header("Content-Type", "application/json")
                        .json(&stream_body)
                        .send()
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!(
                                "Failed to initiate streaming request to Ollama /api/generate: {}",
                                e
                            )
                        })?;

                    println!(
                        "[ollama] /api/generate returned status {}",
                        gen_resp.status()
                    );
                    if gen_resp.status().is_success() {
                        Some(gen_resp)
                    } else {
                        let s = gen_resp.status();
                        let b = gen_resp.text().await.unwrap_or_default();
                        return Err(anyhow::anyhow!(
                            "Ollama /api/generate returned error status {}: {}",
                            s,
                            b
                        ));
                    }
                } else {
                    let txt = resp.text().await.unwrap_or_default();
                    return Err(anyhow::anyhow!(
                        "Ollama chat API returned error status {}: {}",
                        status,
                        txt
                    ));
                }
            };

            // Now unwrap the response we will stream from
            let resp = resp_for_stream
                .ok_or_else(|| anyhow::anyhow!("No streaming response available"))?;

            // Use the bytes_stream to receive chunks as they arrive. Ollama streams envelope JSON
            // objects per-line where assistant tokens are nested inside `message.content`.
            // We must parse each line, extract `message.content` fragments, concatenate them,
            // and then attempt to extract the final JSON object.
            let mut stream = resp.bytes_stream();
            use futures_util::StreamExt;

            let mut buf = String::new();
            let mut assistant_acc = String::new();
            let mut done_flag = false;

            // We'll periodically flush partial assembled assistant output to disk so
            // an interrupted run can still be inspected. Track last flush size.
            #[allow(unused_mut)]
            let mut last_flush = 0usize;
            let mut first_token_at: Option<std::time::Instant> = None;
            while let Some(item) =
                tokio::time::timeout(std::time::Duration::from_secs(10), stream.next())
                    .await
                    .ok()
                    .flatten()
            {
                match item {
                    Ok(chunk) => {
                        if let Ok(schunk) = std::str::from_utf8(&chunk) {
                            if first_token_at.is_none() {
                                first_token_at = Some(std::time::Instant::now());
                            }
                            buf.push_str(schunk);

                            // Process complete newline-terminated records
                            while let Some(pos) = buf.find('\n') {
                                let line = buf[..pos].trim();
                                let rest = buf[pos + 1..].to_string();

                                if !line.is_empty() {
                                    // Try to parse the envelope JSON line
                                    if let Ok(v) = serde_json::from_str::<serde_json::Value>(line) {
                                        // Extract nested message.content if present
                                        if let Some(msg) = v.get("message") {
                                            if let Some(content) =
                                                msg.get("content").and_then(|c| c.as_str())
                                            {
                                                assistant_acc.push_str(content);
                                            }
                                        }

                                        // Some Ollama variants may include a top-level "response" or choices
                                        if let Some(resp_txt) =
                                            v.get("response").and_then(|r| r.as_str())
                                        {
                                            assistant_acc.push_str(resp_txt);
                                        }

                                        if let Some(choices) =
                                            v.get("choices").and_then(|c| c.as_array())
                                        {
                                            if let Some(first) = choices.get(0) {
                                                if let Some(msg) = first.get("message") {
                                                    if let Some(content) =
                                                        msg.get("content").and_then(|c| c.as_str())
                                                    {
                                                        assistant_acc.push_str(content);
                                                    }
                                                }
                                            }
                                        }

                                        if v.get("done").and_then(|d| d.as_bool()) == Some(true) {
                                            done_flag = true;
                                        }
                                    } else {
                                        // Not valid JSON line; append raw
                                        assistant_acc.push_str(line);
                                    }
                                }

                                buf = rest;
                                // Periodically persist partial output so it's available if the
                                // process is interrupted.
                                // Flush eagerly when we have some new content (>256 bytes)
                                if assistant_acc.len().saturating_sub(last_flush) > 256 {
                                    #[cfg(feature = "debug_io")]
                                    {
                                        if let Err(e) = std::fs::write(
                                            "target/ollama_assistant_acc.txt",
                                            &assistant_acc,
                                        ) {
                                            println!("[ollama][debug] partial write failed: {}", e);
                                        } else {
                                            last_flush = assistant_acc.len();
                                            println!("[ollama][debug] flushed {} bytes of assistant_acc to disk", last_flush);
                                        }
                                    }
                                }
                            }
                        }

                        // If we've accumulated something that looks like a JSON object, try to extract it
                        if let Some(obj) = extract_json_object(&assistant_acc) {
                            if self.early_exit_on_json {
                                let ttfb_ms = first_token_at
                                    .map(|ft| ft.duration_since(start).as_millis())
                                    .unwrap_or(0);
                                let total_ms = start.elapsed().as_millis();
                                println!(
                                    "[ollama] timing stream: ttfb={}ms, total={}ms (early-exit)",
                                    ttfb_ms, total_ms
                                );
                                return Ok(obj);
                            }
                            text = obj;
                            break;
                        }
                    }
                    Err(e) => {
                        // Non-fatal: keep trying until total timeout
                        if start.elapsed() > total_stream_timeout {
                            bail!("Streaming read timed out: {}", e);
                        }
                    }
                }

                if done_flag || start.elapsed() > total_stream_timeout {
                    break;
                }
            }

            // Ensure we always flush a final snapshot of assembled content even if small
            if !assistant_acc.is_empty() && assistant_acc.len() > last_flush {
                #[cfg(feature = "debug_io")]
                {
                    let _ = std::fs::write("target/ollama_assistant_acc.txt", &assistant_acc);
                }
            }

            if text.trim().is_empty() {
                // If streaming collected something but we couldn't parse JSON, try to salvage text.
                // Some Ollama variants deliver the assistant output as a quoted JSON string (i.e. a
                // JSON string literal containing another JSON object). Detect and unquote that.
                if !assistant_acc.trim().is_empty() {
                    let mut candidate = assistant_acc.clone();
                    // If the assistant_acc looks like a quoted JSON string, unescape it
                    let t = candidate.trim();
                    if t.starts_with('"') && t.ends_with('"') {
                        if let Ok(unq) = serde_json::from_str::<String>(t) {
                            candidate = unq;
                        }
                    }

                    // Debug: persist the assembled assistant output to a file for offline inspection
                    println!(
                        "[ollama][debug] assembled assistant_acc length = {}",
                        candidate.len()
                    );
                    if candidate.len() > 500 {
                        println!(
                            "[ollama][debug] assembled snippet (start): {}",
                            &candidate[..200.min(candidate.len())]
                        );
                        println!(
                            "[ollama][debug] assembled snippet (end): {}",
                            &candidate[candidate.len().saturating_sub(200)..]
                        );
                    } else {
                        println!("[ollama][debug] assembled content: {}", candidate);
                    }
                    #[cfg(feature = "debug_io")]
                    {
                        if let Err(e) =
                            std::fs::write("target/ollama_assistant_acc.txt", &candidate)
                        {
                            println!(
                                "[ollama][debug] failed to write assistant_acc to file: {}",
                                e
                            );
                        } else {
                            println!("[ollama][debug] wrote assembled assistant_acc to target/ollama_assistant_acc.txt");
                        }
                    }

                    if let Some(obj) = extract_last_json_object(&candidate)
                        .or_else(|| extract_json_object(&candidate))
                    {
                        println!("[ollama][debug] extracted JSON object (len={})", obj.len());
                        if self.early_exit_on_json {
                            let ttfb_ms = first_token_at
                                .map(|ft| ft.duration_since(start).as_millis())
                                .unwrap_or(0);
                            let total_ms = start.elapsed().as_millis();
                            println!("[ollama] timing stream: ttfb={}ms, total={}ms (early-exit salvage)", ttfb_ms, total_ms);
                            return Ok(obj);
                        }
                        text = obj;
                    } else {
                        text = strip_code_fences(&candidate);
                    }
                } else if let Ok(s) = std::str::from_utf8(&buf.as_bytes()) {
                    if let Some(obj) = extract_json_object(s) {
                        if self.early_exit_on_json {
                            let ttfb_ms = first_token_at
                                .map(|ft| ft.duration_since(start).as_millis())
                                .unwrap_or(0);
                            let total_ms = start.elapsed().as_millis();
                            println!(
                                "[ollama] timing stream: ttfb={}ms, total={}ms (early-exit buffer)",
                                ttfb_ms, total_ms
                            );
                            return Ok(obj);
                        }
                        text = obj;
                    } else {
                        text = strip_code_fences(s);
                    }
                }
            }

            // If we’re here, we didn’t early-exit; log final timings for stream path
            let ttfb_ms = first_token_at
                .map(|ft| ft.duration_since(start).as_millis())
                .unwrap_or(0);
            let total_ms = start.elapsed().as_millis();
            println!(
                "[ollama] timing stream: ttfb={}ms, total={}ms",
                ttfb_ms, total_ms
            );

            if text.trim().is_empty() {
                bail!("Ollama chat did not return a usable response within timeout. Check `ollama ps` and model readiness.");
            }
        }

        // `text` now contains the response body retrieved via retry/backoff above.

        // Try parsing as JSON value to extract common shapes
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&text) {
            // 1) { "response": "..." }
            if let Some(resp) = v.get("response").and_then(|r| r.as_str()) {
                return Ok(resp.to_string());
            }

            // 2) { "message": { "content": "..." } }
            if let Some(msg) = v.get("message") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                    // If content contains a fenced code block or an embedded JSON object,
                    // try to extract and return that JSON directly to simplify downstream parsing.
                    if let Some(obj) = extract_json_object(content) {
                        return Ok(obj);
                    }
                    let stripped = strip_code_fences(content);
                    return Ok(stripped);
                }
            }

            // 3) { "choices": [ { "message": { "content": "..." } } ] }
            if let Some(choices) = v.get("choices").and_then(|c| c.as_array()) {
                if let Some(first) = choices.get(0) {
                    if let Some(msg) = first.get("message") {
                        if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                            return Ok(content.to_string());
                        }
                    }
                }
            }
        }

        // If no recognized JSON shape matched, return the raw text as a last resort
        Ok(text)
    }
}

/// A simple local HTTP LLM client that can work with any OpenAI-compatible API
/// This includes local services like text-generation-webui, LocalAI, etc.
#[cfg(feature = "ollama")]
pub struct LocalHttpClient {
    pub url: String,
    pub model: String,
    pub api_key: Option<String>,
}

#[cfg(feature = "ollama")]
impl LocalHttpClient {
    /// Create a new client for OpenAI-compatible APIs (including local services)
    pub fn new(url: String, model: String) -> Self {
        Self {
            url,
            model,
            api_key: None,
        }
    }

    /// Create a client with API key (for services that require it)
    pub fn with_api_key(url: String, model: String, api_key: String) -> Self {
        Self {
            url,
            model,
            api_key: Some(api_key),
        }
    }
}

#[cfg(feature = "ollama")]
#[async_trait::async_trait]
impl LlmClient for LocalHttpClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        #[derive(serde::Serialize, serde::Deserialize)]
        struct Message {
            role: String,
            content: String,
        }

        #[derive(serde::Serialize)]
        struct Req {
            model: String,
            messages: Vec<Message>,
            max_tokens: u32,
            temperature: f32,
        }

        #[derive(serde::Deserialize)]
        struct Choice {
            message: Message,
        }

        #[derive(serde::Deserialize)]
        struct Resp {
            choices: Vec<Choice>,
        }

        let body = Req {
            model: self.model.clone(),
            messages: vec![Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            max_tokens: 2048,
            temperature: 0.1, // Low temperature for more consistent JSON output
        };

        let mut request = reqwest::Client::new()
            .post(format!("{}/v1/chat/completions", self.url))
            .header("Content-Type", "application/json")
            .timeout(std::time::Duration::from_secs(60));

        if let Some(api_key) = &self.api_key {
            request = request.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = request
            .json(&body)
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send request to local LLM: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            bail!("Local LLM API returned error status {}: {}", status, text);
        }

        let parsed: Resp = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse local LLM response: {}", e))?;

        if parsed.choices.is_empty() {
            bail!("Local LLM returned no choices");
        }

        Ok(parsed.choices[0].message.content.clone())
    }
}

/// Build an instruction that forces JSON output conforming to PlanIntent.
pub fn build_prompt(snap: &WorldSnapshot, reg: &ToolRegistry) -> String {
    let tool_list = reg
        .tools
        .iter()
        .map(|t| format!(" - {} {:?}", t.name, t.args))
        .collect::<Vec<_>>()
        .join("\n");
    let schema = r#"
Strict JSON schema:
{
  "plan_id": "string",
  "steps": [
     {"act":"MoveTo","x":INT,"y":INT} |
     {"act":"Throw","item":"smoke|grenade","x":INT,"y":INT} |
     {"act":"CoverFire","target_id":INT,"duration":FLOAT} |
     {"act":"Revive","ally_id":INT}
  ]
}
Return ONLY JSON with no commentary.
"#;
    format!(
        r#"You are an AI game companion planner. Convert the world snapshot into a legal action plan.
Use ONLY allowed tools and arguments. Do not exceed cooldown or LOS checks (the engine will validate).
Allowed tools:
{tools}

Snapshot (redacted):
{snap}

{schema}"#,
        tools = tool_list,
        snap = serde_json::to_string_pretty(snap).unwrap(),
        schema = schema
    )
}

/// Parse and validate that the produced steps are in the allowed registry (structural check).
pub fn parse_llm_plan(json_text: &str, reg: &ToolRegistry) -> Result<PlanIntent> {
    // Try direct parse first
    if let Ok(plan) = serde_json::from_str::<PlanIntent>(json_text.trim()) {
        validate_plan(&plan, reg)?;
        return Ok(plan);
    }

    // Strip common code fences and try again
    let cleaned = strip_code_fences(json_text);
    // If there's fenced JSON like ```json { ... } ``` try to extract inner JSON first
    if let Some(fenced) = extract_json_from_fenced(json_text) {
        if let Ok(plan) = serde_json::from_str::<PlanIntent>(fenced.trim()) {
            validate_plan(&plan, reg)?;
            return Ok(plan);
        }
        // try cleaned fenced
        if let Some(inner_clean) = extract_json_from_fenced(&cleaned) {
            if let Ok(plan) = serde_json::from_str::<PlanIntent>(inner_clean.trim()) {
                validate_plan(&plan, reg)?;
                return Ok(plan);
            }
        }
    }
    if let Ok(plan) = serde_json::from_str::<PlanIntent>(cleaned.as_str()) {
        validate_plan(&plan, reg)?;
        return Ok(plan);
    }

    // Attempt to extract the last JSON object from the text and parse it
    if let Some(obj) = extract_last_json_object(&cleaned) {
        if let Ok(plan) = serde_json::from_str::<PlanIntent>(obj.trim()) {
            validate_plan(&plan, reg)?;
            return Ok(plan);
        }
    }

    // Attempt to extract the first JSON object from the text and parse it
    if let Some(obj) = extract_json_object(&cleaned) {
        if let Ok(plan) = serde_json::from_str::<PlanIntent>(obj.trim()) {
            validate_plan(&plan, reg)?;
            return Ok(plan);
        }
    }

    // Try to obtain a JSON Value from several candidates for tolerant extraction
    let v_opt: Option<serde_json::Value> =
        serde_json::from_str::<serde_json::Value>(cleaned.as_str())
            .ok()
            .or_else(|| {
                extract_last_json_object(json_text).and_then(|s| serde_json::from_str(&s).ok())
            })
            .or_else(|| {
                extract_last_json_object(&cleaned).and_then(|s| serde_json::from_str(&s).ok())
            })
            .or_else(|| extract_json_object(&cleaned).and_then(|s| serde_json::from_str(&s).ok()));

    if let Some(v) = &v_opt {
        // Try to locate a nested JSON inside `message.content` or `response`
        if let Some(msg) = v.get("message") {
            if let Some(content) = msg.get("content").and_then(|c| c.as_str()) {
                // Try to parse content as JSON directly
                if let Ok(plan) = serde_json::from_str::<PlanIntent>(content.trim()) {
                    validate_plan(&plan, reg)?;
                    return Ok(plan);
                }
                // Try to extract JSON from the content string
                if let Some(obj2) = extract_json_object(content) {
                    if let Ok(plan) = serde_json::from_str::<PlanIntent>(obj2.trim()) {
                        validate_plan(&plan, reg)?;
                        return Ok(plan);
                    }
                }
            }
        }

        if let Some(resp_txt) = v.get("response").and_then(|r| r.as_str()) {
            if let Some(obj2) = extract_json_object(resp_txt) {
                if let Ok(plan) = serde_json::from_str::<PlanIntent>(obj2.trim()) {
                    validate_plan(&plan, reg)?;
                    return Ok(plan);
                }
            }
        }
    }

    // Coerce tolerant PlanIntent: accept alternative keys and ensure steps exist
    if let Some(v) = v_opt {
        let plan_id = (|| {
            // common accepted keys
            let candidates = [
                "plan_id",
                "plan_eid",
                "id",
                "plan_no",
                "plan_num",
                "planNumber",
                "plan_n°",
                "plan_n",
            ];
            for &k in &candidates {
                if let Some(vv) = v.get(k) {
                    if let Some(s) = vv.as_str() {
                        return Some(s.to_string());
                    }
                }
            }

            // Try to find any key that, when normalized, matches "planid" or similar
            if let Some(obj) = v.as_object() {
                for (k, vv) in obj.iter() {
                    let norm: String = k
                        .chars()
                        .filter(|c| c.is_alphanumeric())
                        .collect::<String>()
                        .to_lowercase();
                    if norm == "planid"
                        || norm == "plann"
                        || norm == "planno"
                        || norm == "plannumber"
                    {
                        if let Some(s) = vv.as_str() {
                            return Some(s.to_string());
                        }
                    }
                }
            }

            None
        })();

        let plan_id = match plan_id {
            Some(id) => id,
            None => return Err(anyhow::anyhow!("No valid plan_id found in LLM response")),
        };

        let steps_val = v
            .get("steps")
            .cloned()
            .unwrap_or(serde_json::Value::Array(vec![]));
        let steps_json = serde_json::to_string(&steps_val)?;
        let steps: Vec<ActionStep> = serde_json::from_str(&steps_json)?;

        let plan = PlanIntent { plan_id, steps };
        validate_plan(&plan, reg)?;
        return Ok(plan);
    }

    Err(anyhow::anyhow!(
        "Failed to parse LLM plan from text (no valid JSON object found)"
    ))
}

fn strip_code_fences(s: &str) -> String {
    // Remove triple backtick code fences and return inner content if found
    if let Some(start) = s.find("```") {
        if let Some(end_rel) = s[start + 3..].find("```") {
            let inner = &s[start + 3..start + 3 + end_rel];
            return inner.trim().to_string();
        }
    }
    s.to_string()
}

fn validate_plan(plan: &PlanIntent, reg: &ToolRegistry) -> Result<()> {
    for s in &plan.steps {
        match s {
            ActionStep::MoveTo { .. } => {
                if !reg.tools.iter().any(|t| t.name == "MoveTo") {
                    bail!("LLM used disallowed tool MoveTo");
                }
            }
            ActionStep::Throw { .. } => {
                if !reg.tools.iter().any(|t| t.name == "Throw") {
                    bail!("LLM used disallowed tool Throw");
                }
            }
            ActionStep::CoverFire { .. } => {
                if !reg.tools.iter().any(|t| t.name == "CoverFire") {
                    bail!("LLM used disallowed tool CoverFire");
                }
            }
            ActionStep::Revive { .. } => {
                if !reg.tools.iter().any(|t| t.name == "Revive") {
                    bail!("LLM used disallowed tool Revive");
                }
            }
            // Phase 7: For new tools, validate generically
            // TODO: Add specific validation for each tool type
            _ => {
                // Generic validation for Phase 7 tools - check if tool is registered
                // More specific validation can be added later per tool
            }
        }
    }
    Ok(())
}

/// Maximum allowed coordinate value for plan sanitization
const MAX_COORD_BOUND: i32 = 100;

/// Sanitize and validate plan for safety
pub fn sanitize_plan(
    plan: &mut PlanIntent,
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
) -> Result<()> {
    // Remove any steps that exceed bounds or use invalid targets
    plan.steps.retain(|step| match step {
        ActionStep::MoveTo { x, y, speed: _ } => {
            // Check bounds (example: within 100 units)
            (x.abs() <= MAX_COORD_BOUND && y.abs() <= MAX_COORD_BOUND)
                && reg.tools.iter().any(|t| t.name == "MoveTo")
        }
        ActionStep::Throw { item, x, y } => {
            // Check item is allowed
            matches!(item.as_str(), "smoke" | "grenade")
                && (x.abs() <= MAX_COORD_BOUND && y.abs() <= MAX_COORD_BOUND)
                && reg.tools.iter().any(|t| t.name == "Throw")
        }
        ActionStep::CoverFire {
            target_id,
            duration,
        } => {
            // Check target exists and duration reasonable
            snap.enemies.iter().any(|e| e.id == *target_id)
                && *duration > 0.0
                && *duration <= 10.0
                && reg.tools.iter().any(|t| t.name == "CoverFire")
        }
        ActionStep::Revive { ally_id: _ } => {
            // Check ally exists (simplified: allow any ally for now, or validate against known ally IDs)
            reg.tools.iter().any(|t| t.name == "Revive")
        }
        // Phase 7: Accept all new tool types (validation happens in execution layer)
        _ => true,
    });
    Ok(())
}

/// Attempt to extract the first JSON object from a text blob by finding a balanced
/// '{' ... '}' region. Returns `Some(String)` if a balanced JSON-like block is found.
fn extract_json_object(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut start = None;
    let mut depth: i32 = 0;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'{' {
            if start.is_none() {
                start = Some(i);
            }
            depth += 1;
        } else if b == b'}' {
            if depth > 0 {
                depth -= 1;
                if depth == 0 {
                    if let Some(s_idx) = start {
                        // safe to slice because indices are on byte boundaries for UTF-8 ASCII braces
                        if let Ok(sub) = std::str::from_utf8(&bytes[s_idx..=i]) {
                            return Some(sub.to_string());
                        }
                    }
                }
            }
        }
    }
    None
}

/// Extract the last balanced JSON object in a string, if any.
fn extract_last_json_object(s: &str) -> Option<String> {
    let bytes = s.as_bytes();
    let mut start: Option<usize> = None;
    let mut depth: i32 = 0;
    let mut last_range: Option<(usize, usize)> = None;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'{' {
            if start.is_none() {
                start = Some(i);
            }
            depth += 1;
        } else if b == b'}' {
            if depth > 0 {
                depth -= 1;
                if depth == 0 {
                    if let Some(s_idx) = start {
                        last_range = Some((s_idx, i));
                        // continue scanning to prefer the last complete object
                        start = None;
                    }
                }
            }
        }
    }
    if let Some((s_idx, e_idx)) = last_range {
        if let Ok(sub) = std::str::from_utf8(&bytes[s_idx..=e_idx]) {
            return Some(sub.to_string());
        }
    }
    None
}

/// Find JSON inside fenced code blocks (```json ... ``` or ``` ... ```) and return inner content
fn extract_json_from_fenced(s: &str) -> Option<String> {
    // look for ```json first
    if let Some(start) = s.find("```json") {
        if let Some(end_rel) = s[start + 7..].find("```") {
            let inner = &s[start + 7..start + 7 + end_rel];
            return Some(inner.trim().to_string());
        }
    }
    // fallback to any ``` block
    if let Some(start) = s.find("```") {
        if let Some(end_rel) = s[start + 3..].find("```") {
            let inner = &s[start + 3..start + 3 + end_rel];
            return Some(inner.trim().to_string());
        }
    }
    None
}

pub mod ab_testing;
pub mod backpressure;
pub mod circuit_breaker;
pub mod production_hardening;
pub mod rate_limiter;
pub mod retry;
pub mod scheduler;
pub mod telemetry;
pub mod tool_guard;

// Phi-3 Medium Q4 integration (optional, requires --features phi3)
// DEPRECATED: Migrated to Hermes 2 Pro (October 2025)
#[cfg(feature = "phi3")]
pub mod phi3;

// Phi-3 via Ollama (DEPRECATED - use hermes2pro_ollama instead)
#[cfg(feature = "ollama")]
pub mod phi3_ollama;

// Hermes 2 Pro via Ollama (RECOMMENDED - 75-85% success rate vs 40-50% Phi-3)
#[cfg(feature = "ollama")]
pub mod hermes2pro_ollama;

// Prompt engineering for game AI
pub mod prompts;

// Phase 7: Enhanced prompt templates with tool vocabulary
pub mod prompt_template;

// Phase 7: Enhanced plan parser with hallucination detection
pub mod plan_parser;

// Phase 7: Multi-tier fallback system
pub mod fallback_system;

// Prompt compression utilities (Week 5 Action 22)
pub mod compression;
pub mod few_shot;

/// Generate a plan using LLM with Phase 7 multi-tier fallback system
/// 
/// This function uses a 4-tier fallback chain:
/// 1. Full LLM with all 37 tools
/// 2. Simplified LLM with 10 most common tools
/// 3. Heuristic rule-based planning
/// 4. Emergency safe default (Scan + Wait)
///
/// Cache integration is preserved for performance.
pub async fn plan_from_llm(
    client: &dyn LlmClient,
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
) -> PlanSource {
    #[cfg(feature = "llm_cache")]
    {
        // Build cache key for fast lookup
        let prompt = build_prompt(snap, reg);
        let tool_names: Vec<&str> = reg.tools.iter().map(|t| t.name.as_str()).collect();
        let cache_key = PromptKey::new(
            &prompt,
            "default", // TODO: Extract from client
            0.7,       // TODO: Extract temperature from client
            &tool_names,
        );
        
        // Check cache first
        if let Some((cached_plan, _decision)) = GLOBAL_CACHE.get(&cache_key) {
            #[cfg(feature = "debug_io")]
            eprintln!("[plan_from_llm] Cache HIT - returning cached plan: {}", cached_plan.plan.plan_id);
            
            return PlanSource::Llm(cached_plan.plan);
        }
        
        #[cfg(feature = "debug_io")]
        eprintln!("[plan_from_llm] Cache MISS - calling fallback orchestrator");
    }
    
    // Cache miss or disabled - use Phase 7 multi-tier fallback
    use crate::fallback_system::FallbackOrchestrator;
    
    let orchestrator = FallbackOrchestrator::new();
    let result = orchestrator.plan_with_fallback(client, snap, reg).await;
    
    #[cfg(feature = "debug_io")]
    eprintln!(
        "[plan_from_llm] Fallback orchestrator succeeded at tier {} after {} attempts ({} ms)",
        result.tier.as_str(),
        result.attempts.len(),
        result.total_duration_ms
    );
    
    // Cache successful LLM plans (Tier 1 or Tier 2)
    #[cfg(feature = "llm_cache")]
    {
        if result.tier <= fallback_system::FallbackTier::SimplifiedLlm {
            let prompt = build_prompt(snap, reg);
            let tool_names: Vec<&str> = reg.tools.iter().map(|t| t.name.as_str()).collect();
            let cache_key = PromptKey::new(&prompt, "default", 0.7, &tool_names);
            let cached_plan = CachedPlan {
                plan: result.plan.clone(),
                created_at: std::time::Instant::now(),
                tokens_saved: estimate_tokens(&prompt),
            };
            GLOBAL_CACHE.put(cache_key, cached_plan);
            
            #[cfg(feature = "debug_io")]
            eprintln!("[plan_from_llm] Cached new plan: {}", result.plan.plan_id);
        }
    }
    
    // Return appropriate PlanSource based on tier
    match result.tier {
        fallback_system::FallbackTier::FullLlm | fallback_system::FallbackTier::SimplifiedLlm => {
            PlanSource::Llm(result.plan)
        }
        fallback_system::FallbackTier::Heuristic | fallback_system::FallbackTier::Emergency => {
            PlanSource::Fallback {
                plan: result.plan,
                reason: format!(
                    "Used {} tier after {} attempts",
                    result.tier.as_str(),
                    result.attempts.len()
                ),
            }
        }
    }
}

/// Estimate token count from prompt (rough approximation: 4 chars per token)
fn estimate_tokens(prompt: &str) -> u32 {
    (prompt.len() / 4) as u32
}

/// Fallback heuristic plan when LLM fails - simple move towards objective or enemies
pub fn fallback_heuristic_plan(snap: &WorldSnapshot, reg: &ToolRegistry) -> PlanIntent {
    let mut steps = Vec::new();

    // If there's an objective, try to move towards it (simplified)
    if let Some(obj) = &snap.objective {
        if obj == "extract" {
            // Move towards player if far
            let dist = ((snap.me.pos.x - snap.player.pos.x).abs()
                + (snap.me.pos.y - snap.player.pos.y).abs()) as i32;
            if dist > 3 && reg.tools.iter().any(|t| t.name == "move_to") {
                steps.push(ActionStep::MoveTo {
                    x: snap.player.pos.x,
                    y: snap.player.pos.y,
                    speed: None,
                });
            }
        }
    }

    // If enemies nearby, provide cover fire
    if !snap.enemies.is_empty() && reg.tools.iter().any(|t| t.name == "cover_fire") {
        let enemy = &snap.enemies[0];
        steps.push(ActionStep::CoverFire {
            target_id: enemy.id,
            duration: 2.0,
        });
    }

    PlanIntent {
        plan_id: "heuristic-fallback".to_string(),
        steps,
    }
}

/// Get cache statistics (if caching is enabled)
#[cfg(feature = "llm_cache")]
pub fn get_cache_stats() -> cache::CacheStats {
    GLOBAL_CACHE.stats()
}

#[cfg(not(feature = "llm_cache"))]
pub fn get_cache_stats() -> () {
    ()
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{
        CompanionState, Constraints, EnemyState, IVec2, PlayerState, ToolSpec, WorldSnapshot,
    };

    fn create_test_registry() -> ToolRegistry {
        ToolRegistry {
            tools: vec![
                ToolSpec {
                    name: "MoveTo".into(),
                    args: [("x", "i32"), ("y", "i32")]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                },
                ToolSpec {
                    name: "Throw".into(),
                    args: [("item", "enum[smoke,grenade]"), ("x", "i32"), ("y", "i32")]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                },
                ToolSpec {
                    name: "CoverFire".into(),
                    args: [("target_id", "u32"), ("duration", "f32")]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                },
            ],
            constraints: Constraints {
                enforce_cooldowns: true,
                enforce_los: true,
                enforce_stamina: true,
            },
        }
    }

    fn create_test_world_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 1.0,
            player: PlayerState {
                hp: 100,
                pos: IVec2 { x: 2, y: 2 },
                stance: "stand".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 30,
                cooldowns: Default::default(),
                morale: 0.9,
                pos: IVec2 { x: 3, y: 2 },
            },
            enemies: vec![EnemyState {
                id: 99,
                pos: IVec2 { x: 12, y: 2 },
                hp: 60,
                cover: "low".into(),
                last_seen: 1.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: Some("extract".into()),
        }
    }

    // Mock client that returns custom JSON
    struct TestLlmClient {
        response: String,
    }

    #[async_trait::async_trait]
    impl LlmClient for TestLlmClient {
        async fn complete(&self, _prompt: &str) -> Result<String> {
            Ok(self.response.clone())
        }
    }

    #[test]
    fn test_build_prompt() {
        let snap = create_test_world_snapshot();
        let reg = create_test_registry();

        let prompt = build_prompt(&snap, &reg);

        // Check that prompt contains expected elements
        assert!(prompt.contains("AI game companion planner"));
        assert!(prompt.contains("move_to"));
        assert!(prompt.contains("throw"));
        assert!(prompt.contains("cover_fire"));
        assert!(prompt.contains("Return ONLY JSON"));
        assert!(prompt.contains("\"t\": 1.0"));
    }

    #[test]
    fn test_parse_llm_plan_valid() {
        let reg = create_test_registry();
        let json = r#"{
            "plan_id": "test-plan",
            "steps": [
                {"act": "MoveTo", "x": 5, "y": 5},
                {"act": "Throw", "item": "smoke", "x": 7, "y": 3}
            ]
        }"#;

        let result = parse_llm_plan(json, &reg);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.plan_id, "test-plan");
        assert_eq!(plan.steps.len(), 2);
    }

    #[test]
    fn test_parse_llm_plan_invalid_json() {
        let reg = create_test_registry();
        let invalid_json = "not json";

        let result = parse_llm_plan(invalid_json, &reg);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llm_plan_disallowed_tool() {
        let mut reg = create_test_registry();
        // Remove the MoveTo tool to test disallowed tool detection
        reg.tools.retain(|t| t.name != "MoveTo");

        let json = r#"{
            "plan_id": "test-plan",
            "steps": [
                {"act": "MoveTo", "x": 5, "y": 5}
            ]
        }"#;

        let result = parse_llm_plan(json, &reg);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("disallowed tool MoveTo"));
    }

    #[tokio::test]
    async fn test_mock_llm_client() {
        let client = MockLlm;
        let result = client.complete("test prompt").await;

        assert!(result.is_ok());
        let response = result.unwrap();

        // Should be valid JSON
        assert!(serde_json::from_str::<serde_json::Value>(&response).is_ok());
        assert!(response.contains("llm-mock"));
    }

    #[tokio::test]
    async fn test_plan_from_llm_success() {
        let snap = create_test_world_snapshot();
        let reg = create_test_registry();
        let client = MockLlm;

        let result = plan_from_llm(&client, &snap, &reg).await;
        match result {
            PlanSource::Llm(plan) => {
                assert_eq!(plan.plan_id, "llm-mock");
                assert!(!plan.steps.is_empty());
            }
            PlanSource::Fallback { .. } => panic!("Expected LLM plan"),
        }
    }

    #[tokio::test]
    async fn test_plan_from_llm_invalid_response() {
        let snap = create_test_world_snapshot();
        let reg = create_test_registry();
        let client = TestLlmClient {
            response: "invalid json".to_string(),
        };

        let result = plan_from_llm(&client, &snap, &reg).await;
        // Should fallback to heuristic or emergency plan (Phase 7 multi-tier fallback)
        match result {
            PlanSource::Llm(_) => panic!("Expected fallback"),
            PlanSource::Fallback { plan, .. } => {
                // Phase 7: Plans are generated with UUID-based IDs
                assert!(
                    plan.plan_id.starts_with("heuristic-") || plan.plan_id.starts_with("emergency-"),
                    "Expected heuristic or emergency plan, got: {}",
                    plan.plan_id
                );
            }
        }
    }

    #[tokio::test]
    async fn test_plan_from_llm_disallowed_tool() {
        let snap = create_test_world_snapshot();
        let mut reg = create_test_registry();
        // Remove all tools
        reg.tools.clear();

        let client = MockLlm;

        let result = plan_from_llm(&client, &snap, &reg).await;
        // Should fallback to heuristic or emergency plan (which uses no tools when registry is empty)
        match result {
            PlanSource::Llm(_) => panic!("Expected fallback"),
            PlanSource::Fallback { plan, .. } => {
                // Phase 7: Plans are generated with UUID-based IDs
                assert!(
                    plan.plan_id.starts_with("heuristic-") || plan.plan_id.starts_with("emergency-"),
                    "Expected heuristic or emergency plan, got: {}",
                    plan.plan_id
                );
                // Note: Emergency plan always returns Scan + Wait even with empty registry
                // Heuristic with empty registry returns empty steps
            }
        }
    }

    #[test]
    fn test_parse_llm_plan_empty_steps() {
        let reg = create_test_registry();
        let json = r#"{
            "plan_id": "empty-plan",
            "steps": []
        }"#;

        let result = parse_llm_plan(json, &reg);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.plan_id, "empty-plan");
        assert!(plan.steps.is_empty());
    }

    #[test]
    fn test_parse_llm_plan_all_action_types() {
        let mut reg = create_test_registry();
        // Add revive tool
        reg.tools.push(ToolSpec {
            name: "revive".into(),
            args: [("ally_id", "u32")]
                .into_iter()
                .map(|(k, v)| (k.into(), v.into()))
                .collect(),
        });

        let json = r#"{
            "plan_id": "all-actions",
            "steps": [
                {"act": "MoveTo", "x": 5, "y": 5},
                {"act": "Throw", "item": "grenade", "x": 7, "y": 3},
                {"act": "CoverFire", "target_id": 42, "duration": 3.5},
                {"act": "Revive", "ally_id": 123}
            ]
        }"#;

        let result = parse_llm_plan(json, &reg);
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.steps.len(), 4);
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn test_ollama_client_creation() {
        let client = OllamaClient {
            url: "http://localhost:11434".to_string(),
            model: "llama2".to_string(),
        };
        assert_eq!(client.url, "http://localhost:11434");
        assert_eq!(client.model, "llama2");
    }

    #[cfg(feature = "ollama")]
    #[test]
    fn test_local_http_client_creation() {
        let client = LocalHttpClient::new(
            "http://localhost:5000".to_string(),
            "test-model".to_string(),
        );
        assert_eq!(client.url, "http://localhost:5000");
        assert_eq!(client.model, "test-model");
        assert!(client.api_key.is_none());

        let client_with_key = LocalHttpClient::with_api_key(
            "http://localhost:5000".to_string(),
            "test-model".to_string(),
            "test-key".to_string(),
        );
        assert_eq!(client_with_key.api_key, Some("test-key".to_string()));
    }

    #[test]
    fn test_prompt_includes_constraints() {
        let snap = create_test_world_snapshot();
        let reg = create_test_registry();

        let prompt = build_prompt(&snap, &reg);

        // Check that prompt mentions validation
        assert!(prompt.contains("engine will validate"));
        assert!(prompt.contains("Do not exceed cooldown or LOS checks"));
    }

    #[test]
    fn test_parse_llm_plan_malformed_step() {
        let reg = create_test_registry();
        let json = r#"{
            "plan_id": "malformed",
            "steps": [
                {"act": "MoveTo", "x": "not_a_number", "y": 5}
            ]
        }"#;

        let result = parse_llm_plan(json, &reg);
        // Should fail to parse due to type mismatch
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llm_plan_missing_plan_id() {
        let reg = create_test_registry();
        let json = r#"{
            "steps": [
                {"act": "MoveTo", "x": 5, "y": 5}
            ]
        }"#;

        let result = parse_llm_plan(json, &reg);
        // Should fail due to missing plan_id
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_llm_plan_from_fenced_json() {
        let reg = create_test_registry();
        let text = r#"```json
        {
          "plan_id": "fenced",
          "steps": [ {"act":"MoveTo","x":1,"y":2} ]
        }
        ```"#;
        let res = parse_llm_plan(text, &reg).unwrap();
        assert_eq!(res.plan_id, "fenced");
        assert_eq!(res.steps.len(), 1);
    }

    #[test]
    fn test_parse_llm_plan_with_nonstandard_planid_key() {
        let reg = create_test_registry();
        // Simulate the model returning plan_n° key
        let text = r#"{
          "plan_n°": "7",
          "steps": [ {"act":"MoveTo","x":3,"y":4} ]
        }"#;
        let res = parse_llm_plan(text, &reg).unwrap();
        assert_eq!(res.plan_id, "7");
        assert_eq!(res.steps.len(), 1);
    }

    #[test]
    fn test_parse_llm_plan_from_envelope_message_content() {
        let reg = create_test_registry();
        // Simulate Ollama envelope with nested message.content carrying JSON
        let text = r#"{
          "message": { "role": "assistant", "content": "{\n  \"plan_id\": \"env\",\n  \"steps\": [ {\"act\":\"MoveTo\",\"x\":5,\"y\":6 } ]\n}" }
        }"#;
        let res = parse_llm_plan(text, &reg).unwrap();
        assert_eq!(res.plan_id, "env");
        assert_eq!(res.steps.len(), 1);
    }

    #[test]
    fn test_parse_llm_plan_pick_last_json_object() {
        let reg = create_test_registry();
        // Test picking the last complete JSON object
        let text = r#"{"plan_id": "first", "steps": []}
{"plan_id": "final", "steps": [ {"act":"MoveTo","x":1,"y":1} ] }"#;
        let res = parse_llm_plan(text, &reg).unwrap();
        assert_eq!(res.plan_id, "final");
        assert_eq!(res.steps.len(), 1);
    }
}

