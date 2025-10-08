/*!
# LLM-Enhanced Dialogue System

Advanced dialogue system with LLM integration for dynamic conversation generation,
emotional intelligence, context awareness, and adaptive branching.
*/

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use regex::Regex;
use rand::{thread_rng, Rng};

// Base dialogue system
use crate::{DialogueGraph, DialogueNode, DialogueResponse};

// LLM Integration
use astraweave_llm::LlmClient;
use astraweave_context::{ConversationHistory, ContextConfig, Role, Message};
use astraweave_prompts::template::PromptTemplate;
use astraweave_prompts::engine::TemplateEngine;
use astraweave_prompts::context::PromptContext as TemplateContext;
use astraweave_rag::{RagPipeline, MemoryQuery, RetrievedMemory};
use astraweave_persona::{LlmPersonaManager, LlmPersona};
use astraweave_embeddings::{Memory, MemoryCategory};

/// Enhanced dialogue system with LLM capabilities
pub struct LlmDialogueSystem {
    /// LLM client for generation
    llm_client: Arc<dyn LlmClient>,
    
    /// RAG pipeline for context retrieval
    rag_pipeline: Arc<RwLock<RagPipeline>>,
    
    /// Template engine for prompt generation
    template_engine: Arc<RwLock<TemplateEngine>>,
    
    /// Active conversations
    conversations: Arc<RwLock<HashMap<String, ActiveConversation>>>,
    
    /// Dialogue configuration
    config: DialogueConfig,
    
    /// Performance metrics
    metrics: Arc<RwLock<DialogueMetrics>>,
}

/// Configuration for LLM dialogue system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueConfig {
    /// Maximum response length in tokens
    pub max_response_tokens: usize,
    
    /// Temperature for dialogue generation
    pub temperature: f32,
    
    /// Top-p for dialogue generation
    pub top_p: f32,
    
    /// Number of dialogue options to generate
    pub num_dialogue_options: usize,
    
    /// Emotional analysis settings
    pub emotion_analysis: EmotionAnalysisConfig,
    
    /// Context settings
    pub context_config: DialogueContextConfig,
    
    /// Branching settings
    pub branching_config: BranchingConfig,
    
    /// Quality control settings
    pub quality_control: QualityControlConfig,
}

impl Default for DialogueConfig {
    fn default() -> Self {
        Self {
            max_response_tokens: 256,
            temperature: 0.8,
            top_p: 0.9,
            num_dialogue_options: 3,
            emotion_analysis: EmotionAnalysisConfig::default(),
            context_config: DialogueContextConfig::default(),
            branching_config: BranchingConfig::default(),
            quality_control: QualityControlConfig::default(),
        }
    }
}

/// Configuration for emotional analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionAnalysisConfig {
    /// Enable sentiment analysis
    pub enable_sentiment_analysis: bool,
    
    /// Enable emotion detection
    pub enable_emotion_detection: bool,
    
    /// Enable empathy responses
    pub enable_empathy_responses: bool,
    
    /// Sentiment influence on responses
    pub sentiment_influence: f32,
    
    /// Emotional memory retention
    pub emotional_memory_retention: f32,
}

impl Default for EmotionAnalysisConfig {
    fn default() -> Self {
        Self {
            enable_sentiment_analysis: true,
            enable_emotion_detection: true,
            enable_empathy_responses: true,
            sentiment_influence: 0.3,
            emotional_memory_retention: 0.7,
        }
    }
}

/// Configuration for dialogue context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueContextConfig {
    /// Maximum conversation history to consider
    pub max_history_turns: usize,
    
    /// Number of relevant memories to retrieve
    pub max_relevant_memories: usize,
    
    /// Context window size in tokens
    pub context_window_size: usize,
    
    /// Enable conversation summarization
    pub enable_summarization: bool,
}

