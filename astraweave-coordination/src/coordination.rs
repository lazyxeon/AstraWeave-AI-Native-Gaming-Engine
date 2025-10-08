use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{RwLock, mpsc};
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use futures::future::join_all;

use crate::agent::{
    Agent, AgentMessage, AgentGoal, Task, TaskResult, MessagePriority,
    CoordinationContext, CoordinationStatus, WorldEvent, ResourceUsage
};

/// Central coordinator for managing multiple LLM agents
pub struct AgentCoordinator {
    /// Registered agents
    agents: Arc<RwLock<HashMap<String, Box<dyn Agent>>>>,
    /// Message routing system
    message_router: MessageRouter,
    /// Resource manager
    resource_manager: ResourceManager,
    /// Event dispatcher
    event_dispatcher: EventDispatcher,
    /// Active coordination sessions
    coordination_sessions: Arc<RwLock<HashMap<String, Arc<RwLock<CoordinationContext>>>>>,
    /// Performance metrics
    metrics: Arc<RwLock<CoordinationMetrics>>,
    /// Configuration
    config: CoordinatorConfig,
}

/// Configuration for the agent coordinator
#[derive(Debug, Clone)]
pub struct CoordinatorConfig {
    /// Maximum number of concurrent tasks per agent
    pub max_tasks_per_agent: usize,
    /// Maximum message queue size per agent
    pub max_message_queue_size: usize,
    /// Default task timeout
    pub default_task_timeout: Duration,
    /// Resource allocation strategy
    pub resource_strategy: ResourceStrategy,
    /// Enable performance monitoring
    pub enable_metrics: bool,
    /// Maximum coordination session duration
    pub max_coordination_duration: Duration,
}

/// Resource allocation strategies
#[derive(Debug, Clone)]
pub enum ResourceStrategy {
    FirstCome,     // First agent to request gets resources
    Priority,      // Based on agent priority and task priority
    LoadBalance,   // Distribute load evenly
    Adaptive,      // Learn optimal allocation over time
}

impl Default for CoordinatorConfig {
    fn default() -> Self {
        Self {
            max_tasks_per_agent: 5,
            max_message_queue_size: 100,
            default_task_timeout: Duration::from_secs(30),
            resource_strategy: ResourceStrategy::Priority,
            enable_metrics: true,
            max_coordination_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Message routing system for inter-agent communication
pub struct MessageRouter {
    /// Message delivery channels
    channels: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<AgentMessage>>>>,
    /// Message history for debugging
    message_history: Arc<RwLock<Vec<AgentMessage>>>,
    /// Routing rules
    routing_rules: Arc<RwLock<Vec<RoutingRule>>>,
}

/// Rule for message routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,
    pub from_pattern: Option<String>,    // Regex pattern for sender
    pub to_pattern: Option<String>,      // Regex pattern for recipient
    pub message_type: Option<String>,    // Message type filter
    pub action: RoutingAction,
    pub priority: i32,                   // Higher priority rules apply first
}

/// Action to take when routing rule matches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutingAction {
    Allow,
    Block,
    Redirect(String),  // Redirect to different agent
    Broadcast(Vec<String>), // Send to multiple agents
    Transform(String), // Transform message content (placeholder)
}

/// Resource management for coordinated agents
pub struct ResourceManager {
    /// Current resource allocations
    allocations: Arc<RwLock<HashMap<String, ResourceAllocation>>>,
    /// Resource limits
    limits: ResourceLimits,
    /// Allocation strategy
    strategy: ResourceStrategy,
}

/// Resource allocation for an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    pub agent_id: String,
    pub llm_calls_allocated: u32,
    pub llm_calls_used: u32,
    pub memory_allocated: u32,
    pub memory_used: u32,
    pub allocated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
}

/// System-wide resource limits
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub total_llm_calls_per_minute: u32,
    pub total_memory_mb: u32,
    pub max_concurrent_llm_calls: u32,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            total_llm_calls_per_minute: 1000,
            total_memory_mb: 8192, // 8GB
            max_concurrent_llm_calls: 50,
        }
    }
}

