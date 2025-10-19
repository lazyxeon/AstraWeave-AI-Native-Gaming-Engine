use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// PolyHaven API client
pub struct PolyHavenClient {
    client: reqwest::Client,
    base_url: String,
}

/// Response from /files/{asset_id} endpoint
/// Structure varies by asset type:
/// Textures: { "Diffuse": { "2k": { "png": { "url": "...", "size": 123 } } } }
/// HDRIs: { "hdri": { "2k": { "exr": { "url": "...", "size": 123 } } }, "tonemapped": { "url": "..." } }
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FilesResponse {
    #[serde(flatten)]
    pub maps: HashMap<String, serde_json::Value>,
}

/// File information (URL, size, MD5)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileInfo {
    pub url: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub md5: String,
}

/// Response from /info/{asset_id} endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InfoResponse {
    pub name: String,
    #[serde(default)]
    pub categories: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub download_count: u64,
}

/// Resolved asset with download URLs
#[derive(Debug, Clone)]
pub struct ResolvedAsset {
    pub id: String,
    pub kind: String,
    pub resolution: String,
    pub urls: HashMap<String, String>,
    pub info: InfoResponse,
}

impl PolyHavenClient {
    /// Create new client
    pub fn new() -> Result<Self> {
        Self::new_with_base_url("https://api.polyhaven.com")
    }

