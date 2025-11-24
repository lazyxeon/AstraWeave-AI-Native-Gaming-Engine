use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextureGenerationParams {
    pub prompt: String,
    pub width: u32,
    pub height: u32,
    pub scale: f64,
    pub roughness: f64,
    pub seed: Option<u32>,
    pub usage: String, // "albedo", "normal", "height"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshGenerationParams {
    pub prompt: String,
    pub size: f32,
    pub resolution: u32,
    pub height_scale: f32,
    pub seed: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AiAssetRequest {
    Texture(TextureGenerationParams),
    Mesh(MeshGenerationParams),
}
