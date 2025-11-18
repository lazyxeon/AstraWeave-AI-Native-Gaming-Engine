/*!
# Embedding Clients

Trait-based abstraction for different embedding model backends.
Supports mock clients for testing, local models via ONNX/Candle, and remote APIs.
*/

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Main trait for embedding clients
#[async_trait]
pub trait EmbeddingClient: Send + Sync {
    /// Embed a single text into a vector
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Embed multiple texts in a batch (more efficient)
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;

    /// Get the dimensionality of embeddings from this client
    fn dimensions(&self) -> usize;

    /// Get model information
    fn model_info(&self) -> ModelInfo;
}

/// Information about an embedding model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    /// Model name/identifier
    pub name: String,
    /// Model version
    pub version: String,
    /// Vector dimensions
    pub dimensions: usize,
    /// Maximum input tokens
    pub max_tokens: usize,
    /// Model description
    pub description: String,
}

/// Mock embedding client for testing and development
pub struct MockEmbeddingClient {
    dimensions: usize,
    /// Deterministic mapping from text to embeddings for testing
    cache: Arc<RwLock<HashMap<String, Vec<f32>>>>,
}

impl Default for MockEmbeddingClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MockEmbeddingClient {
    /// Create a new mock client with specified dimensions
    pub fn new() -> Self {
        Self::with_dimensions(384)
    }

    /// Create a mock client with custom dimensions
    pub fn with_dimensions(dimensions: usize) -> Self {
        Self {
            dimensions,
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Generate a deterministic embedding from text hash
    fn generate_mock_embedding(&self, text: &str) -> Vec<f32> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let seed = hasher.finish();

        // Use the hash as a seed for deterministic random generation
        let mut rng = SmallRng::seed_from_u64(seed);

        let mut embedding = Vec::with_capacity(self.dimensions);
        for _ in 0..self.dimensions {
            embedding.push(rng.random_range(-1.0..1.0));
        }

        // Normalize to unit vector for cosine similarity
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for value in &mut embedding {
                *value /= magnitude;
            }
        }

        embedding
    }
}

use rand::{rngs::SmallRng, Rng, SeedableRng};

#[async_trait]
impl EmbeddingClient for MockEmbeddingClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(embedding) = cache.get(text) {
                return Ok(embedding.clone());
            }
        }

        // Generate new embedding
        let embedding = self.generate_mock_embedding(text);

        // Cache it
        {
            let mut cache = self.cache.write().await;
            cache.insert(text.to_string(), embedding.clone());
        }

        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            results.push(self.embed(text).await?);
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "mock-embeddings".to_string(),
            version: "1.0.0".to_string(),
            dimensions: self.dimensions,
            max_tokens: 512,
            description: "Mock embedding client for testing".to_string(),
        }
    }
}

/// Local embedding client using ONNX Runtime
#[cfg(feature = "onnx")]
pub struct OnnxEmbeddingClient {
    dimensions: usize,
    model_path: String,
    session: Arc<ort::Session>,
    tokenizer: Arc<tokenizers::Tokenizer>,
}

#[cfg(feature = "onnx")]
impl OnnxEmbeddingClient {
    /// Create a new ONNX embedding client
    pub async fn new(model_path: String, tokenizer_path: String) -> Result<Self> {
        // Load ONNX model
        let session = ort::Session::builder()?
            .with_optimization_level(ort::GraphOptimizationLevel::All)?
            .commit_from_file(&model_path)?;

        // Load tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Infer dimensions from model output
        let dimensions = 384; // Default for all-MiniLM-L6-v2

        Ok(Self {
            dimensions,
            model_path,
            session: Arc::new(session),
            tokenizer: Arc::new(tokenizer),
        })
    }

    /// Tokenize text for model input
    fn tokenize(&self, text: &str) -> Result<(Vec<i64>, Vec<i64>)> {
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|&id| id as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|&mask| mask as i64)
            .collect();

        Ok((input_ids, attention_mask))
    }

    /// Run inference on tokenized input
    fn run_inference(&self, input_ids: Vec<i64>, attention_mask: Vec<i64>) -> Result<Vec<f32>> {
        use ort::{Tensor, Value};

        // Create input tensors
        let input_ids_tensor =
            Tensor::from_array(([1, input_ids.len()], input_ids.into_boxed_slice()))?;
        let attention_mask_tensor =
            Tensor::from_array(([1, attention_mask.len()], attention_mask.into_boxed_slice()))?;

        // Run inference
        let outputs = self.session.run([
            Value::from(input_ids_tensor),
            Value::from(attention_mask_tensor),
        ])?;

        // Extract embeddings from output
        let output_tensor = &outputs[0];
        let embedding: Vec<f32> = output_tensor
            .try_extract_raw_tensor()?
            .as_slice()?
            .iter()
            .map(|&x| x as f32)
            .collect();

        Ok(embedding)
    }
}

