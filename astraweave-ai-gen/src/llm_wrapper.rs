use crate::schema::{MeshGenerationParams, TextureGenerationParams};
use anyhow::Result;
use astraweave_llm::LlmClient;
use std::sync::Arc;

#[derive(Clone)]
pub struct AiClient {
    client: Arc<dyn LlmClient>,
}

impl AiClient {
    pub fn new(client: Arc<dyn LlmClient>) -> Self {
        Self { client }
    }

    pub async fn generate_texture_params(&self, prompt: &str) -> Result<TextureGenerationParams> {
        let system_prompt = r#"You are an AI assistant that generates texture parameters for a procedural generation engine.
Output ONLY valid JSON matching this schema:
{
    "prompt": "string",
    "width": number (default 512),
    "height": number (default 512),
    "scale": number (default 10.0),
    "roughness": number (default 0.5),
    "seed": number (optional),
    "usage": "albedo" | "normal" | "height"
}
Do not include markdown formatting or explanations."#;

        let full_prompt = format!("{}\n\nUser Request: {}", system_prompt, prompt);
        let response = self.client.complete(&full_prompt).await?;

        // Clean up response (remove markdown code blocks if present)
        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let params: TextureGenerationParams = serde_json::from_str(clean_json)?;
        Ok(params)
    }

    pub async fn generate_mesh_params(&self, prompt: &str) -> Result<MeshGenerationParams> {
        let system_prompt = r#"You are an AI assistant that generates mesh parameters for a procedural generation engine.
Output ONLY valid JSON matching this schema:
{
    "prompt": "string",
    "size": number (default 10.0),
    "resolution": number (default 32),
    "height_scale": number (default 1.0),
    "seed": number (optional)
}
Do not include markdown formatting or explanations."#;

        let full_prompt = format!("{}\n\nUser Request: {}", system_prompt, prompt);
        let response = self.client.complete(&full_prompt).await?;

        let clean_json = response
            .trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim();

        let params: MeshGenerationParams = serde_json::from_str(clean_json)?;
        Ok(params)
    }
}