/// Event dispatching system for world events
pub struct EventDispatcher {
    /// Event subscriptions by agent
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>, // agent_id -> event_types
    /// Event history
    event_history: Arc<RwLock<Vec<WorldEvent>>>,
    /// Event processing queue
    event_queue: Arc<RwLock<Vec<WorldEvent>>>,
}

/// Metrics for coordination system performance
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CoordinationMetrics {
    pub messages_sent: u64,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub tasks_assigned: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub coordination_sessions: u64,
    pub average_task_duration: f64,
    pub resource_utilization: f64,
    pub agent_availability: HashMap<String, f64>,
    pub event_processing_time: f64,
}

impl AgentCoordinator {
    pub fn new(config: CoordinatorConfig) -> Self {
        let message_router = MessageRouter::new();
        let resource_manager = ResourceManager::new(
            ResourceLimits::default(),
            config.resource_strategy.clone(),
        );
        let event_dispatcher = EventDispatcher::new();

        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            message_router,
            resource_manager,
            event_dispatcher,
            coordination_sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(CoordinationMetrics::default())),
            config,
        }
    }

    /// Register a new agent with the coordinator
    pub async fn register_agent(&self, agent: Box<dyn Agent>) -> Result<()> {
        let agent_id = agent.agent_id().to_string();
        let agent_type = agent.agent_type().to_string();

        info!("Registering agent {} (type: {})", agent_id, agent_type);

        // Add to agents map
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id.clone(), agent);
        }

        // Set up message channel
        self.message_router.setup_agent_channel(&agent_id).await;

        // Initialize resource allocation
        self.resource_manager.initialize_allocation(&agent_id).await?;

        // Subscribe to events
        {
            let agents = self.agents.read().await;
            if let Some(agent) = agents.get(&agent_id) {
                let subscriptions = agent.get_event_subscriptions();
                self.event_dispatcher.subscribe_agent(&agent_id, subscriptions).await;
            }
        }

        Ok(())
    }

    /// Unregister an agent
    pub async fn unregister_agent(&self, agent_id: &str) -> Result<()> {
        info!("Unregistering agent {}", agent_id);

        // Remove from coordination sessions
        self.remove_agent_from_sessions(agent_id).await;

        // Clean up message channel
        self.message_router.remove_agent_channel(agent_id).await;

        // Release resources
        self.resource_manager.release_allocation(agent_id).await;

        // Unsubscribe from events
        self.event_dispatcher.unsubscribe_agent(agent_id).await;

        // Remove from agents map
        let mut agents = self.agents.write().await;
        agents.remove(agent_id);

        Ok(())
    }

    /// Send a message between agents
    pub async fn send_message(&self, message: AgentMessage) -> Result<()> {
        debug!("Routing message from {} to {}", message.from, message.to);

        // Check routing rules
        if !self.message_router.should_deliver(&message).await {
            warn!("Message blocked by routing rules: {:?}", message);
            return Ok(());
        }

        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.messages_sent += 1;
        }

        // Deliver message
        match self.message_router.deliver_message(message).await {
            Ok(_) => {
                if self.config.enable_metrics {
                    let mut metrics = self.metrics.write().await;
                    metrics.messages_delivered += 1;
                }
            }
            Err(e) => {
                if self.config.enable_metrics {
                    let mut metrics = self.metrics.write().await;
                    metrics.messages_failed += 1;
                }
                return Err(e);
            }
        }

        Ok(())
    }

    /// Assign a task to the most suitable agent
    pub async fn assign_task(&self, task: Task) -> Result<String> {
        debug!("Assigning task: {} ({})", task.description, task.task_type);

        // Find suitable agents
        let suitable_agents = self.find_suitable_agents(&task).await?;

        if suitable_agents.is_empty() {
            return Err(anyhow!("No suitable agents found for task: {}", task.task_type));
        }

        // Select best agent based on strategy
        let selected_agent = self.select_best_agent(&suitable_agents, &task).await?;

        // Check resource availability
        if !self.resource_manager.can_allocate(&selected_agent, &task.resource_requirements).await {
            return Err(anyhow!("Insufficient resources for task assignment to {}", selected_agent));
        }

        // Allocate resources
        self.resource_manager.allocate_resources(&selected_agent, &task.resource_requirements).await?;

        // Assign task to agent
        {
            let agents = self.agents.read().await;
            if let Some(agent) = agents.get(&selected_agent) {
                agent.add_task(task.clone()).await;
            } else {
                return Err(anyhow!("Selected agent {} not found", selected_agent));
            }
        }

        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.tasks_assigned += 1;
        }

        info!("Assigned task {} to agent {}", task.id, selected_agent);
        Ok(selected_agent)
    }

    /// Start a coordination session with multiple agents
    pub async fn start_coordination(
        &self,
        participant_ids: Vec<String>,
        goals: Vec<AgentGoal>,
    ) -> Result<String> {
        let session_id = Uuid::new_v4().to_string();

        info!("Starting coordination session {} with agents: {:?}", session_id, participant_ids);

        // Validate all participants exist
        {
            let agents = self.agents.read().await;
            for agent_id in &participant_ids {
                if !agents.contains_key(agent_id) {
                    return Err(anyhow!("Agent {} not found for coordination", agent_id));
                }
            }
        }

        // Create coordination context
        let context = Arc::new(RwLock::new(CoordinationContext {
            session_id: session_id.clone(),
            participants: participant_ids.clone(),
            shared_state: HashMap::new(),
            coordination_goals: goals,
            start_time: Utc::now(),
            last_update: Utc::now(),
            status: CoordinationStatus::Planning,
        }));

        // Register session
        {
            let mut sessions = self.coordination_sessions.write().await;
            sessions.insert(session_id.clone(), context.clone());
        }

        // Add agents to session
        {
            let agents = self.agents.read().await;
            for agent_id in &participant_ids {
                if let Some(agent) = agents.get(agent_id) {
                    // Note: This would require agents to implement joining coordination
                    debug!("Added agent {} to coordination session", agent_id);
                }
            }
        }

        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.coordination_sessions += 1;
        }

        Ok(session_id)
    }

    /// End a coordination session
    pub async fn end_coordination(&self, session_id: &str) -> Result<()> {
        info!("Ending coordination session {}", session_id);

        // Remove session
        let session = {
            let mut sessions = self.coordination_sessions.write().await;
            sessions.remove(session_id)
        };

        if let Some(context) = session {
            // Notify participants
            let ctx = context.read().await;
            let participants = ctx.participants.clone();
            drop(ctx);

            {
                let mut ctx = context.write().await;
                ctx.status = CoordinationStatus::Completed;
                ctx.last_update = Utc::now();
            }

            // Remove agents from session
            for agent_id in participants {
                debug!("Removed agent {} from coordination session", agent_id);
            }
        }

        Ok(())
    }

    /// Dispatch a world event to interested agents
    pub async fn dispatch_event(&self, event: WorldEvent) -> Result<()> {
        debug!("Dispatching event: {} ({})", event.description, event.event_type);

        let start_time = std::time::Instant::now();

        // Get interested agents
        let interested_agents = self.event_dispatcher.get_interested_agents(&event).await;

        // Dispatch to agents in parallel
        let dispatch_tasks: Vec<_> = interested_agents.into_iter().map(|agent_id| {
            let event = event.clone();
            async move {
                let agents = self.agents.read().await;
                if let Some(agent) = agents.get(&agent_id) {
                    match agent.handle_world_event(&event).await {
                        Ok(_) => debug!("Agent {} handled event {}", agent_id, event.id),
                        Err(e) => warn!("Agent {} failed to handle event {}: {}", agent_id, event.id, e),
                    }
                }
            }
        }).collect();

        join_all(dispatch_tasks).await;

        // Update metrics
        if self.config.enable_metrics {
            let mut metrics = self.metrics.write().await;
            metrics.event_processing_time = start_time.elapsed().as_millis() as f64;
        }

        // Store event in history
        self.event_dispatcher.store_event(event).await;

        Ok(())
    }

    /// Get performance metrics
    pub async fn get_metrics(&self) -> CoordinationMetrics {
        if self.config.enable_metrics {
            self.metrics.read().await.clone()
        } else {
            CoordinationMetrics::default()
        }
    }

    /// Find agents suitable for a task
    async fn find_suitable_agents(&self, task: &Task) -> Result<Vec<String>> {
        let agents = self.agents.read().await;
        let mut suitable = Vec::new();

        for (agent_id, agent) in agents.iter() {
            if agent.can_handle_task(task) && agent.is_available() {
                suitable.push(agent_id.clone());
            }
        }

        Ok(suitable)
    }

    /// Select the best agent for a task
    async fn select_best_agent(&self, candidates: &[String], task: &Task) -> Result<String> {
        if candidates.is_empty() {
            return Err(anyhow!("No candidates provided"));
        }

        match self.config.resource_strategy {
            ResourceStrategy::FirstCome => Ok(candidates[0].clone()),
            ResourceStrategy::Priority => {
                // Select agent with lowest current load
                let agents = self.agents.read().await;
                let mut best_agent = candidates[0].clone();
                let mut lowest_load = f64::MAX;

                for agent_id in candidates {
                    if let Some(agent) = agents.get(agent_id) {
                        let usage = agent.get_resource_usage();
                        let load = usage.active_tasks as f64;
                        if load < lowest_load {
                            lowest_load = load;
                            best_agent = agent_id.clone();
                        }
                    }
                }

                Ok(best_agent)
            }
            ResourceStrategy::LoadBalance => {
                // Simple round-robin for now
                let index = (task.id.len() % candidates.len()) as usize;
                Ok(candidates[index].clone())
            }
            ResourceStrategy::Adaptive => {
                // For now, use priority strategy
                // In practice, would use ML to learn optimal assignments
                self.select_best_agent(candidates, task).await
            }
        }
    }

    /// Remove agent from all coordination sessions
    async fn remove_agent_from_sessions(&self, agent_id: &str) {
        let sessions = self.coordination_sessions.read().await;
        for (session_id, context) in sessions.iter() {
            let mut ctx = context.write().await;
            if ctx.participants.contains(&agent_id.to_string()) {
                ctx.participants.retain(|id| id != agent_id);
                ctx.last_update = Utc::now();
                debug!("Removed agent {} from session {}", agent_id, session_id);
            }
        }
    }

    /// Main update loop - call periodically
    pub async fn update(&self) -> Result<()> {
        // Process pending messages
        self.message_router.process_pending_messages().await?;

        // Update resource allocations
        self.resource_manager.update_allocations().await;

        // Process pending events
        self.event_dispatcher.process_pending_events().await;

        // Clean up expired coordination sessions
        self.cleanup_expired_sessions().await;

        // Update metrics
        if self.config.enable_metrics {
            self.update_metrics().await;
        }

        Ok(())
    }

    /// Clean up expired coordination sessions
    async fn cleanup_expired_sessions(&self) {
        let now = Utc::now();
        let mut sessions = self.coordination_sessions.write().await;

        sessions.retain(|session_id, context| {
            let ctx = context.try_read();
            if let Ok(ctx) = ctx {
                let duration = now - ctx.start_time;
                let keep = duration.to_std().unwrap_or(Duration::ZERO) < self.config.max_coordination_duration;
                if !keep {
                    debug!("Cleaning up expired coordination session {}", session_id);
                }
                keep
            } else {
                true // Keep if we can't read (likely in use)
            }
        });
    }

    /// Update performance metrics
    async fn update_metrics(&self) {
        let agents = self.agents.read().await;
        let mut metrics = self.metrics.write().await;

        // Update agent availability
        metrics.agent_availability.clear();
        for (agent_id, agent) in agents.iter() {
            let availability = if agent.is_available() { 1.0 } else { 0.0 };
            metrics.agent_availability.insert(agent_id.clone(), availability);
        }

        // Calculate resource utilization
        let total_usage: u32 = agents.values()
            .map(|agent| agent.get_resource_usage().active_tasks)
            .sum();
        let total_capacity = agents.len() * self.config.max_tasks_per_agent;
        metrics.resource_utilization = if total_capacity > 0 {
            total_usage as f64 / total_capacity as f64
        } else {
            0.0
        };
    }
}

