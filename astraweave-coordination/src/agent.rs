use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Core trait for all LLM-powered agents in the coordination system
#[async_trait]
pub trait Agent: Send + Sync {
    /// Unique agent identifier
    fn agent_id(&self) -> &str;

    /// Agent type (e.g., "npc", "director", "quest_generator")
    fn agent_type(&self) -> &str;

    /// Current agent state
    fn get_state(&self) -> &AgentState;

    /// Update agent state
    fn set_state(&mut self, state: AgentState);

    /// Process a message from another agent
    async fn handle_message(&mut self, message: &AgentMessage) -> Result<Option<AgentMessage>>;

    /// Get agent's current goals
    fn get_goals(&self) -> Vec<AgentGoal>;

    /// Update agent's goals
    fn set_goals(&mut self, goals: Vec<AgentGoal>);

    /// Get agent's capabilities
    fn get_capabilities(&self) -> Vec<String>;

    /// Check if agent can handle a specific task
    fn can_handle_task(&self, task: &Task) -> bool;

    /// Execute a task assigned by coordinator
    async fn execute_task(&mut self, task: &Task) -> Result<TaskResult>;

    /// Get agent's current resource usage
    fn get_resource_usage(&self) -> ResourceUsage;

    /// Subscribe to specific event types
    fn get_event_subscriptions(&self) -> Vec<String>;

    /// Handle world event
    async fn handle_world_event(&mut self, event: &WorldEvent) -> Result<()>;
}

/// State of an agent in the coordination system
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AgentState {
    Idle,
    Processing,
    WaitingForInput,
    Collaborating,
    Disabled,
    Error(String),
}

/// Message passed between agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub from: String,
    pub to: String,
    pub message_type: MessageType,
    pub content: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub priority: MessagePriority,
    pub reply_to: Option<String>,
}

/// Types of messages agents can send
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Notification,
    Question,
    Coordination,
    Resource,
    Event,
}

/// Priority levels for agent messages
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Goal that an agent is working towards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentGoal {
    pub id: String,
    pub description: String,
    pub goal_type: GoalType,
    pub priority: f32,
    pub target_value: Option<f32>,
    pub current_value: f32,
    pub deadline: Option<DateTime<Utc>>,
    pub dependencies: Vec<String>, // Goal IDs this depends on
    pub status: GoalStatus,
}

/// Types of goals agents can have
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GoalType {
    Maintain, // Keep a value within range
    Achieve,  // Reach a target value
    Avoid,    // Keep something from happening
    Explore,  // Gather information
    Collaborate, // Work with other agents
}

/// Status of a goal
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum GoalStatus {
    Active,
    Paused,
    Completed,
    Failed,
    Abandoned,
}

/// Task assigned to an agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub task_type: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub priority: f32,
    pub estimated_duration: Option<u64>, // milliseconds
    pub deadline: Option<DateTime<Utc>>,
    pub required_capabilities: Vec<String>,
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for a task
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceRequirements {
    pub llm_calls: u32,
    pub memory_mb: u32,
    pub compute_units: u32,
    pub exclusive_resources: Vec<String>,
}

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: String,
    pub success: bool,
    pub result_data: serde_json::Value,
    pub execution_time_ms: u64,
    pub resources_used: ResourceUsage,
    pub error_message: Option<String>,
    pub side_effects: Vec<SideEffect>,
}

/// Side effect of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SideEffect {
    pub effect_type: String,
    pub description: String,
    pub affected_entities: Vec<String>,
    pub impact_level: f32,
}

/// Current resource usage by an agent
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceUsage {
    pub llm_calls_per_minute: u32,
    pub memory_mb: u32,
    pub active_tasks: u32,
    pub queue_size: u32,
    pub last_activity: DateTime<Utc>,
}

/// World event that agents can react to
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorldEvent {
    pub id: String,
    pub event_type: String,
    pub description: String,
    pub location: Option<String>,
    pub participants: Vec<String>,
    pub event_data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
    pub severity: EventSeverity,
}

/// Severity of world events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum EventSeverity {
    Trivial,
    Minor,
    Moderate,
    Major,
    Critical,
}

/// Base implementation for common agent functionality
pub struct BaseAgent {
    pub id: String,
    pub agent_type: String,
    pub state: AgentState,
    pub goals: Vec<AgentGoal>,
    pub capabilities: Vec<String>,
    pub resource_usage: ResourceUsage,
    pub event_subscriptions: Vec<String>,
    pub message_queue: Arc<RwLock<Vec<AgentMessage>>>,
    pub task_queue: Arc<RwLock<Vec<Task>>>,
    pub coordination_context: Option<Arc<RwLock<CoordinationContext>>>,
}