#[cfg(feature = "onnx")]
#[async_trait]
impl EmbeddingClient for OnnxEmbeddingClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let (input_ids, attention_mask) = self.tokenize(text)?;
        let embedding = self.run_inference(input_ids, attention_mask)?;
        Ok(embedding)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        // For now, process sequentially
        // TODO: Implement true batching with padded inputs
        for text in texts {
            results.push(self.embed(text).await?);
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "onnx-local-embeddings".to_string(),
            version: "1.0.0".to_string(),
            dimensions: self.dimensions,
            max_tokens: 512,
            description: format!("Local ONNX embedding model at {}", self.model_path),
        }
    }
}

/// Candle-based embedding client for pure Rust inference
#[cfg(feature = "candle")]
pub struct CandleEmbeddingClient {
    dimensions: usize,
    model: Arc<parking_lot::RwLock<candle_transformers::models::bert::BertModel>>,
    tokenizer: Arc<tokenizers::Tokenizer>,
    device: candle_core::Device,
}

#[cfg(feature = "candle")]
impl CandleEmbeddingClient {
    /// Create a new Candle embedding client
    pub async fn new(model_path: String, tokenizer_path: String) -> Result<Self> {
        use candle_core::{Device, Tensor};
        use candle_nn::VarBuilder;
        use candle_transformers::models::bert::{BertModel, Config};

        // Determine device (CPU/CUDA)
        let device = Device::cuda_if_available(0).unwrap_or(Device::Cpu);

        // Load tokenizer
        let tokenizer = tokenizers::Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| anyhow!("Failed to load tokenizer: {}", e))?;

        // Load model weights and config
        let api = hf_hub::api::tokio::Api::new()?;
        let repo = api.model("sentence-transformers/all-MiniLM-L6-v2".parse()?);
        let config_filename = repo.get("config.json").await?;
        let weights_filename = repo.get("pytorch_model.bin").await?;

        // Load configuration
        let config: Config = serde_json::from_slice(&std::fs::read(config_filename)?)?;
        let dimensions = config.hidden_size;

        // Load weights
        let weights = candle_core::safetensors::load(weights_filename, &device)?;
        let var_builder = VarBuilder::from_tensors(weights, candle_core::DType::F32, &device);

        // Create model
        let model = BertModel::load(&var_builder, &config)?;

        Ok(Self {
            dimensions,
            model: Arc::new(parking_lot::RwLock::new(model)),
            tokenizer: Arc::new(tokenizer),
            device,
        })
    }
}

#[cfg(feature = "candle")]
#[async_trait]
impl EmbeddingClient for CandleEmbeddingClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        use candle_core::Tensor;

        // Tokenize input
        let encoding = self
            .tokenizer
            .encode(text, true)
            .map_err(|e| anyhow!("Tokenization failed: {}", e))?;

        let input_ids: Vec<u32> = encoding.get_ids().to_vec();
        let attention_mask: Vec<u32> = encoding.get_attention_mask().to_vec();

        // Convert to tensors
        let input_ids = Tensor::new(&input_ids[..], &self.device)?.unsqueeze(0)?; // Add batch dimension
        let attention_mask = Tensor::new(&attention_mask[..], &self.device)?.unsqueeze(0)?; // Add batch dimension

        // Run forward pass
        let model = self.model.read();
        let output = model.forward(&input_ids, &attention_mask)?;

        // Mean pooling to get sentence embeddings
        let embeddings = output.mean(1)?; // Average over sequence length

        // Convert to Vec<f32>
        let embedding_vec: Vec<f32> = embeddings.to_vec1()?;

        Ok(embedding_vec)
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let mut results = Vec::with_capacity(texts.len());

        for text in texts {
            results.push(self.embed(text).await?);
        }

        Ok(results)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "candle-sentence-transformers".to_string(),
            version: "1.0.0".to_string(),
            dimensions: self.dimensions,
            max_tokens: 512,
            description: "Local Candle-based sentence transformer".to_string(),
        }
    }
}

/// Remote API embedding client (OpenAI-compatible)
#[cfg(feature = "http")]
pub struct RemoteEmbeddingClient {
    api_url: String,
    api_key: Option<String>,
    model: String,
    dimensions: usize,
    client: reqwest::Client,
}

#[cfg(feature = "http")]
impl RemoteEmbeddingClient {
    /// Create a new remote embedding client
    pub fn new(api_url: String, model: String, dimensions: usize) -> Self {
        Self {
            api_url,
            api_key: None,
            model,
            dimensions,
            client: reqwest::Client::new(),
        }
    }

    /// Create a client with API key
    pub fn with_api_key(
        api_url: String,
        model: String,
        dimensions: usize,
        api_key: String,
    ) -> Self {
        Self {
            api_url,
            api_key: Some(api_key),
            model,
            dimensions,
            client: reqwest::Client::new(),
        }
    }
}

