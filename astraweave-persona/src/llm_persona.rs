/*!
# LLM-Enhanced Persona System

Advanced persona system with LLM integration for dynamic personality evolution,
context-aware prompt generation, and adaptive behavior.
*/

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// LLM Integration
use astraweave_context::{ContextConfig, ConversationHistory, Role};
use astraweave_embeddings::EmbeddingClient;
use astraweave_llm::LlmClient;
// Prompt types are provided by astraweave_prompts compat layer
use astraweave_prompts::context::PromptContext as TemplateContext;
use astraweave_prompts::engine::TemplateEngine;
use astraweave_prompts::template::PromptTemplate;
use astraweave_rag::{MemoryQuery, RagPipeline};

// Base persona from memory crate
use astraweave_memory::Persona as BasePersona;

/// Enhanced persona with LLM capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmPersona {
    /// Base persona data
    pub base: BasePersona,

    /// LLM-specific configuration
    pub llm_config: PersonaLlmConfig,

    /// Dynamic personality state
    pub personality_state: PersonalityState,

    /// Learning and adaptation data
    pub adaptation: AdaptationData,

    /// Prompt generation settings
    pub prompt_settings: PromptSettings,

    /// Memory and experience data
    pub memory_profile: MemoryProfile,
}

/// LLM configuration for persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonaLlmConfig {
    /// Temperature for LLM generation (0.0 to 2.0)
    pub temperature: f32,

    /// Top-p sampling parameter
    pub top_p: f32,

    /// Maximum tokens for persona responses
    pub max_tokens: usize,

    /// Personality factors that influence generation
    pub personality_factors: HashMap<String, f32>,

    /// Response style preferences
    pub response_style: ResponseStyle,

    /// Context window size for this persona
    pub context_window_size: usize,
}

impl Default for PersonaLlmConfig {
    fn default() -> Self {
        let mut personality_factors = HashMap::new();
        personality_factors.insert("creativity".to_string(), 0.7);
        personality_factors.insert("empathy".to_string(), 0.8);
        personality_factors.insert("assertiveness".to_string(), 0.6);
        personality_factors.insert("curiosity".to_string(), 0.7);
        personality_factors.insert("humor".to_string(), 0.5);

        Self {
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: 512,
            personality_factors,
            response_style: ResponseStyle::Conversational,
            context_window_size: 2048,
        }
    }
}

/// Response style preferences
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ResponseStyle {
    /// Natural conversational style
    Conversational,
    /// Formal and structured
    Formal,
    /// Creative and expressive
    Creative,
    /// Technical and precise
    Technical,
    /// Playful and casual
    Playful,
    /// Mysterious and enigmatic
    Mysterious,
}

/// Dynamic personality state that evolves over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityState {
    /// Current mood (-1.0 to 1.0, negative = sad, positive = happy)
    pub current_mood: f32,

    /// Energy level (0.0 to 1.0)
    pub energy_level: f32,

    /// Confidence level (0.0 to 1.0)
    pub confidence: f32,

    /// Trust level towards player (0.0 to 1.0)
    pub trust_level: f32,

    /// Current emotional state
    pub emotional_state: EmotionalState,

    /// Personality drift over time
    pub personality_drift: HashMap<String, f32>,

    /// Recent significant events affecting personality
    pub recent_influences: Vec<PersonalityInfluence>,
}

impl Default for PersonalityState {
    fn default() -> Self {
        Self {
            current_mood: 0.0,
            energy_level: 0.7,
            confidence: 0.6,
            trust_level: 0.5,
            emotional_state: EmotionalState::Neutral,
            personality_drift: HashMap::new(),
            recent_influences: Vec::new(),
        }
    }
}

/// Emotional states for the persona
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EmotionalState {
    Joyful,
    Excited,
    Calm,
    Neutral,
    Thoughtful,
    Concerned,
    Frustrated,
    Sad,
    Angry,
    Surprised,
    Curious,
    Confident,
    Anxious,
}

/// An event that influences personality development
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonalityInfluence {
    /// Description of the event
    pub event: String,

    /// How it affects personality factors
    pub factor_changes: HashMap<String, f32>,

    /// Timestamp of the event
    pub timestamp: u64,

    /// Importance/weight of this influence
    pub importance: f32,

    /// Decay rate (how quickly this influence fades)
    pub decay_rate: f32,
}

/// Learning and adaptation data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationData {
    /// Total interactions with the player
    pub interaction_count: u64,

    /// Successful interactions (positive feedback)
    pub successful_interactions: u64,

    /// Learning rate (how quickly persona adapts)
    pub learning_rate: f32,

    /// Preferred conversation topics
    pub preferred_topics: HashMap<String, f32>,

    /// Topics to avoid
    pub topics_to_avoid: Vec<String>,

    /// Player behavior patterns learned
    pub player_patterns: PlayerPatterns,

    /// Adaptation history
    pub adaptation_history: Vec<AdaptationEvent>,
}