// Implementation for sub-systems

impl MessageRouter {
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            message_history: Arc::new(RwLock::new(Vec::new())),
            routing_rules: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn setup_agent_channel(&self, agent_id: &str) {
        let (sender, _receiver) = mpsc::unbounded_channel();
        let mut channels = self.channels.write().await;
        channels.insert(agent_id.to_string(), sender);
    }

    async fn remove_agent_channel(&self, agent_id: &str) {
        let mut channels = self.channels.write().await;
        channels.remove(agent_id);
    }

    async fn should_deliver(&self, message: &AgentMessage) -> bool {
        let rules = self.routing_rules.read().await;

        // Apply routing rules in priority order
        for rule in rules.iter() {
            if self.rule_matches(rule, message) {
                return matches!(rule.action, RoutingAction::Allow);
            }
        }

        // Default: allow delivery
        true
    }

    async fn deliver_message(&self, message: AgentMessage) -> Result<()> {
        let channels = self.channels.read().await;

        if let Some(channel) = channels.get(&message.to) {
            channel.send(message.clone()).map_err(|_| anyhow!("Failed to send message to agent {}", message.to))?;
        } else {
            return Err(anyhow!("Agent {} not found", message.to));
        }

        // Store in history
        let mut history = self.message_history.write().await;
        history.push(message);

        // Keep history manageable
        if history.len() > 1000 {
            history.remove(0);
        }

        Ok(())
    }