#[cfg(feature = "http")]
#[async_trait]
impl EmbeddingClient for RemoteEmbeddingClient {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let embeddings = self.embed_batch(&[text.to_string()]).await?;
        Ok(embeddings.into_iter().next().unwrap_or_default())
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        #[derive(Serialize)]
        struct EmbeddingRequest<'a> {
            input: &'a [String],
            model: &'a str,
        }

        #[derive(Deserialize)]
        struct EmbeddingResponse {
            data: Vec<EmbeddingData>,
        }

        #[derive(Deserialize)]
        struct EmbeddingData {
            embedding: Vec<f32>,
        }

        let request = EmbeddingRequest {
            input: texts,
            model: &self.model,
        };

        let mut req = self
            .client
            .post(&format!("{}/embeddings", self.api_url))
            .header("Content-Type", "application/json")
            .json(&request);

        if let Some(api_key) = &self.api_key {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req.send().await?;

        if !response.status().is_success() {
            return Err(anyhow!("API request failed: {}", response.status()));
        }

        let response_data: EmbeddingResponse = response.json().await?;

        Ok(response_data
            .data
            .into_iter()
            .map(|d| d.embedding)
            .collect())
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    fn model_info(&self) -> ModelInfo {
        ModelInfo {
            name: self.model.clone(),
            version: "unknown".to_string(),
            dimensions: self.dimensions,
            max_tokens: 8192,
            description: format!("Remote embedding API: {}", self.api_url),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_embedding_client() {
        let client = MockEmbeddingClient::new();

        let text1 = "Hello world";
        let text2 = "Hello world"; // Same text
        let text3 = "Goodbye world"; // Different text

        let emb1 = client.embed(text1).await.unwrap();
        let emb2 = client.embed(text2).await.unwrap();
        let emb3 = client.embed(text3).await.unwrap();

        assert_eq!(emb1, emb2); // Same text should produce same embedding
        assert_ne!(emb1, emb3); // Different text should produce different embedding
        assert_eq!(emb1.len(), 384); // Default dimensions
    }

    #[tokio::test]
    async fn test_batch_embedding() {
        let client = MockEmbeddingClient::new();

        let texts = vec![
            "First text".to_string(),
            "Second text".to_string(),
            "Third text".to_string(),
        ];

        let embeddings = client.embed_batch(&texts).await.unwrap();

        assert_eq!(embeddings.len(), 3);
        assert_eq!(embeddings[0].len(), 384);

        // Verify they're different
        assert_ne!(embeddings[0], embeddings[1]);
        assert_ne!(embeddings[1], embeddings[2]);
    }

    #[tokio::test]
    async fn test_model_info() {
        let client = MockEmbeddingClient::with_dimensions(256);
        let info = client.model_info();

        assert_eq!(info.dimensions, 256);
        assert_eq!(info.name, "mock-embeddings");
    }

    // ============================================================================
    // Determinism Validation Tests (Sprint 1 Day 1 - Phase 8.7)
    // ============================================================================

    #[tokio::test]
    async fn test_mock_embedding_determinism_across_instances() {
        // Create two separate client instances
        let client1 = MockEmbeddingClient::new();
        let client2 = MockEmbeddingClient::new();

        let text = "Deterministic test text";

        // Generate embeddings from both clients
        let emb1 = client1.embed(text).await.unwrap();
        let emb2 = client2.embed(text).await.unwrap();

        // Should be identical (not relying on cache, separate instances)
        assert_eq!(
            emb1, emb2,
            "Same text should produce identical embeddings across separate client instances"
        );
    }

    #[tokio::test]
    async fn test_mock_embedding_determinism_batch_vs_single() {
        let client = MockEmbeddingClient::new();

        let text = "Batch determinism validation";

        // Generate via single call
        let single = client.embed(text).await.unwrap();

        // Clear cache to force regeneration
        client.cache.write().await.clear();

        // Generate via batch call
        let batch = client
            .embed_batch(&[text.to_string()])
            .await
            .unwrap()
            .into_iter()
            .next()
            .unwrap();

        // Should be identical
        assert_eq!(
            single, batch,
            "Same text should produce identical embeddings via single vs batch calls"
        );
    }

    #[tokio::test]
    async fn test_mock_embedding_unit_length_normalization() {
        let client = MockEmbeddingClient::new();

        let texts = vec!["Normalize me", "Test vector", "Unit length validation"];

        for text in texts {
            let embedding = client.embed(&text).await.unwrap();

            // Calculate magnitude
            let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();

            // Should be unit length (magnitude = 1.0)
            assert!(
                (magnitude - 1.0).abs() < 1e-5,
                "Embedding for '{}' should be unit length, got magnitude {}",
                text,
                magnitude
            );
        }
    }

    #[tokio::test]
    async fn test_mock_embedding_different_texts_different_embeddings() {
        // Create two separate client instances (no shared cache)
        let client1 = MockEmbeddingClient::new();
        let client2 = MockEmbeddingClient::new();

        let text_a = "This is text A";
        let text_b = "This is text B";

        // Generate from separate clients to avoid cache
        let emb_a = client1.embed(text_a).await.unwrap();
        let emb_b = client2.embed(text_b).await.unwrap();

        // Different texts should produce different embeddings
        assert_ne!(
            emb_a, emb_b,
            "Different texts should produce different embeddings (deterministically)"
        );

        // Verify consistency: same text from different clients should match
        let emb_a2 = client2.embed(text_a).await.unwrap();
        assert_eq!(
            emb_a, emb_a2,
            "Same text from different clients should match (determinism)"
        );
    }
}