impl Default for AdaptationData {
    fn default() -> Self {
        Self {
            interaction_count: 0,
            successful_interactions: 0,
            learning_rate: 0.1,
            preferred_topics: HashMap::new(),
            topics_to_avoid: Vec::new(),
            player_patterns: PlayerPatterns::default(),
            adaptation_history: Vec::new(),
        }
    }
}

/// Learned patterns about player behavior
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlayerPatterns {
    /// Player's communication style
    pub communication_style: Option<String>,

    /// Player's interests
    pub interests: Vec<String>,

    /// Player's typical session length
    pub avg_session_length: Option<f32>,

    /// Player's preferred interaction times
    pub preferred_times: Vec<String>,

    /// Player's emotional patterns
    pub emotional_patterns: HashMap<String, f32>,
}

/// An adaptation event in the persona's learning history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdaptationEvent {
    /// What triggered the adaptation
    pub trigger: String,

    /// What changed
    pub changes: HashMap<String, f32>,

    /// When it happened
    pub timestamp: u64,

    /// Success rating of this adaptation
    pub success_rating: Option<f32>,
}

/// Prompt generation settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptSettings {
    /// Base system prompt template
    pub system_prompt_template: String,

    /// Context injection strategy
    pub context_injection: ContextInjectionStrategy,

    /// Memory retrieval settings
    pub memory_retrieval: MemoryRetrievalSettings,

    /// Few-shot examples for this persona
    pub few_shot_examples: Vec<FewShotExample>,

    /// Dynamic prompt modifiers
    pub prompt_modifiers: HashMap<String, String>,
}

impl Default for PromptSettings {
    fn default() -> Self {
        Self {
            system_prompt_template: "You are {{persona.name}}, {{persona.description}}. Your personality: {{persona.personality}}. Current mood: {{state.mood}}.".to_string(),
            context_injection: ContextInjectionStrategy::Contextual,
            memory_retrieval: MemoryRetrievalSettings::default(),
            few_shot_examples: Vec::new(),
            prompt_modifiers: HashMap::new(),
        }
    }
}

/// Strategy for injecting context into prompts
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ContextInjectionStrategy {
    /// Full context with memories
    Full,
    /// Only recent context
    Recent,
    /// Contextual based on relevance
    Contextual,
    /// Minimal context
    Minimal,
}

/// Settings for memory retrieval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrievalSettings {
    /// Maximum memories to retrieve
    pub max_memories: usize,

    /// Minimum similarity threshold
    pub min_similarity: f32,

    /// Memory categories to prioritize (stored as strings since MemoryCategory was removed)
    pub priority_categories: Vec<String>,

    /// Recency bonus for newer memories
    pub recency_bonus: f32,
}

impl Default for MemoryRetrievalSettings {
    fn default() -> Self {
        Self {
            max_memories: 5,
            min_similarity: 0.3,
            priority_categories: vec!["Social".to_string(), "Dialogue".to_string()],
            recency_bonus: 0.1,
        }
    }
}

/// A few-shot example for prompt engineering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FewShotExample {
    /// Example input/situation
    pub input: String,

    /// Expected/desired output
    pub output: String,

    /// Context for this example
    pub context: Option<String>,

    /// Tags for categorizing examples
    pub tags: Vec<String>,
}

/// Memory profile for the persona
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryProfile {
    /// Core memories that define the persona
    pub core_memories: Vec<String>,

    /// Episodic memories from interactions
    pub episodic_memories: Vec<String>,

    /// Semantic knowledge
    pub semantic_knowledge: Vec<String>,

    /// Memory consolidation preferences
    pub consolidation_preferences: ConsolidationPreferences,
}

impl Default for MemoryProfile {
    fn default() -> Self {
        Self {
            core_memories: Vec::new(),
            episodic_memories: Vec::new(),
            semantic_knowledge: Vec::new(),
            consolidation_preferences: ConsolidationPreferences::default(),
        }
    }
}

/// Preferences for memory consolidation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsolidationPreferences {
    /// How often to consolidate memories
    pub consolidation_frequency: u32,

    /// Importance threshold for keeping memories
    pub importance_threshold: f32,

    /// Maximum memories to keep
    pub max_memories: usize,

    /// Forgetting curve parameters
    pub forgetting_curve: ForgettingCurve,
}

impl Default for ConsolidationPreferences {
    fn default() -> Self {
        Self {
            consolidation_frequency: 100, // Every 100 interactions
            importance_threshold: 0.3,
            max_memories: 1000,
            forgetting_curve: ForgettingCurve::default(),
        }
    }
}