/// Context shared among coordinated agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoordinationContext {
    pub session_id: String,
    pub participants: Vec<String>, // Agent IDs
    pub shared_state: HashMap<String, serde_json::Value>,
    pub coordination_goals: Vec<AgentGoal>,
    pub start_time: DateTime<Utc>,
    pub last_update: DateTime<Utc>,
    pub status: CoordinationStatus,
}

/// Status of coordination session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum CoordinationStatus {
    Planning,
    Active,
    Synchronizing,
    Completing,
    Completed,
    Failed,
}

impl BaseAgent {
    pub fn new(agent_type: String, capabilities: Vec<String>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_type,
            state: AgentState::Idle,
            goals: Vec::new(),
            capabilities,
            resource_usage: ResourceUsage::default(),
            event_subscriptions: Vec::new(),
            message_queue: Arc::new(RwLock::new(Vec::new())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            coordination_context: None,
        }
    }

    /// Add a message to the agent's queue
    pub async fn receive_message(&self, message: AgentMessage) {
        let mut queue = self.message_queue.write().await;
        queue.push(message);

        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));
    }

    /// Get next message from queue
    pub async fn get_next_message(&self) -> Option<AgentMessage> {
        let mut queue = self.message_queue.write().await;
        queue.pop()
    }

    /// Add a task to the agent's queue
    pub async fn add_task(&self, task: Task) {
        let mut queue = self.task_queue.write().await;
        queue.push(task);

        // Sort by priority (highest first)
        queue.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
    }

    /// Get next task from queue
    pub async fn get_next_task(&self) -> Option<Task> {
        let mut queue = self.task_queue.write().await;
        queue.pop()
    }

    /// Create a response message
    pub fn create_response(&self, original: &AgentMessage, content: serde_json::Value) -> AgentMessage {
        AgentMessage {
            id: Uuid::new_v4().to_string(),
            from: self.id.clone(),
            to: original.from.clone(),
            message_type: MessageType::Response,
            content,
            timestamp: Utc::now(),
            priority: original.priority.clone(),
            reply_to: Some(original.id.clone()),
        }
    }

    /// Create a notification message
    pub fn create_notification(&self, to: String, content: serde_json::Value, priority: MessagePriority) -> AgentMessage {
        AgentMessage {
            id: Uuid::new_v4().to_string(),
            from: self.id.clone(),
            to,
            message_type: MessageType::Notification,
            content,
            timestamp: Utc::now(),
            priority,
            reply_to: None,
        }
    }

    /// Update resource usage
    pub fn update_resource_usage(&mut self, usage: ResourceUsage) {
        self.resource_usage = usage;
        self.resource_usage.last_activity = Utc::now();
    }

    /// Check if agent is available for new tasks
    pub fn is_available(&self) -> bool {
        matches!(self.state, AgentState::Idle) && self.resource_usage.active_tasks < 5
    }

    /// Check if agent can handle resource requirements
    pub fn can_handle_requirements(&self, requirements: &ResourceRequirements) -> bool {
        // Simple resource check - in practice would be more sophisticated
        self.resource_usage.memory_mb + requirements.memory_mb < 1024 && // 1GB limit
        self.resource_usage.llm_calls_per_minute + requirements.llm_calls < 60
    }

    /// Join coordination context
    pub async fn join_coordination(&mut self, context: Arc<RwLock<CoordinationContext>>) -> Result<()> {
        let mut ctx = context.write().await;
        if !ctx.participants.contains(&self.id) {
            ctx.participants.push(self.id.clone());
            ctx.last_update = Utc::now();
        }
        self.coordination_context = Some(context.clone());
        info!("Agent {} joined coordination session {}", self.id, ctx.session_id);
        Ok(())
    }

    /// Leave coordination context
    pub async fn leave_coordination(&mut self) -> Result<()> {
        if let Some(context) = &self.coordination_context {
            let mut ctx = context.write().await;
            ctx.participants.retain(|id| id != &self.id);
            ctx.last_update = Utc::now();
            info!("Agent {} left coordination session {}", self.id, ctx.session_id);
        }
        self.coordination_context = None;
        Ok(())
    }

    /// Update shared state in coordination context
    pub async fn update_shared_state(&self, key: String, value: serde_json::Value) -> Result<()> {
        if let Some(context) = &self.coordination_context {
            let mut ctx = context.write().await;
            ctx.shared_state.insert(key, value);
            ctx.last_update = Utc::now();
        }
        Ok(())
    }

    /// Get shared state from coordination context
    pub async fn get_shared_state(&self, key: &str) -> Option<serde_json::Value> {
        if let Some(context) = &self.coordination_context {
            let ctx = context.read().await;
            ctx.shared_state.get(key).cloned()
        } else {
            None
        }
    }
}