    /// Create new client with custom base URL (for testing)
    pub fn new_with_base_url(base_url: &str) -> Result<Self> {
        let client = reqwest::Client::builder()
            .user_agent("AstraWeave-Assets/0.1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
        })
    }

    /// Fetch files metadata for an asset
    pub async fn get_files(&self, asset_id: &str) -> Result<FilesResponse> {
        let url = format!("{}/files/{}", self.base_url, asset_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to fetch files for asset: {}", asset_id))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "PolyHaven API error for {}: HTTP {}",
                asset_id,
                response.status()
            ));
        }

        let files: FilesResponse = response
            .json()
            .await
            .context("Failed to parse files response")?;

        Ok(files)
    }

    /// Fetch info metadata for an asset
    pub async fn get_info(&self, asset_id: &str) -> Result<InfoResponse> {
        let url = format!("{}/info/{}", self.base_url, asset_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context(format!("Failed to fetch info for asset: {}", asset_id))?;

        if !response.status().is_success() {
            return Err(anyhow!(
                "PolyHaven API error for {}: HTTP {}",
                asset_id,
                response.status()
            ));
        }

        let info: InfoResponse = response
            .json()
            .await
            .context("Failed to parse info response")?;

        Ok(info)
    }

    /// Resolve texture asset with fallback resolutions
    pub async fn resolve_texture(
        &self,
        asset_id: &str,
        requested_res: &str,
        requested_maps: &[String],
    ) -> Result<ResolvedAsset> {
        let files = self.get_files(asset_id).await?;
        let info = self.get_info(asset_id).await?;

        // Resolution fallback priority: requested → 2k → 1k → whatever's available
        let fallback_order = self.resolution_fallback_order(requested_res);
        
        // Extract URLs for requested maps
        let mut urls = HashMap::new();
        let mut selected_res = requested_res.to_string();

        for map_name in requested_maps {
            // Get PolyHaven map names (capitalized)
            let ph_map_names = self.polyhaven_map_names(map_name);
            let mut found = false;

            // Try each alternative name
            for ph_name in &ph_map_names {
                if let Some(map_value) = files.maps.get(*ph_name) {
                    // Parse nested JSON structure: { "2k": { "png": { "url": "..." } } }
                    if let Some(resolutions_map) = map_value.as_object() {
                        // Try each resolution in fallback order
                        for res in &fallback_order {
                            if let Some(formats_value) = resolutions_map.get(*res) {
                                if let Some(formats_map) = formats_value.as_object() {
                                    // Prefer PNG > EXR > JPG
                                    for format in &["png", "exr", "jpg"] {
                                        if let Some(file_info_value) = formats_map.get(*format) {
                                            if let Ok(file_info) =
                                                serde_json::from_value::<FileInfo>(file_info_value.clone())
                                            {
                                                urls.insert(map_name.clone(), file_info.url);
                                                selected_res = res.to_string();
                                                found = true;
                                                break;
                                            }
                                        }
                                    }

                                    if found {
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    if found {
                        break;
                    }
                }
            }

            if !found {
                eprintln!(
                    "⚠️  Map '{}' not available for {} (tried: {:?})",
                    map_name, asset_id, ph_map_names
                );
            }
        }

        if urls.is_empty() {
            return Err(anyhow!(
                "No maps found for texture {}",
                asset_id
            ));
        }

        Ok(ResolvedAsset {
            id: asset_id.to_string(),
            kind: "texture".to_string(),
            resolution: selected_res,
            urls,
            info,
        })
    }

    /// Resolve HDRI asset
    pub async fn resolve_hdri(&self, asset_id: &str, requested_res: &str) -> Result<ResolvedAsset> {
        let files = self.get_files(asset_id).await?;
        let info = self.get_info(asset_id).await?;

        let fallback_order = self.resolution_fallback_order(requested_res);
        let mut hdri_url = None;
        let mut selected_res = requested_res.to_string();

        // HDRIs have structure: { "hdri": { "2k": { "exr": {...}, "hdr": {...} } } }
        if let Some(hdri_value) = files.maps.get("hdri") {
            if let Some(resolutions_map) = hdri_value.as_object() {
                for res in &fallback_order {
                    if let Some(formats_value) = resolutions_map.get(*res) {
                        if let Some(formats_map) = formats_value.as_object() {
                            // Prefer EXR > HDR for HDRIs
                            for format in &["exr", "hdr"] {
                                if let Some(file_info_value) = formats_map.get(*format) {
                                    if let Ok(file_info) =
                                        serde_json::from_value::<FileInfo>(file_info_value.clone())
                                    {
                                        hdri_url = Some(file_info.url);
                                        selected_res = res.to_string();
                                        break;
                                    }
                                }
                            }

                            if hdri_url.is_some() {
                                break;
                            }
                        }
                    }
                }
            }
        }

        match hdri_url {
            Some(url) => {
                let mut urls = HashMap::new();
                urls.insert("hdri".to_string(), url);

                Ok(ResolvedAsset {
                    id: asset_id.to_string(),
                    kind: "hdri".to_string(),
                    resolution: selected_res,
                    urls,
                    info,
                })
            }
            None => Err(anyhow!("No HDRI file found for asset: {}", asset_id)),
        }
    }

    /// Resolve model asset
    pub async fn resolve_model(
        &self,
        asset_id: &str,
        requested_res: &str,
        format: &str,
    ) -> Result<ResolvedAsset> {
        let files = self.get_files(asset_id).await?;
        let info = self.get_info(asset_id).await?;

        let fallback_order = self.resolution_fallback_order(requested_res);
        let mut model_urls = HashMap::new();
        let mut selected_res = requested_res.to_string();

        // Look for model formats: gltf, glb, blend, fbx
        for (map_name, map_value) in &files.maps {
            if map_name.to_lowercase().contains(format)
                || map_name.to_lowercase().contains("gltf")
                || map_name.to_lowercase().contains("glb")
                || map_name.to_lowercase().contains("blend")
            {
                if let Some(resolutions_map) = map_value.as_object() {
                    for res in &fallback_order {
                        if let Some(formats_value) = resolutions_map.get(*res) {
                            if let Some(formats_map) = formats_value.as_object() {
                                // Prefer GLB > GLTF > BLEND
                                for model_format in &["glb", "gltf", "blend"] {
                                    if let Some(file_info_value) = formats_map.get(*model_format) {
                                        if let Ok(file_info) =
                                            serde_json::from_value::<FileInfo>(file_info_value.clone())
                                        {
                                            model_urls.insert("model".to_string(), file_info.url);
                                            selected_res = res.to_string();
                                            break;
                                        }
                                    }
                                }

                                if !model_urls.is_empty() {
                                    break;
                                }
                            }
                        }
                    }
                }

                if !model_urls.is_empty() {
                    break;
                }
            }
        }

        if model_urls.is_empty() {
            return Err(anyhow!(
                "No model file found for asset {} (format: {})",
                asset_id,
                format
            ));
        }

        Ok(ResolvedAsset {
            id: asset_id.to_string(),
            kind: "model".to_string(),
            resolution: selected_res,
            urls: model_urls,
            info,
        })
    }

    /// Resolution fallback order
    fn resolution_fallback_order(&self, requested: &str) -> Vec<&str> {
        match requested {
            "8k" => vec!["8k", "4k", "2k", "1k"],
            "4k" => vec!["4k", "2k", "1k", "8k"],
            "2k" => vec!["2k", "1k", "4k", "8k"],
            "1k" => vec!["1k", "2k", "4k", "8k"],
            _ => vec!["2k", "1k", "4k", "8k"], // Default to 2k
        }
    }

    /// PolyHaven API map names (capitalized)
    fn polyhaven_map_names(&self, user_map_name: &str) -> Vec<&str> {
        match user_map_name {
            "albedo" => vec!["Diffuse", "diff", "diffuse", "Color"],
            "normal" => vec!["nor_gl", "nor_dx", "Normal"],
            "roughness" => vec!["Rough", "Roughness"],
            "metallic" => vec!["Metal", "Metallic", "Metalness"],
            "ao" => vec!["AO", "ao", "ambient_occlusion"],
            "height" | "displacement" => vec!["Displacement", "disp", "Bump", "Height"],
            _ => vec![],
        }
    }
}

impl Default for PolyHavenClient {
    fn default() -> Self {
        Self::new().expect("Failed to create PolyHaven client")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = PolyHavenClient::new();
        assert!(client.is_ok(), "Should create client successfully");
    }

    #[tokio::test]
    #[cfg(feature = "live-api-tests")]
    async fn test_real_api_call() {
        let client = PolyHavenClient::default();

        // Test with known asset
        let result = client.get_info("aerial_rocks_02").await;
        assert!(result.is_ok(), "Failed to fetch real asset info");

        let info = result.unwrap();
        assert_eq!(info.name, "Aerial Rocks 02");
    }
}