    async fn process_pending_messages(&self) -> Result<()> {
        // In a real implementation, this would process queued messages
        // For now, messages are delivered immediately
        Ok(())
    }

    fn rule_matches(&self, rule: &RoutingRule, message: &AgentMessage) -> bool {
        // Simple pattern matching - in practice would use regex
        if let Some(from_pattern) = &rule.from_pattern {
            if !message.from.contains(from_pattern) {
                return false;
            }
        }

        if let Some(to_pattern) = &rule.to_pattern {
            if !message.to.contains(to_pattern) {
                return false;
            }
        }

        if let Some(msg_type) = &rule.message_type {
            if format!("{:?}", message.message_type) != *msg_type {
                return false;
            }
        }

        true
    }
}

impl ResourceManager {
    pub fn new(limits: ResourceLimits, strategy: ResourceStrategy) -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            limits,
            strategy,
        }
    }

    async fn initialize_allocation(&self, agent_id: &str) -> Result<()> {
        let allocation = ResourceAllocation {
            agent_id: agent_id.to_string(),
            llm_calls_allocated: self.limits.total_llm_calls_per_minute / 10, // Start with 10% allocation
            llm_calls_used: 0,
            memory_allocated: self.limits.total_memory_mb / 10,
            memory_used: 0,
            allocated_at: Utc::now(),
            expires_at: None,
        };

        let mut allocations = self.allocations.write().await;
        allocations.insert(agent_id.to_string(), allocation);

        Ok(())
    }

    async fn can_allocate(&self, agent_id: &str, requirements: &crate::agent::ResourceRequirements) -> bool {
        let allocations = self.allocations.read().await;

        if let Some(allocation) = allocations.get(agent_id) {
            allocation.llm_calls_allocated >= allocation.llm_calls_used + requirements.llm_calls &&
            allocation.memory_allocated >= allocation.memory_used + requirements.memory_mb
        } else {
            false
        }
    }

    async fn allocate_resources(&self, agent_id: &str, requirements: &crate::agent::ResourceRequirements) -> Result<()> {
        let mut allocations = self.allocations.write().await;

        if let Some(allocation) = allocations.get_mut(agent_id) {
            allocation.llm_calls_used += requirements.llm_calls;
            allocation.memory_used += requirements.memory_mb;
        } else {
            return Err(anyhow!("No allocation found for agent {}", agent_id));
        }

        Ok(())
    }

    async fn release_allocation(&self, agent_id: &str) {
        let mut allocations = self.allocations.write().await;
        allocations.remove(agent_id);
    }

    async fn update_allocations(&self) {
        // Reset usage counters periodically
        let now = Utc::now();
        let mut allocations = self.allocations.write().await;

        for allocation in allocations.values_mut() {
            if (now - allocation.allocated_at).num_minutes() >= 1 {
                allocation.llm_calls_used = 0;
                allocation.allocated_at = now;
            }
        }
    }
}

