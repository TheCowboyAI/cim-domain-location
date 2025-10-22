//! Location Workflow Management
//!
//! This module implements workflow state machines for location-based processes
//! such as location verification, approval workflows, and hierarchical reorganization.

pub mod definitions;
pub mod manager;
pub mod location_workflows;

pub use definitions::*;
pub use manager::*;
pub use location_workflows::*;

use uuid::Uuid;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Unique identifier for workflow definitions
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowId(Uuid);

impl WorkflowId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn new_named(name: &str) -> Self {
        // Create deterministic UUID from name for consistent workflow IDs
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        name.hash(&mut hasher);
        let hash = hasher.finish();
        
        // Convert hash to UUID format (simplified approach)
        let uuid_bytes = [
            (hash >> 56) as u8, (hash >> 48) as u8, (hash >> 40) as u8, (hash >> 32) as u8,
            (hash >> 24) as u8, (hash >> 16) as u8, (hash >> 8) as u8, hash as u8,
            0, 0, 0, 0, 0, 0, 0, 0
        ];
        
        Self(Uuid::from_bytes(uuid_bytes))
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
    
    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for WorkflowId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for workflow instances
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowInstanceId(Uuid);

impl WorkflowInstanceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for WorkflowInstanceId {
    fn default() -> Self {
        Self::new()
    }
}

/// Unique identifier for workflow nodes
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Workflow execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is actively running
    Running,
    /// Workflow is waiting for user input
    Waiting,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed(String),
    /// Workflow was cancelled
    Cancelled,
}

/// Workflow execution context containing runtime data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowContext {
    /// Dynamic variables available to workflow
    pub variables: HashMap<String, serde_json::Value>,
    /// Current location being processed
    pub location_id: Option<Uuid>,
    /// User who initiated the workflow
    pub initiated_by: Option<Uuid>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl WorkflowContext {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            location_id: None,
            initiated_by: None,
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_location(mut self, location_id: Uuid) -> Self {
        self.location_id = Some(location_id);
        self
    }
    
    pub fn with_user(mut self, user_id: Uuid) -> Self {
        self.initiated_by = Some(user_id);
        self
    }
    
    pub fn set_variable(&mut self, key: String, value: serde_json::Value) {
        self.variables.insert(key, value);
    }
    
    pub fn get_variable(&self, key: &str) -> Option<&serde_json::Value> {
        self.variables.get(key)
    }
}

impl Default for WorkflowContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Record of a workflow state transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTransition {
    /// Unique identifier for this transition
    pub id: Uuid,
    /// Source node
    pub from_node: NodeId,
    /// Destination node
    pub to_node: NodeId,
    /// When transition occurred
    pub transitioned_at: DateTime<Utc>,
    /// Who triggered the transition
    pub transitioned_by: Option<Uuid>,
    /// Reason for transition
    pub reason: Option<String>,
    /// Additional transition data
    pub data: HashMap<String, serde_json::Value>,
}

/// Status of individual workflow nodes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is waiting to be activated
    Pending,
    /// Node is currently active
    Active,
    /// Node has been completed
    Completed,
    /// Node was skipped
    Skipped,
    /// Node failed
    Failed(String),
}

/// Error types for workflow operations
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum WorkflowError {
    #[error("Workflow not found: {workflow_id}")]
    WorkflowNotFound { workflow_id: String },
    
    #[error("Invalid transition from {from} to {to}: {reason}")]
    InvalidTransition {
        from: String,
        to: String,
        reason: String,
    },
    
    #[error("Invalid workflow definition: {reason}")]
    InvalidDefinition { reason: String },
    
    #[error("Workflow engine error: {message}")]
    EngineError { message: String },
    
    #[error("Location not found: {location_id}")]
    LocationNotFound { location_id: Uuid },
    
    #[error("Permission denied for user: {user_id}")]
    PermissionDenied { user_id: Uuid },
}

pub type WorkflowResult<T> = Result<T, WorkflowError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_id_creation() {
        let id1 = WorkflowId::new();
        let id2 = WorkflowId::new();
        assert_ne!(id1, id2);
        
        let named_id1 = WorkflowId::new_named("test_workflow");
        let named_id2 = WorkflowId::new_named("test_workflow");
        assert_eq!(named_id1, named_id2); // Should be deterministic
    }
    
    #[test]
    fn test_workflow_context() {
        let mut context = WorkflowContext::new();
        
        let location_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        context = context.with_location(location_id).with_user(user_id);
        context.set_variable("test_var".into(), serde_json::json!("test_value"));
        
        assert_eq!(context.location_id, Some(location_id));
        assert_eq!(context.initiated_by, Some(user_id));
        assert_eq!(context.get_variable("test_var"), Some(&serde_json::json!("test_value")));
    }
}