/// Parameters for memory forgetting
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForgettingCurve {
    /// Base decay rate
    pub decay_rate: f32,

    /// Importance multiplier (important memories decay slower)
    pub importance_multiplier: f32,

    /// Rehearsal bonus (frequently accessed memories decay slower)
    pub rehearsal_bonus: f32,
}

impl Default for ForgettingCurve {
    fn default() -> Self {
        Self {
            decay_rate: 0.1,
            importance_multiplier: 2.0,
            rehearsal_bonus: 0.5,
        }
    }
}

/// Main LLM-enhanced persona manager
pub struct LlmPersonaManager {
    /// The enhanced persona
    persona: Arc<RwLock<LlmPersona>>,

    /// LLM client for generation
    llm_client: Arc<dyn LlmClient>,

    /// RAG pipeline for memory retrieval
    rag_pipeline: Arc<RwLock<RagPipeline>>,

    /// Conversation history
    conversation_history: Arc<RwLock<ConversationHistory>>,

    /// Template engine for prompt generation
    template_engine: Arc<RwLock<TemplateEngine>>,

    /// Performance metrics
    metrics: Arc<RwLock<PersonaMetrics>>,
}

/// Performance metrics for persona system
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PersonaMetrics {
    pub total_interactions: u64,
    pub successful_generations: u64,
    pub failed_generations: u64,
    pub avg_response_time_ms: f32,
    pub personality_evolution_events: u64,
    pub memory_consolidations: u64,
    pub adaptation_learning_events: u64,
}

impl LlmPersonaManager {
    /// Create a new LLM persona manager
    pub async fn new(
        base_persona: BasePersona,
        llm_client: Arc<dyn LlmClient>,
        rag_pipeline: RagPipeline,
        _embedding_client: Arc<dyn EmbeddingClient>,
    ) -> Result<Self> {
        // Create enhanced persona
        let llm_persona = LlmPersona {
            base: base_persona,
            llm_config: PersonaLlmConfig::default(),
            personality_state: PersonalityState::default(),
            adaptation: AdaptationData::default(),
            prompt_settings: PromptSettings::default(),
            memory_profile: MemoryProfile::default(),
        };

        // Set up conversation history
        let context_config = ContextConfig {
            max_tokens: llm_persona.llm_config.context_window_size,
            enable_summarization: true,
            ..Default::default()
        };
        let conversation_history =
            ConversationHistory::with_llm_client(context_config, llm_client.clone());

        // Set up template engine
        let mut template_engine = TemplateEngine::new();
        Self::setup_persona_templates(&mut template_engine).await?;

        Ok(Self {
            persona: Arc::new(RwLock::new(llm_persona)),
            llm_client,
            rag_pipeline: Arc::new(RwLock::new(rag_pipeline)),
            conversation_history: Arc::new(RwLock::new(conversation_history)),
            template_engine: Arc::new(RwLock::new(template_engine)),
            metrics: Arc::new(RwLock::new(PersonaMetrics::default())),
        })
    }

    /// Generate a response as this persona
    pub async fn generate_response(&self, input: &str, context: Option<&str>) -> Result<String> {
        let start_time = std::time::Instant::now();

        // 1. Update interaction count
        {
            let mut persona = self.persona.write().await;
            persona.adaptation.interaction_count += 1;
        }

        // 2. Add user input to conversation history
        {
            let history = self.conversation_history.write().await;
            history.add_message(Role::User, input.to_string()).await?;
        }

        // 3. Retrieve relevant memories
        let memories = {
            let rag = self.rag_pipeline.read().await;
            let query = MemoryQuery::text(input);
            rag.retrieve_with_query(&query, 5).await?
        };

        // 4. Build context-aware prompt
        let prompt = self.build_persona_prompt(input, context, &memories).await?;

        // 5. Generate response using LLM
        let response = self.llm_client.complete(&prompt).await?;
        let cleaned_response = self.clean_and_validate_response(&response).await?;

        // 6. Add response to conversation history
        {
            let history = self.conversation_history.write().await;
            history
                .add_message(Role::Assistant, cleaned_response.clone())
                .await?;
        }

        // 7. Store this interaction as a memory
        let interaction_memory = format!(
            "Player: {} | {}: {}",
            input,
            self.get_persona_name().await,
            cleaned_response
        );
        {
            let mut rag = self.rag_pipeline.write().await;
            rag.add_memory(interaction_memory).await?;
        }

        // 8. Update persona state based on interaction
        self.update_personality_state(input, &cleaned_response)
            .await?;

        // 9. Update metrics
        let duration = start_time.elapsed().as_secs_f32() * 1000.0;
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_interactions += 1;
            metrics.successful_generations += 1;
            metrics.avg_response_time_ms =
                (metrics.avg_response_time_ms * (metrics.total_interactions - 1) as f32 + duration)
                    / metrics.total_interactions as f32;
        }