impl AgentGoal {
    pub fn new(description: String, goal_type: GoalType, priority: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            description,
            goal_type,
            priority,
            target_value: None,
            current_value: 0.0,
            deadline: None,
            dependencies: Vec::new(),
            status: GoalStatus::Active,
        }
    }

    /// Check if goal is satisfied
    pub fn is_satisfied(&self) -> bool {
        match self.goal_type {
            GoalType::Achieve => {
                if let Some(target) = self.target_value {
                    self.current_value >= target
                } else {
                    false
                }
            }
            GoalType::Maintain => {
                if let Some(target) = self.target_value {
                    (self.current_value - target).abs() < 0.1
                } else {
                    false
                }
            }
            GoalType::Avoid => self.current_value == 0.0,
            GoalType::Explore | GoalType::Collaborate => {
                matches!(self.status, GoalStatus::Completed)
            }
        }
    }

    /// Update goal progress
    pub fn update_progress(&mut self, new_value: f32) {
        self.current_value = new_value;

        if self.is_satisfied() && self.status == GoalStatus::Active {
            self.status = GoalStatus::Completed;
        }
    }

    /// Check if goal is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = &self.deadline {
            Utc::now() > *deadline && self.status != GoalStatus::Completed
        } else {
            false
        }
    }
}

impl Task {
    pub fn new(task_type: String, description: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            task_type,
            description,
            parameters: HashMap::new(),
            priority: 0.5,
            estimated_duration: None,
            deadline: None,
            required_capabilities: Vec::new(),
            resource_requirements: ResourceRequirements::default(),
        }
    }

    /// Add parameter to task
    pub fn with_parameter<T: Serialize>(mut self, key: String, value: T) -> Self {
        self.parameters.insert(key, serde_json::to_value(value).unwrap_or_default());
        self
    }

    /// Set priority
    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority.max(0.0).min(1.0);
        self
    }

    /// Add required capability
    pub fn with_capability(mut self, capability: String) -> Self {
        self.required_capabilities.push(capability);
        self
    }

    /// Check if task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(deadline) = &self.deadline {
            Utc::now() > *deadline
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_goal_satisfaction() {
        let mut goal = AgentGoal::new(
            "Reach level 10".to_string(),
            GoalType::Achieve,
            0.8,
        );
        goal.target_value = Some(10.0);
        goal.current_value = 5.0;

        assert!(!goal.is_satisfied());

        goal.update_progress(10.0);
        assert!(goal.is_satisfied());
        assert_eq!(goal.status, GoalStatus::Completed);
    }

    #[test]
    fn test_task_creation() {
        let task = Task::new("generate_dialogue".to_string(), "Generate NPC dialogue".to_string())
            .with_parameter("npc_id".to_string(), "npc_001")
            .with_priority(0.8)
            .with_capability("llm_completion".to_string());

        assert_eq!(task.task_type, "generate_dialogue");
        assert_eq!(task.priority, 0.8);
        assert_eq!(task.required_capabilities.len(), 1);
        assert!(task.parameters.contains_key("npc_id"));
    }

    #[tokio::test]
    async fn test_base_agent_messaging() {
        let agent = BaseAgent::new(
            "test_agent".to_string(),
            vec!["test_capability".to_string()],
        );

        let message = AgentMessage {
            id: Uuid::new_v4().to_string(),
            from: "sender".to_string(),
            to: agent.id.clone(),
            message_type: MessageType::Request,
            content: serde_json::json!({"test": "data"}),
            timestamp: Utc::now(),
            priority: MessagePriority::Normal,
            reply_to: None,
        };

        agent.receive_message(message.clone()).await;

        let received = agent.get_next_message().await;
        assert!(received.is_some());
        assert_eq!(received.unwrap().id, message.id);
    }

    #[tokio::test]
    async fn test_coordination_context() {
        let mut agent = BaseAgent::new(
            "test_agent".to_string(),
            vec!["coordination".to_string()],
        );

        let context = Arc::new(RwLock::new(CoordinationContext {
            session_id: Uuid::new_v4().to_string(),
            participants: Vec::new(),
            shared_state: HashMap::new(),
            coordination_goals: Vec::new(),
            start_time: Utc::now(),
            last_update: Utc::now(),
            status: CoordinationStatus::Planning,
        }));

        agent.join_coordination(context.clone()).await.unwrap();

        let ctx = context.read().await;
        assert!(ctx.participants.contains(&agent.id));

        drop(ctx);

        agent.update_shared_state("test_key".to_string(), serde_json::json!("test_value")).await.unwrap();

        let value = agent.get_shared_state("test_key").await;
        assert!(value.is_some());
        assert_eq!(value.unwrap(), serde_json::json!("test_value"));
    }
}