impl EventDispatcher {
    pub fn new() -> Self {
        Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            event_history: Arc::new(RwLock::new(Vec::new())),
            event_queue: Arc::new(RwLock::new(Vec::new())),
        }
    }

    async fn subscribe_agent(&self, agent_id: &str, event_types: Vec<String>) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.insert(agent_id.to_string(), event_types);
    }

    async fn unsubscribe_agent(&self, agent_id: &str) {
        let mut subscriptions = self.subscriptions.write().await;
        subscriptions.remove(agent_id);
    }

    async fn get_interested_agents(&self, event: &WorldEvent) -> Vec<String> {
        let subscriptions = self.subscriptions.read().await;
        let mut interested = Vec::new();

        for (agent_id, event_types) in subscriptions.iter() {
            if event_types.contains(&event.event_type) || event_types.contains(&"*".to_string()) {
                interested.push(agent_id.clone());
            }
        }

        interested
    }

    async fn store_event(&self, event: WorldEvent) {
        let mut history = self.event_history.write().await;
        history.push(event);

        // Keep history manageable
        if history.len() > 1000 {
            history.remove(0);
        }
    }

    async fn process_pending_events(&self) {
        // In a real implementation, this would process queued events
        // For now, events are dispatched immediately
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::agent::BaseAgent;

    struct TestAgent {
        base: BaseAgent,
    }

    #[async_trait]
    impl Agent for TestAgent {
        fn agent_id(&self) -> &str { &self.base.id }
        fn agent_type(&self) -> &str { &self.base.agent_type }
        fn get_state(&self) -> &crate::agent::AgentState { &self.base.state }
        fn set_state(&mut self, state: crate::agent::AgentState) { self.base.state = state; }
        async fn handle_message(&mut self, _message: &AgentMessage) -> Result<Option<AgentMessage>> { Ok(None) }
        fn get_goals(&self) -> Vec<AgentGoal> { self.base.goals.clone() }
        fn set_goals(&mut self, goals: Vec<AgentGoal>) { self.base.goals = goals; }
        fn get_capabilities(&self) -> Vec<String> { self.base.capabilities.clone() }
        fn can_handle_task(&self, _task: &Task) -> bool { true }
        async fn execute_task(&mut self, task: &Task) -> Result<TaskResult> {
            Ok(TaskResult {
                task_id: task.id.clone(),
                success: true,
                result_data: serde_json::json!({}),
                execution_time_ms: 100,
                resources_used: ResourceUsage::default(),
                error_message: None,
                side_effects: Vec::new(),
            })
        }
        fn get_resource_usage(&self) -> ResourceUsage { self.base.resource_usage.clone() }
        fn get_event_subscriptions(&self) -> Vec<String> { self.base.event_subscriptions.clone() }
        async fn handle_world_event(&mut self, _event: &WorldEvent) -> Result<()> { Ok(()) }
        fn is_available(&self) -> bool { self.base.is_available() }
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let coordinator = AgentCoordinator::new(CoordinatorConfig::default());

        let agent = TestAgent {
            base: BaseAgent::new("test_agent".to_string(), vec!["test".to_string()]),
        };
        let agent_id = agent.agent_id().to_string();

        coordinator.register_agent(Box::new(agent)).await.unwrap();

        let agents = coordinator.agents.read().await;
        assert!(agents.contains_key(&agent_id));
    }

    #[tokio::test]
    async fn test_task_assignment() {
        let coordinator = AgentCoordinator::new(CoordinatorConfig::default());

        let agent = TestAgent {
            base: BaseAgent::new("test_agent".to_string(), vec!["test".to_string()]),
        };

        coordinator.register_agent(Box::new(agent)).await.unwrap();

        let task = Task::new("test_task".to_string(), "Test task".to_string());
        let assigned_agent = coordinator.assign_task(task).await.unwrap();

        assert!(!assigned_agent.is_empty());
    }
}