impl Default for DialogueContextConfig {
    fn default() -> Self {
        Self {
            max_history_turns: 10,
            max_relevant_memories: 5,
            context_window_size: 2048,
            enable_summarization: true,
        }
    }
}

/// Configuration for dialogue branching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BranchingConfig {
    /// Enable dynamic branching
    pub enable_dynamic_branching: bool,
    
    /// Branching probability threshold
    pub branching_threshold: f32,
    
    /// Maximum branch depth
    pub max_branch_depth: usize,
    
    /// Branch merge strategy
    pub merge_strategy: BranchMergeStrategy,
}

impl Default for BranchingConfig {
    fn default() -> Self {
        Self {
            enable_dynamic_branching: true,
            branching_threshold: 0.7,
            max_branch_depth: 5,
            merge_strategy: BranchMergeStrategy::Contextual,
        }
    }
}

/// Strategies for merging dialogue branches
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum BranchMergeStrategy {
    /// Merge based on context similarity
    Contextual,
    /// Merge based on emotional state
    Emotional,
    /// Merge based on persona consistency
    PersonaBased,
    /// No automatic merging
    None,
}

/// Configuration for quality control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControlConfig {
    /// Enable response validation
    pub enable_validation: bool,
    
    /// Minimum response quality score
    pub min_quality_score: f32,
    
    /// Enable profanity filtering
    pub enable_profanity_filter: bool,
    
    /// Enable repetition detection
    pub enable_repetition_detection: bool,
    
    /// Maximum retry attempts for poor quality
    pub max_retry_attempts: usize,
}

impl Default for QualityControlConfig {
    fn default() -> Self {
        Self {
            enable_validation: true,
            min_quality_score: 0.6,
            enable_profanity_filter: true,
            enable_repetition_detection: true,
            max_retry_attempts: 3,
        }
    }
}

/// An active conversation with context and state
#[derive(Debug, Clone)]
pub struct ActiveConversation {
    /// Conversation ID
    pub id: String,
    
    /// Participant personas
    pub participants: Vec<String>,
    
    /// Conversation history
    pub history: ConversationHistory,
    
    /// Current emotional state
    pub emotional_state: EmotionalState,
    
    /// Conversation metadata
    pub metadata: ConversationMetadata,
    
    /// Current dialogue graph (if using structured dialogue)
    pub current_graph: Option<DialogueGraph>,
    
    /// Current node in the graph
    pub current_node_id: Option<String>,
    
    /// Dynamic branching state
    pub branching_state: BranchingState,
}

/// Current emotional state of a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionalState {
    /// Overall sentiment (-1.0 to 1.0)
    pub sentiment: f32,
    
    /// Detected emotions with confidence scores
    pub emotions: HashMap<String, f32>,
    
    /// Emotional intensity (0.0 to 1.0)
    pub intensity: f32,
    
    /// Emotional valence (-1.0 to 1.0, negative to positive)
    pub valence: f32,
    
    /// Emotional arousal (0.0 to 1.0, calm to excited)
    pub arousal: f32,
}

impl Default for EmotionalState {
    fn default() -> Self {
        Self {
            sentiment: 0.0,
            emotions: HashMap::new(),
            intensity: 0.5,
            valence: 0.0,
            arousal: 0.5,
        }
    }
}

/// Metadata about a conversation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationMetadata {
    /// When the conversation started
    pub started_at: u64,
    
    /// Last activity timestamp
    pub last_activity: u64,
    
    /// Turn count
    pub turn_count: usize,
    
    /// Conversation topics
    pub topics: Vec<String>,
    
    /// Conversation quality metrics
    pub quality_metrics: QualityMetrics,
    
    /// Additional context
    pub context: HashMap<String, serde_json::Value>,
}

/// Quality metrics for a conversation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Average response quality (0.0 to 1.0)
    pub avg_response_quality: f32,
    
    /// Coherence score (0.0 to 1.0)
    pub coherence_score: f32,
    
    /// Engagement score (0.0 to 1.0)
    pub engagement_score: f32,
    
    /// Repetition penalty
    pub repetition_penalty: f32,
}

