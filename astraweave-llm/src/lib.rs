use anyhow::{bail, Result};
use astraweave_core::{ActionStep, PlanIntent, ToolRegistry, WorldSnapshot};

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

        let body = Req {
            model: &self.model,
            prompt,
            stream: false,
        };

        let client = reqwest::Client::new();
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

        Ok(parsed.response)
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
    let plan: PlanIntent = serde_json::from_str(json_text.trim())?;
    // basic allowlist check
    for s in &plan.steps {
        match s {
            ActionStep::MoveTo { .. } => {
                if !reg.tools.iter().any(|t| t.name == "move_to") {
                    bail!("LLM used disallowed tool MoveTo");
                }
            }
            ActionStep::Throw { .. } => {
                if !reg.tools.iter().any(|t| t.name == "throw") {
                    bail!("LLM used disallowed tool Throw");
                }
            }
            ActionStep::CoverFire { .. } => {
                if !reg.tools.iter().any(|t| t.name == "cover_fire") {
                    bail!("LLM used disallowed tool CoverFire");
                }
            }
            ActionStep::Revive { .. } => {
                if !reg.tools.iter().any(|t| t.name == "revive") {
                    bail!("LLM used disallowed tool Revive");
                }
            }
        }
    }
    Ok(plan)
}

/// End-to-end: build prompt → LLM → parse → PlanIntent.
pub async fn plan_from_llm(
    client: &dyn LlmClient,
    snap: &WorldSnapshot,
    reg: &ToolRegistry,
) -> Result<PlanIntent> {
    let prompt = build_prompt(snap, reg);
    let text = client.complete(&prompt).await?;
    let plan = parse_llm_plan(&text, reg)?;
    Ok(plan)
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
                    name: "move_to".into(),
                    args: [("x", "i32"), ("y", "i32")]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                },
                ToolSpec {
                    name: "throw".into(),
                    args: [("item", "enum[smoke,grenade]"), ("x", "i32"), ("y", "i32")]
                        .into_iter()
                        .map(|(k, v)| (k.into(), v.into()))
                        .collect(),
                },
                ToolSpec {
                    name: "cover_fire".into(),
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
        // Remove the move_to tool
        reg.tools.retain(|t| t.name != "move_to");

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
        assert!(result.is_ok());

        let plan = result.unwrap();
        assert_eq!(plan.plan_id, "llm-mock");
        assert!(!plan.steps.is_empty());
    }

    #[tokio::test]
    async fn test_plan_from_llm_invalid_response() {
        let snap = create_test_world_snapshot();
        let reg = create_test_registry();
        let client = TestLlmClient {
            response: "invalid json".to_string(),
        };

        let result = plan_from_llm(&client, &snap, &reg).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_plan_from_llm_disallowed_tool() {
        let snap = create_test_world_snapshot();
        let mut reg = create_test_registry();
        // Remove all tools
        reg.tools.clear();

        let client = MockLlm;

        let result = plan_from_llm(&client, &snap, &reg).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("disallowed tool"));
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
}
