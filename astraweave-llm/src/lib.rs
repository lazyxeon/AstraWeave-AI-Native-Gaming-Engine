use anyhow::{Result, bail};
use astraweave_core::{WorldSnapshot, PlanIntent, ToolRegistry, ActionStep};
use serde::{Serialize, Deserialize};

/// Trait for LLM clients (mock, Ollama, etc).
#[async_trait::async_trait]
pub trait LlmClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Mock client (no model). Emits a basic plan using simple heuristics.
pub struct MockLlm;

#[async_trait::async_trait]
impl LlmClient for MockLlm {
    async fn complete(&self, prompt: &str) -> Result<String> {
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

#[cfg(feature="ollama")]
pub struct OllamaClient { pub url: String, pub model: String }

#[cfg(feature="ollama")]
#[async_trait::async_trait]
impl LlmClient for OllamaClient {
    async fn complete(&self, prompt: &str) -> Result<String> {
        #[derive(Serialize)] struct Req<'a> { model: &'a str, prompt: &'a str, stream: bool }
        #[derive(Deserialize)] struct Resp { response: String }
        let body = Req { model: &self.model, prompt, stream: false };
        let resp = reqwest::Client::new()
            .post(format!("{}/api/generate", self.url))
            .json(&body)
            .send().await?;
        let parsed: Resp = resp.json().await?;
        Ok(parsed.response)
    }
}

/// Build an instruction that forces JSON output conforming to PlanIntent.
pub fn build_prompt(snap: &WorldSnapshot, reg: &ToolRegistry) -> String {
    let tool_list = reg.tools.iter()
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
pub async fn plan_from_llm(client: &dyn LlmClient, snap: &WorldSnapshot, reg: &ToolRegistry) -> Result<PlanIntent> {
    let prompt = build_prompt(snap, reg);
    let text = client.complete(&prompt).await?;
    let plan = parse_llm_plan(&text, reg)?;
    Ok(plan)
}