/// Dynamic branching state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BranchingState {
    /// Current branch ID
    pub current_branch: Option<String>,
    
    /// Available branches
    pub available_branches: Vec<DialogueBranch>,
    
    /// Branch history
    pub branch_history: Vec<String>,
    
    /// Branch probability scores
    pub branch_probabilities: HashMap<String, f32>,
}

/// A dynamic dialogue branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueBranch {
    /// Branch ID
    pub id: String,
    
    /// Branch condition/trigger
    pub condition: String,
    
    /// Branch probability
    pub probability: f32,
    
    /// Branch content
    pub content: DialogueBranchContent,
    
    /// Branch metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Content of a dialogue branch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DialogueBranchContent {
    /// Generated responses
    pub responses: Vec<GeneratedResponse>,
    
    /// Emotional context
    pub emotional_context: EmotionalState,
    
    /// Narrative impact
    pub narrative_impact: f32,
}

/// A generated dialogue response with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedResponse {
    /// The response text
    pub text: String,
    
    /// Quality score
    pub quality_score: f32,
    
    /// Emotional tone
    pub emotional_tone: EmotionalState,
    
    /// Confidence level
    pub confidence: f32,
    
    /// Generation metadata
    pub metadata: ResponseMetadata,
}

/// Metadata about response generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    /// Generation method used
    pub generation_method: GenerationMethod,
    
    /// Processing time in milliseconds
    pub processing_time_ms: f32,
    
    /// Number of retry attempts
    pub retry_attempts: usize,
    
    /// Context used for generation
    pub context_summary: String,
}

/// Methods used for response generation
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum GenerationMethod {
    /// Pure LLM generation
    LlmGeneration,
    
    /// Template-based generation
    TemplateBased,
    
    /// Hybrid approach
    Hybrid,
    
    /// Graph-based (traditional)
    GraphBased,
}

/// Performance metrics for the dialogue system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DialogueMetrics {
    pub total_conversations: u64,
    pub active_conversations: u64,
    pub total_responses_generated: u64,
    pub successful_generations: u64,
    pub failed_generations: u64,
    pub avg_response_time_ms: f32,
    pub avg_response_quality: f32,
    pub emotion_detection_accuracy: f32,
    pub conversation_satisfaction: f32,
}

impl LlmDialogueSystem {
    /// Create a new LLM dialogue system
    pub async fn new(
        llm_client: Arc<dyn LlmClient>,
        rag_pipeline: RagPipeline,
        config: DialogueConfig,
    ) -> Result<Self> {
        let mut template_engine = TemplateEngine::new();
        Self::setup_dialogue_templates(&mut template_engine).await?;
        
        Ok(Self {
            llm_client,
            rag_pipeline: Arc::new(RwLock::new(rag_pipeline)),
            template_engine: Arc::new(RwLock::new(template_engine)),
            conversations: Arc::new(RwLock::new(HashMap::new())),
            config,
            metrics: Arc::new(RwLock::new(DialogueMetrics::default())),
        })
    }
    