        Ok(cleaned_response)
    }

    /// Get the persona's name
    pub async fn get_persona_name(&self) -> String {
        // Extract name from tone field (temporary until we have a proper name field)
        let persona = self.persona.read().await;
        persona.base.voice.clone() // Using voice as name for now
    }

    /// Build a context-aware prompt for the persona
    async fn build_persona_prompt(
        &self,
        input: &str,
        context: Option<&str>,
        memories: &[astraweave_rag::RetrievedMemory],
    ) -> Result<String> {
        let persona = self.persona.read().await;
        let conversation_context = {
            let history = self.conversation_history.read().await;
            history.get_context(1024).await?
        };

        // Build template context
        let mut template_context = TemplateContext::new();

        // Basic persona info
        template_context.set_path(
            "persona.name",
            persona.base.voice.clone().into(),
        ); // Using voice as name
        template_context.set_path("persona.tone", persona.base.tone.clone().into());
        template_context.set_path(
            "persona.humor",
            persona.base.humor.clone().into(),
        );
        template_context.set_path("persona.risk", persona.base.risk.clone().into());
        template_context.set_path(
            "persona.backstory",
            persona.base.backstory.clone().into(),
        );

        // Personality state
        template_context.set_path(
            "state.mood",
            format!("{:?}", persona.personality_state.emotional_state).into(),
        );
        template_context.set_path(
            "state.energy",
            persona.personality_state.energy_level.to_string().into(),
        );
        template_context.set_path(
            "state.confidence",
            persona.personality_state.confidence.to_string().into(),
        );
        template_context.set_path(
            "state.trust",
            persona.personality_state.trust_level.to_string().into(),
        );

        // Input and context
        template_context.set("user_input".to_string(), input.to_string().into());
        if let Some(ctx) = context {
            template_context.set("additional_context".to_string(), ctx.to_string().into());
        }
        template_context.set(
            "conversation_history".to_string(),
            conversation_context.into(),
        );

        // Memories
        if !memories.is_empty() {
            let memory_text: Vec<String> = memories
                .iter()
                .map(|m| format!("- {}", m.memory.text))
                .collect();
            template_context.set(
                "relevant_memories".to_string(),
                memory_text.join("\n").into(),
            );
        }

        // Personality factors
        let personality_factors: Vec<String> = persona
            .llm_config
            .personality_factors
            .iter()
            .map(|(k, v)| format!("{}: {:.1}", k, v))
            .collect();
        template_context.set(
            "personality_factors".to_string(),
            personality_factors.join(", ").into(),
        );

        // Use template engine to generate prompt
        let template_engine = self.template_engine.read().await;
        template_engine.render("persona_response", &template_context)
    }

    /// Clean and validate the LLM response
    async fn clean_and_validate_response(&self, response: &str) -> Result<String> {
        // Basic cleaning
        let cleaned = response
            .trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ");

        // Validate length
        if cleaned.is_empty() {
            return Err(anyhow!("Empty response generated"));
        }

        if cleaned.len() > 2048 {
            // Truncate if too long
            return Ok(cleaned[..2048].to_string());
        }

        Ok(cleaned)
    }

    /// Update personality state based on interaction
    async fn update_personality_state(&self, input: &str, _response: &str) -> Result<()> {
        let mut persona = self.persona.write().await;

        // Simple sentiment analysis (could be enhanced with LLM)
        let positive_words = [
            "good",
            "great",
            "excellent",
            "amazing",
            "wonderful",
            "thank",
            "appreciate",
        ];
        let negative_words = [
            "bad", "terrible", "awful", "hate", "wrong", "error", "problem",
        ];

        let input_lower = input.to_lowercase();
        let positive_count = positive_words
            .iter()
            .filter(|&&word| input_lower.contains(word))
            .count();
        let negative_count = negative_words
            .iter()
            .filter(|&&word| input_lower.contains(word))
            .count();

        // Adjust mood based on interaction sentiment
        let mood_change = (positive_count as f32 - negative_count as f32) * 0.1;
        persona.personality_state.current_mood =
            (persona.personality_state.current_mood + mood_change).clamp(-1.0, 1.0);

        // Adjust energy level (decreases slightly with interaction, recovers with rest words)
        let rest_words = ["rest", "sleep", "wait", "pause", "relax"];
        let is_resting = rest_words.iter().any(|&w| input_lower.contains(w));
        
        if is_resting {
            persona.personality_state.energy_level = (persona.personality_state.energy_level + 0.2).min(1.0);
        } else {
            persona.personality_state.energy_level = (persona.personality_state.energy_level - 0.01).max(0.0);
        }

        // Adjust confidence based on success
        if positive_count > negative_count {
            persona.personality_state.confidence = (persona.personality_state.confidence + 0.02).min(1.0);
        } else if negative_count > positive_count {
            persona.personality_state.confidence = (persona.personality_state.confidence - 0.02).max(0.0);
        }

        // Update emotional state based on mood and energy
        persona.personality_state.emotional_state = Self::calculate_emotional_state(
            persona.personality_state.current_mood,
            persona.personality_state.energy_level
        );

        // Adjust trust based on successful interaction
        persona.personality_state.trust_level =
            (persona.personality_state.trust_level + 0.01).min(1.0);

        // Increment successful interactions (simplified success detection)
        if positive_count > negative_count || input_lower.contains("good") {
            persona.adaptation.successful_interactions += 1;
        }

        Ok(())
    }

    /// Calculate emotional state from mood and energy
    fn calculate_emotional_state(mood: f32, energy: f32) -> EmotionalState {
        if mood >= 0.3 {
            if energy >= 0.6 {
                EmotionalState::Excited
            } else if energy >= 0.3 {
                EmotionalState::Joyful
            } else {
                EmotionalState::Calm
            }
        } else if mood <= -0.3 {
            if energy >= 0.6 {
                EmotionalState::Angry
            } else if energy >= 0.3 {
                EmotionalState::Frustrated
            } else {
                EmotionalState::Sad
            }
        } else {
            // Neutral mood range
            if energy >= 0.7 {
                EmotionalState::Curious
            } else if energy <= 0.3 {
                EmotionalState::Thoughtful
            } else {
                EmotionalState::Neutral
            }
        }
    }

    /// Set up template engine with persona-specific templates
    async fn setup_persona_templates(engine: &mut TemplateEngine) -> Result<()> {
        // Main persona response template
        let response_template = PromptTemplate::new("persona_response".to_string(),
            r#"You are {{persona.name}}, speaking with tone: {{persona.tone}}, humor level: {{persona.humor}}, risk tolerance: {{persona.risk}}.

{{persona.backstory}}

Your current state:
- Mood: {{state.mood}}
- Energy: {{state.energy}}
- Confidence: {{state.confidence}}
- Trust in player: {{state.trust}}

Personality factors: {{personality_factors}}

{{#if conversation_history}}
Recent conversation:
{{conversation_history}}
{{/if}}

{{#if relevant_memories}}
Relevant memories:
{{relevant_memories}}
{{/if}}

{{#if additional_context}}
Additional context: {{additional_context}}
{{/if}}

The player says: "{{user_input}}"

Respond as {{persona.name}} would, staying true to your personality, current state, and past experiences:"#.trim().to_string()
    );

        engine.register_template("persona_response", response_template)?;

        Ok(())
    }

    /// Evolve the persona based on long-term interactions
    pub async fn evolve_personality(&self, trigger: &str) -> Result<()> {
        let mut persona = self.persona.write().await;

        // Create personality influence event
        let mut factor_changes = HashMap::new();

        // Example evolution rules (could be made more sophisticated)
        if trigger.contains("creative") || trigger.contains("art") || trigger.contains("imagine") {
            factor_changes.insert("creativity".to_string(), 0.05);
        }

        if trigger.contains("help") || trigger.contains("support") || trigger.contains("care") {
            factor_changes.insert("empathy".to_string(), 0.03);
        }

        if trigger.contains("joke") || trigger.contains("funny") || trigger.contains("laugh") {
            factor_changes.insert("humor".to_string(), 0.04);
        }

        if !factor_changes.is_empty() {
            // Apply changes to personality factors
            for (factor, change) in &factor_changes {
                let current = persona
                    .llm_config
                    .personality_factors
                    .get(factor)
                    .unwrap_or(&0.5);
                let new_value = (current + change).clamp(0.0, 1.0);
                persona
                    .llm_config
                    .personality_factors
                    .insert(factor.clone(), new_value);
            }

            // Record the influence
            let influence = PersonalityInfluence {
                event: trigger.to_string(),
                factor_changes,
                timestamp: chrono::Utc::now().timestamp() as u64,
                importance: 0.5,
                decay_rate: 0.1,
            };

            persona.personality_state.recent_influences.push(influence);

            // Update metrics
            drop(persona);
            let mut metrics = self.metrics.write().await;
            metrics.personality_evolution_events += 1;
        }

        Ok(())
    }

    /// Get current persona state
    pub async fn get_persona_state(&self) -> LlmPersona {
        self.persona.read().await.clone()
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> PersonaMetrics {
        self.metrics.read().await.clone()
    }

    /// Set persona state (useful for testing and restoration)
    pub async fn set_persona_state(&self, state: LlmPersona) {
        let mut persona = self.persona.write().await;
        *persona = state;
    }

    /// Trigger maintenance tasks for the persona's memory (consolidation and forgetting)
    pub async fn maintenance(&self) -> Result<()> {
        let mut rag = self.rag_pipeline.write().await;
        
        // Trigger consolidation
        if let Err(e) = rag.trigger_consolidation().await {
            eprintln!("Consolidation error: {}", e);
        }
        
        // Trigger forgetting
        if let Err(e) = rag.trigger_forgetting().await {
            eprintln!("Forgetting error: {}", e);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_embeddings::{MockEmbeddingClient, VectorStore};
    use astraweave_llm::MockLlm;
    use astraweave_rag::VectorStoreWrapper;

    #[tokio::test]
    async fn test_llm_persona_creation() {
        let base_persona = BasePersona::default();
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));

        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            astraweave_rag::RagConfig::default(),
        );

        let manager = LlmPersonaManager::new(
            base_persona,
            llm_client,
            rag_pipeline,
            Arc::new(MockEmbeddingClient::new()),
        )
        .await
        .unwrap();

        let persona_state = manager.get_persona_state().await;
        assert_eq!(persona_state.personality_state.trust_level, 0.5);
        assert_eq!(persona_state.adaptation.interaction_count, 0);
    }

    #[tokio::test]
    async fn test_personality_evolution() {
        let base_persona = BasePersona::default();
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));

        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            astraweave_rag::RagConfig::default(),
        );

        let manager = LlmPersonaManager::new(
            base_persona,
            llm_client,
            rag_pipeline,
            Arc::new(MockEmbeddingClient::new()),
        )
        .await
        .unwrap();

        // Evolve personality with creative trigger
        manager
            .evolve_personality("Let's create some art together")
            .await
            .unwrap();

        let persona_state = manager.get_persona_state().await;
        let creativity = persona_state
            .llm_config
            .personality_factors
            .get("creativity")
            .unwrap_or(&0.5);

        // Should have increased creativity
        assert!(*creativity > 0.7);
        assert_eq!(persona_state.personality_state.recent_influences.len(), 1);
    }

    #[tokio::test]
    async fn test_response_generation() {
        let base_persona = BasePersona::default();
        let llm_client = Arc::new(MockLlm);
        let embedding_client = Arc::new(MockEmbeddingClient::new());
        let vector_store = Arc::new(VectorStoreWrapper::new(VectorStore::new(384)));

        let rag_pipeline = RagPipeline::new(
            embedding_client,
            vector_store,
            Some(llm_client.clone()),
            astraweave_rag::RagConfig::default(),
        );

        let manager = LlmPersonaManager::new(
            base_persona,
            llm_client,
            rag_pipeline,
            Arc::new(MockEmbeddingClient::new()),
        )
        .await
        .unwrap();

        let response = manager.generate_response("Hello, how are you?", None).await;

        // Should succeed with MockLlm
        assert!(response.is_ok());
        let response_text = response.unwrap();
        assert!(!response_text.is_empty());

        // Check metrics updated
        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_interactions, 1);
        assert_eq!(metrics.successful_generations, 1);
    }

    #[test]
    fn test_response_style_variants() {
        let styles = vec![
            ResponseStyle::Conversational,
            ResponseStyle::Formal,
            ResponseStyle::Creative,
            ResponseStyle::Technical,
            ResponseStyle::Playful,
            ResponseStyle::Mysterious,
        ];
        
        for style in styles {
            let _ = format!("{:?}", style);
        }
    }

    #[test]
    fn test_emotional_state_variants() {
        let states = vec![
            EmotionalState::Joyful,
            EmotionalState::Excited,
            EmotionalState::Calm,
            EmotionalState::Neutral,
            EmotionalState::Thoughtful,
            EmotionalState::Concerned,
            EmotionalState::Frustrated,
            EmotionalState::Sad,
            EmotionalState::Angry,
            EmotionalState::Surprised,
            EmotionalState::Curious,
            EmotionalState::Confident,
            EmotionalState::Anxious,
        ];
        
        for state in states {
            let _ = format!("{:?}", state);
        }
    }

    #[test]
    fn test_context_injection_strategy_variants() {
        let strategies = vec![
            ContextInjectionStrategy::Full,
            ContextInjectionStrategy::Recent,
            ContextInjectionStrategy::Contextual,
            ContextInjectionStrategy::Minimal,
        ];
        
        for strategy in strategies {
            let _ = format!("{:?}", strategy);
        }
    }

    #[test]
    fn test_persona_llm_config_default() {
        let config = PersonaLlmConfig::default();
        
        assert_eq!(config.temperature, 0.8);
        assert_eq!(config.top_p, 0.9);
        assert_eq!(config.max_tokens, 512);
        assert_eq!(config.context_window_size, 2048);
        assert!(matches!(config.response_style, ResponseStyle::Conversational));
        
        // Check default personality factors
        assert_eq!(config.personality_factors.len(), 5);
        assert_eq!(*config.personality_factors.get("creativity").unwrap(), 0.7);
        assert_eq!(*config.personality_factors.get("empathy").unwrap(), 0.8);
        assert_eq!(*config.personality_factors.get("assertiveness").unwrap(), 0.6);
        assert_eq!(*config.personality_factors.get("curiosity").unwrap(), 0.7);
        assert_eq!(*config.personality_factors.get("humor").unwrap(), 0.5);
    }

    #[test]
    fn test_personality_state_default() {
        let state = PersonalityState::default();
        
        assert_eq!(state.current_mood, 0.0);
        assert_eq!(state.energy_level, 0.7);
        assert_eq!(state.confidence, 0.6);
        assert_eq!(state.trust_level, 0.5);
        assert!(matches!(state.emotional_state, EmotionalState::Neutral));
        assert!(state.personality_drift.is_empty());
        assert!(state.recent_influences.is_empty());
    }

    #[test]
    fn test_adaptation_data_default() {
        let data = AdaptationData::default();
        
        assert_eq!(data.interaction_count, 0);
        assert_eq!(data.successful_interactions, 0);
        assert_eq!(data.learning_rate, 0.1);
        assert!(data.preferred_topics.is_empty());
        assert!(data.topics_to_avoid.is_empty());
        assert!(data.adaptation_history.is_empty());
    }

    #[test]
    fn test_player_patterns_default() {
        let patterns = PlayerPatterns::default();
        
        assert!(patterns.communication_style.is_none());
        assert!(patterns.interests.is_empty());
        assert!(patterns.avg_session_length.is_none());
        assert!(patterns.preferred_times.is_empty());
        assert!(patterns.emotional_patterns.is_empty());
    }

    #[test]
    fn test_prompt_settings_default() {
        let settings = PromptSettings::default();
        
        assert!(settings.system_prompt_template.contains("{{persona.name}}"));
        assert!(matches!(settings.context_injection, ContextInjectionStrategy::Contextual));
        assert!(settings.few_shot_examples.is_empty());
        assert!(settings.prompt_modifiers.is_empty());
    }

    #[test]
    fn test_memory_retrieval_settings_default() {
        let settings = MemoryRetrievalSettings::default();
        
        assert_eq!(settings.max_memories, 5);
        assert_eq!(settings.min_similarity, 0.3);
        assert_eq!(settings.recency_bonus, 0.1);
        assert_eq!(settings.priority_categories.len(), 2);
        assert!(settings.priority_categories.contains(&"Social".to_string()));
        assert!(settings.priority_categories.contains(&"Dialogue".to_string()));
    }

    #[test]
    fn test_memory_profile_default() {
        let profile = MemoryProfile::default();
        
        assert!(profile.core_memories.is_empty());
        assert!(profile.episodic_memories.is_empty());
        assert!(profile.semantic_knowledge.is_empty());
    }

    #[test]
    fn test_consolidation_preferences_default() {
        let prefs = ConsolidationPreferences::default();
        
        assert_eq!(prefs.consolidation_frequency, 100);
        assert_eq!(prefs.importance_threshold, 0.3);
        assert_eq!(prefs.max_memories, 1000);
    }

    #[test]
    fn test_forgetting_curve_default() {
        let curve = ForgettingCurve::default();
        
        assert_eq!(curve.decay_rate, 0.1);
        assert_eq!(curve.importance_multiplier, 2.0);
        assert_eq!(curve.rehearsal_bonus, 0.5);
    }

    #[test]
    fn test_persona_metrics_default() {
        let metrics = PersonaMetrics::default();
        
        assert_eq!(metrics.total_interactions, 0);
        assert_eq!(metrics.successful_generations, 0);
        assert_eq!(metrics.failed_generations, 0);
        assert_eq!(metrics.avg_response_time_ms, 0.0);
        assert_eq!(metrics.personality_evolution_events, 0);
        assert_eq!(metrics.memory_consolidations, 0);
        assert_eq!(metrics.adaptation_learning_events, 0);
    }

    #[test]
    fn test_llm_persona_serialization() {
        let persona = LlmPersona {
            base: BasePersona::default(),
            llm_config: PersonaLlmConfig::default(),
            personality_state: PersonalityState::default(),
            adaptation: AdaptationData::default(),
            prompt_settings: PromptSettings::default(),
            memory_profile: MemoryProfile::default(),
        };
        
        let serialized = serde_json::to_string(&persona).unwrap();
        let deserialized: LlmPersona = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(persona.personality_state.current_mood, deserialized.personality_state.current_mood);
        assert_eq!(persona.llm_config.temperature, deserialized.llm_config.temperature);
    }

    #[test]
    fn test_personality_influence_serialization() {
        let mut factor_changes = HashMap::new();
        factor_changes.insert("test".to_string(), 0.5);
        
        let influence = PersonalityInfluence {
            event: "Test event".to_string(),
            factor_changes,
            timestamp: 1234567890,
            importance: 0.75,
            decay_rate: 0.05,
        };
        
        let serialized = serde_json::to_string(&influence).unwrap();
        let deserialized: PersonalityInfluence = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(influence.event, deserialized.event);
        assert_eq!(influence.importance, deserialized.importance);
    }

    #[test]
    fn test_few_shot_example_serialization() {
        let example = FewShotExample {
            input: "Test input".to_string(),
            output: "Test output".to_string(),
            context: Some("Test context".to_string()),
            tags: vec!["test".to_string(), "example".to_string()],
        };
        
        let serialized = serde_json::to_string(&example).unwrap();
        let deserialized: FewShotExample = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(example.input, deserialized.input);
        assert_eq!(example.tags.len(), deserialized.tags.len());
    }

    #[test]
    fn test_adaptation_event_serialization() {
        let mut changes = HashMap::new();
        changes.insert("confidence".to_string(), 0.1);
        
        let event = AdaptationEvent {
            trigger: "Positive feedback".to_string(),
            changes,
            timestamp: 9876543210,
            success_rating: Some(0.95),
        };
        
        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: AdaptationEvent = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(event.trigger, deserialized.trigger);
        assert_eq!(event.success_rating, deserialized.success_rating);
    }

    #[test]
    fn test_calculate_emotional_state_high_mood_high_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.5, 0.8);
        assert!(matches!(state, EmotionalState::Excited));
    }

    #[test]
    fn test_calculate_emotional_state_high_mood_medium_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.5, 0.5);
        assert!(matches!(state, EmotionalState::Joyful));
    }

    #[test]
    fn test_calculate_emotional_state_high_mood_low_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.5, 0.2);
        assert!(matches!(state, EmotionalState::Calm));
    }

    #[test]
    fn test_calculate_emotional_state_low_mood_high_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(-0.5, 0.8);
        assert!(matches!(state, EmotionalState::Angry));
    }

    #[test]
    fn test_calculate_emotional_state_low_mood_medium_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(-0.5, 0.5);
        assert!(matches!(state, EmotionalState::Frustrated));
    }

    #[test]
    fn test_calculate_emotional_state_low_mood_low_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(-0.5, 0.2);
        assert!(matches!(state, EmotionalState::Sad));
    }

    #[test]
    fn test_calculate_emotional_state_neutral_mood_high_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.0, 0.8);
        assert!(matches!(state, EmotionalState::Curious));
    }

    #[test]
    fn test_calculate_emotional_state_neutral_mood_low_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.0, 0.2);
        assert!(matches!(state, EmotionalState::Thoughtful));
    }

    #[test]
    fn test_calculate_emotional_state_neutral_mood_medium_energy() {
        let state = LlmPersonaManager::calculate_emotional_state(0.0, 0.5);
        assert!(matches!(state, EmotionalState::Neutral));
    }

    #[test]
    fn test_llm_persona_clone() {
        let persona = LlmPersona {
            base: BasePersona::default(),
            llm_config: PersonaLlmConfig::default(),
            personality_state: PersonalityState::default(),
            adaptation: AdaptationData::default(),
            prompt_settings: PromptSettings::default(),
            memory_profile: MemoryProfile::default(),
        };
        
        let cloned = persona.clone();
        
        assert_eq!(persona.personality_state.current_mood, cloned.personality_state.current_mood);
        assert_eq!(persona.llm_config.temperature, cloned.llm_config.temperature);
    }

    #[test]
    fn test_llm_persona_debug() {
        let persona = LlmPersona {
            base: BasePersona::default(),
            llm_config: PersonaLlmConfig::default(),
            personality_state: PersonalityState::default(),
            adaptation: AdaptationData::default(),
            prompt_settings: PromptSettings::default(),
            memory_profile: MemoryProfile::default(),
        };
        
        let debug_str = format!("{:?}", persona);
        assert!(debug_str.contains("LlmPersona"));
    }

    #[test]
    fn test_player_patterns_serialization() {
        let mut emotional_patterns = HashMap::new();
        emotional_patterns.insert("happiness".to_string(), 0.8);
        
        let patterns = PlayerPatterns {
            communication_style: Some("formal".to_string()),
            interests: vec!["science".to_string()],
            avg_session_length: Some(60.0),
            preferred_times: vec!["morning".to_string()],
            emotional_patterns,
        };
        
        let serialized = serde_json::to_string(&patterns).unwrap();
        let deserialized: PlayerPatterns = serde_json::from_str(&serialized).unwrap();
        
        assert_eq!(patterns.communication_style, deserialized.communication_style);
        assert_eq!(patterns.avg_session_length, deserialized.avg_session_length);
    }
}