    /// Start a new conversation
    pub async fn start_conversation(
        &self,
        conversation_id: String,
        participants: Vec<String>,
        initial_context: Option<String>,
    ) -> Result<String> {
        let context_config = ContextConfig {
            max_tokens: self.config.context_config.context_window_size,
            sliding_window_size: self.config.context_config.max_history_turns,
            enable_summarization: self.config.context_config.enable_summarization,
            ..Default::default()
        };
        
        let history = ConversationHistory::with_llm_client(
            context_config,
            self.llm_client.clone()
        );
        
        let mut metadata = ConversationMetadata {
            started_at: chrono::Utc::now().timestamp() as u64,
            last_activity: chrono::Utc::now().timestamp() as u64,
            turn_count: 0,
            topics: Vec::new(),
            quality_metrics: QualityMetrics::default(),
            context: HashMap::new(),
        };
        
        // Add initial context if provided
        if let Some(context) = initial_context {
            metadata.context.insert("initial_context".to_string(), context.into());
        }
        
        let conversation = ActiveConversation {
            id: conversation_id.clone(),
            participants,
            history,
            emotional_state: EmotionalState::default(),
            metadata,
            current_graph: None,
            current_node_id: None,
            branching_state: BranchingState::default(),
        };
        
        {
            let mut conversations = self.conversations.write().await;
            conversations.insert(conversation_id.clone(), conversation);
        }
        
        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_conversations += 1;
            metrics.active_conversations += 1;
        }
        
        Ok(conversation_id)
    }
    
    /// Generate a response in a conversation
    pub async fn generate_response(
        &self,
        conversation_id: &str,
        speaker_persona: Option<&LlmPersona>,
        player_input: &str,
        context: Option<&str>,
    ) -> Result<GeneratedResponse> {
        let start_time = std::time::Instant::now();
        
        // Get conversation
        let mut conversation = {
            let conversations = self.conversations.read().await;
            conversations.get(conversation_id)
                .ok_or_else(|| anyhow!("Conversation not found: {}", conversation_id))?
                .clone()
        };
        
        // Add player input to history
        conversation.history.add_message(Role::User, player_input.to_string()).await?;
        
        // Analyze player input emotion
        let player_emotion = self.analyze_emotion(player_input).await?;
        
        // Retrieve relevant memories
        let relevant_memories = {
            let rag = self.rag_pipeline.read().await;
            let query = MemoryQuery::text(player_input);
            rag.retrieve_with_query(&query, self.config.context_config.max_relevant_memories).await?
        };
        
        // Generate response with retries for quality
        let mut response = None;
        let mut attempts = 0;
        
        while response.is_none() && attempts < self.config.quality_control.max_retry_attempts {
            attempts += 1;
            
            match self.generate_single_response(
                &conversation,
                speaker_persona,
                player_input,
                context,
                &player_emotion,
                &relevant_memories,
            ).await {
                Ok(generated) => {
                    if self.validate_response_quality(&generated).await? {
                        response = Some(generated);
                    }
                }
                Err(e) => {
                    if attempts >= self.config.quality_control.max_retry_attempts {
                        return Err(e);
                    }
                    // Continue to retry
                }
            }
        }
        
        let response = response.ok_or_else(|| anyhow!("Failed to generate quality response after {} attempts", attempts))?;
        
        // Add response to conversation history
        conversation.history.add_message(Role::Assistant, response.text.clone()).await?;
        
        // Update conversation state
        conversation.metadata.last_activity = chrono::Utc::now().timestamp() as u64;
        conversation.metadata.turn_count += 1;
        conversation.emotional_state = self.blend_emotional_states(&conversation.emotional_state, &response.emotional_tone);
        
        // Store interaction as memory
        let memory_text = format!("Player: {} | Assistant: {}", player_input, response.text);
        {
            let mut rag = self.rag_pipeline.write().await;
            rag.add_memory(memory_text).await?;
        }
        
        // Update conversation in storage
        {
            let mut conversations = self.conversations.write().await;
            conversations.insert(conversation_id.to_string(), conversation);
        }
        
        // Update metrics
        let processing_time = start_time.elapsed().as_millis() as f32;
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_responses_generated += 1;
            metrics.successful_generations += 1;
            
            let total = metrics.successful_generations as f32;
            metrics.avg_response_time_ms = 
                (metrics.avg_response_time_ms * (total - 1.0) + processing_time) / total;
            metrics.avg_response_quality = 
                (metrics.avg_response_quality * (total - 1.0) + response.quality_score) / total;
        }
        
        Ok(response)
    }
    
    /// Generate a single response (internal method)
    async fn generate_single_response(
        &self,
        conversation: &ActiveConversation,
        speaker_persona: Option<&LlmPersona>,
        player_input: &str,
        context: Option<&str>,
        player_emotion: &EmotionalState,
        relevant_memories: &[RetrievedMemory],
    ) -> Result<GeneratedResponse> {
        // Build context-aware prompt
        let prompt = self.build_dialogue_prompt(
            conversation,
            speaker_persona,
            player_input,
            context,
            player_emotion,
            relevant_memories,
        ).await?;
        
        // Generate response using LLM
        let raw_response = self.llm_client.complete(&prompt).await?;
        
        // Clean and validate response
        let cleaned_response = self.clean_response(&raw_response);
        
        // Analyze response emotion
        let response_emotion = self.analyze_emotion(&cleaned_response).await?;
        
        // Calculate quality score
        let quality_score = self.calculate_response_quality(&cleaned_response, conversation, player_input).await?;
        
        // Calculate confidence based on various factors
        let confidence = self.calculate_confidence(&cleaned_response, quality_score, relevant_memories.len());
        
        Ok(GeneratedResponse {
            text: cleaned_response,
            quality_score,
            emotional_tone: response_emotion,
            confidence,
            metadata: ResponseMetadata {
                generation_method: GenerationMethod::LlmGeneration,
                processing_time_ms: 0.0, // Will be set by caller
                retry_attempts: 0, // Will be set by caller
                context_summary: self.summarize_context(conversation, relevant_memories).await,
            },
        })
    }
    
    /// Build a dialogue prompt with all context
    async fn build_dialogue_prompt(
        &self,
        conversation: &ActiveConversation,
        speaker_persona: Option<&LlmPersona>,
        player_input: &str,
        context: Option<&str>,
        player_emotion: &EmotionalState,
        relevant_memories: &[RetrievedMemory],
    ) -> Result<String> {
        let mut template_context = TemplateContext::new();
        
        // Basic dialogue info
        template_context.set("player_input", player_input);
        template_context.set("turn_count", &conversation.metadata.turn_count.to_string());
        
        // Speaker persona information
        if let Some(persona) = speaker_persona {
            template_context.set("speaker.name", &persona.base.voice); // Using voice as name
            template_context.set("speaker.tone", &persona.base.tone);
            template_context.set("speaker.humor", &persona.base.humor);
            template_context.set("speaker.backstory", &persona.base.backstory);
            template_context.set("speaker.mood", &format!("{:?}", persona.personality_state.emotional_state));
            template_context.set("speaker.confidence", &persona.personality_state.confidence.to_string());
        }
        
        // Player emotional state
        template_context.set("player_emotion.sentiment", &player_emotion.sentiment.to_string());
        template_context.set("player_emotion.intensity", &player_emotion.intensity.to_string());
        
        // Conversation history
        let conversation_context = conversation.history.get_context(1024).await?;
        template_context.set("conversation_history", &conversation_context);
        
        // Relevant memories
        if !relevant_memories.is_empty() {
            let memory_texts: Vec<String> = relevant_memories
                .iter()
                .map(|m| format!("- {}", m.memory.text))
                .collect();
            template_context.set("relevant_memories", &memory_texts.join("\n"));
        }
        
        // Additional context
        if let Some(ctx) = context {
            template_context.set("additional_context", ctx);
        }
        
        // Current emotional state of conversation
        template_context.set("conversation_sentiment", &conversation.emotional_state.sentiment.to_string());
        
        // Use template engine to generate prompt
        let template_engine = self.template_engine.read().await;
        template_engine.render("dialogue_response", &template_context)
    }
    
    /// Analyze emotion in text
    async fn analyze_emotion(&self, text: &str) -> Result<EmotionalState> {
        if !self.config.emotion_analysis.enable_emotion_detection {
            return Ok(EmotionalState::default());
        }
        
        // Simple emotion analysis (could be enhanced with dedicated models)
        let positive_words = ["happy", "joy", "excited", "great", "amazing", "wonderful", "love", "like"];
        let negative_words = ["sad", "angry", "hate", "terrible", "awful", "disappointed", "frustrated"];
        let neutral_words = ["okay", "fine", "normal", "maybe", "perhaps"];
        
        let text_lower = text.to_lowercase();
        let words: Vec<&str> = text_lower.split_whitespace().collect();
        
        let positive_count = positive_words.iter().filter(|&&word| words.contains(&word)).count() as f32;
        let negative_count = negative_words.iter().filter(|&&word| words.contains(&word)).count() as f32;
        let total_emotional_words = positive_count + negative_count;
        
        let sentiment = if total_emotional_words > 0.0 {
            (positive_count - negative_count) / total_emotional_words
        } else {
            0.0
        };
        
        let intensity = (total_emotional_words / words.len() as f32).min(1.0);
        let valence = sentiment;
        let arousal = intensity;
        
        // Detect specific emotions (simplified)
        let mut emotions = HashMap::new();
        if positive_count > 0.0 {
            emotions.insert("joy".to_string(), positive_count / words.len() as f32);
        }
        if negative_count > 0.0 {
            emotions.insert("sadness".to_string(), negative_count / words.len() as f32);
        }
        
        Ok(EmotionalState {
            sentiment,
            emotions,
            intensity,
            valence,
            arousal,
        })
    }
    
    /// Clean and format response text
    fn clean_response(&self, raw_response: &str) -> String {
        // Remove common LLM artifacts
        let cleaned = raw_response
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty() && !line.starts_with("Assistant:") && !line.starts_with("AI:"))
            .collect::<Vec<_>>()
            .join(" ");
        
        // Remove excessive punctuation
        let re = Regex::new(r"[.]{3,}").unwrap();
        let cleaned = re.replace_all(&cleaned, "...");
        
        // Ensure proper sentence ending
        let cleaned = if !cleaned.ends_with('.') && !cleaned.ends_with('!') && !cleaned.ends_with('?') {
            format!("{}.", cleaned)
        } else {
            cleaned.to_string()
        };
        
        cleaned
    }
    
    /// Validate response quality
    async fn validate_response_quality(&self, response: &GeneratedResponse) -> Result<bool> {
        if !self.config.quality_control.enable_validation {
            return Ok(true);
        }
        
        // Check minimum quality score
        if response.quality_score < self.config.quality_control.min_quality_score {
            return Ok(false);
        }
        
        // Check for repetition
        if self.config.quality_control.enable_repetition_detection {
            if self.detect_repetition(&response.text) {
                return Ok(false);
            }
        }
        
        // Check for profanity (basic check)
        if self.config.quality_control.enable_profanity_filter {
            if self.contains_profanity(&response.text) {
                return Ok(false);
            }
        }
        
        Ok(true)
    }
    
    /// Calculate response quality score
    async fn calculate_response_quality(&self, response: &str, conversation: &ActiveConversation, input: &str) -> Result<f32> {
        let mut quality_score = 0.7; // Base score
        
        // Length appropriateness
        let word_count = response.split_whitespace().count();
        if word_count > 5 && word_count < 100 {
            quality_score += 0.1;
        }
        
        // Relevance to input (simple keyword matching)
        let input_words: Vec<&str> = input.to_lowercase().split_whitespace().collect();
        let response_words: Vec<&str> = response.to_lowercase().split_whitespace().collect();
        let common_words = input_words.iter().filter(|word| response_words.contains(word)).count();
        let relevance = common_words as f32 / input_words.len().max(1) as f32;
        quality_score += relevance * 0.2;
        
        // Coherence (simple check for complete sentences)
        if response.contains('.') || response.contains('!') || response.contains('?') {
            quality_score += 0.1;
        }
        
        quality_score.min(1.0)
    }
    
    /// Calculate confidence score
    fn calculate_confidence(&self, response: &str, quality_score: f32, memory_count: usize) -> f32 {
        let mut confidence = quality_score;
        
        // Boost confidence if we have relevant memories
        if memory_count > 0 {
            confidence += (memory_count as f32 * 0.05).min(0.2);
        }
        
        // Reduce confidence for very short responses
        if response.split_whitespace().count() < 3 {
            confidence -= 0.2;
        }
        
        confidence.clamp(0.0, 1.0)
    }
    
    /// Detect repetitive content
    fn detect_repetition(&self, text: &str) -> bool {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.len() < 4 {
            return false;
        }
        
        // Check for repeated phrases
        for window_size in 2..=4 {
            for i in 0..=(words.len() - window_size * 2) {
                let phrase1 = &words[i..i + window_size];
                for j in (i + window_size)..=(words.len() - window_size) {
                    let phrase2 = &words[j..j + window_size];
                    if phrase1 == phrase2 {
                        return true;
                    }
                }
            }
        }
        
        false
    }
    
    /// Basic profanity check
    fn contains_profanity(&self, text: &str) -> bool {
        let profanity_words = ["damn", "hell", "shit", "fuck"]; // Basic list
        let text_lower = text.to_lowercase();
        profanity_words.iter().any(|&word| text_lower.contains(word))
    }
    
    /// Blend two emotional states
    fn blend_emotional_states(&self, state1: &EmotionalState, state2: &EmotionalState) -> EmotionalState {
        let blend_factor = 0.3; // How much to blend in the new state
        
        EmotionalState {
            sentiment: state1.sentiment * (1.0 - blend_factor) + state2.sentiment * blend_factor,
            emotions: state2.emotions.clone(), // Use newer emotions
            intensity: state1.intensity * (1.0 - blend_factor) + state2.intensity * blend_factor,
            valence: state1.valence * (1.0 - blend_factor) + state2.valence * blend_factor,
            arousal: state1.arousal * (1.0 - blend_factor) + state2.arousal * blend_factor,
        }
    }
    
    /// Summarize context for metadata
    async fn summarize_context(&self, conversation: &ActiveConversation, memories: &[RetrievedMemory]) -> String {
        format!(
            "Turn {}, {} memories, sentiment: {:.2}",
            conversation.metadata.turn_count,
            memories.len(),
            conversation.emotional_state.sentiment
        )
    }
    
    /// Set up template engine with dialogue templates
    async fn setup_dialogue_templates(engine: &mut TemplateEngine) -> Result<()> {
    // Main dialogue response template
    let dialogue_template = PromptTemplate::new("dialogue_response".to_string(),
        r#"You are {{speaker.name}} engaged in conversation. Your personality: {{speaker.tone}}, humor: {{speaker.humor}}.

{{speaker.backstory}}

Current conversation state:
- Your mood: {{speaker.mood}}
- Your confidence: {{speaker.confidence}}
- Turn: {{turn_count}}
- Conversation sentiment: {{conversation_sentiment}}

Player's emotional state:
- Sentiment: {{player_emotion.sentiment}}
- Intensity: {{player_emotion.intensity}}

{{#if conversation_history}}
Recent conversation:
{{conversation_history}}
{{/if}}

{{#if relevant_memories}}
Relevant memories:
{{relevant_memories}}
{{/if}}

{{#if additional_context}}
Context: {{additional_context}}
{{/if}}

The player says: "{{player_input}}"

Respond as {{speaker.name}} would, considering their personality, the emotional context, and conversation history. Be natural, engaging, and emotionally appropriate:"#.trim().to_string()
    );

    engine.register_template("dialogue_response", dialogue_template)?;
        
        Ok(())
    }
    
    /// Get conversation by ID
    pub async fn get_conversation(&self, conversation_id: &str) -> Option<ActiveConversation> {
        let conversations = self.conversations.read().await;
        conversations.get(conversation_id).cloned()
    }
    
    /// Get system metrics
    pub async fn get_metrics(&self) -> DialogueMetrics {
        self.metrics.read().await.clone()
    }
    
    /// End a conversation
    pub async fn end_conversation(&self, conversation_id: &str) -> Result<()> {
        let mut conversations = self.conversations.write().await;
        if conversations.remove(conversation_id).is_some() {
            let mut metrics = self.metrics.write().await;
            metrics.active_conversations = metrics.active_conversations.saturating_sub(1);
            Ok(())
        } else {
            Err(anyhow!("Conversation not found: {}", conversation_id))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_llm::MockLlm;
    use astraweave_embeddings::{MockEmbeddingClient, VectorStore};
    use astraweave_rag::{VectorStoreWrapper, RagConfig};
    use astraweave_memory::Persona as BasePersona;
    
    #[tokio::test]
    async fn test_dialogue_system_creation() {
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        
        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            RagConfig::default()
        );
        
        let dialogue_system = LlmDialogueSystem::new(
            llm_client,
            rag_pipeline,
            DialogueConfig::default()
        ).await.unwrap();
        
        let metrics = dialogue_system.get_metrics().await;
        assert_eq!(metrics.total_conversations, 0);
    }
    
    #[tokio::test]
    async fn test_start_conversation() {
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        
        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            RagConfig::default()
        );
        
        let dialogue_system = LlmDialogueSystem::new(
            llm_client,
            rag_pipeline,
            DialogueConfig::default()
        ).await.unwrap();
        
        let conversation_id = dialogue_system.start_conversation(
            "test_conversation".to_string(),
            vec!["player".to_string(), "ai".to_string()],
            Some("Testing dialogue system".to_string())
        ).await.unwrap();
        
        assert_eq!(conversation_id, "test_conversation");
        
        let conversation = dialogue_system.get_conversation(&conversation_id).await;
        assert!(conversation.is_some());
        
        let conv = conversation.unwrap();
        assert_eq!(conv.participants.len(), 2);
        assert_eq!(conv.metadata.turn_count, 0);
    }
    
    #[tokio::test]
    async fn test_emotion_analysis() {
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        
        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            RagConfig::default()
        );
        
        let dialogue_system = LlmDialogueSystem::new(
            llm_client,
            rag_pipeline,
            DialogueConfig::default()
        ).await.unwrap();
        
        // Test positive emotion
        let positive_emotion = dialogue_system.analyze_emotion("I'm so happy and excited!").await.unwrap();
        assert!(positive_emotion.sentiment > 0.0);
        assert!(positive_emotion.intensity > 0.0);
        
        // Test negative emotion
        let negative_emotion = dialogue_system.analyze_emotion("I'm really sad and frustrated").await.unwrap();
        assert!(negative_emotion.sentiment < 0.0);
        
        // Test neutral emotion
        let neutral_emotion = dialogue_system.analyze_emotion("The weather is okay today").await.unwrap();
        assert!(negative_emotion.sentiment.abs() < positive_emotion.sentiment.abs());
    }
    
    #[test]
    fn test_repetition_detection() {
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));
        
        let dialogue_system = tokio_test::block_on(async {
            let rag_pipeline = RagPipeline::new(
                embedding_client,
                vector_store,
                Some(llm_client.clone()),
                RagConfig::default()
            );
            
            LlmDialogueSystem::new(
                llm_client,
                rag_pipeline,
                DialogueConfig::default()
            ).await.unwrap()
        });
        
        // Test repetitive text
        assert!(dialogue_system.detect_repetition("hello world hello world"));
        assert!(dialogue_system.detect_repetition("I am happy I am happy"));
        
        // Test non-repetitive text
        assert!(!dialogue_system.detect_repetition("hello world this is different"));
        assert!(!dialogue_system.detect_repetition("I am happy today"));
    }
}