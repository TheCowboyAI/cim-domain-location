//! Workflow management and execution

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::{
    WorkflowId, WorkflowInstanceId, NodeId, WorkflowStatus, WorkflowContext, 
    WorkflowTransition, NodeStatus, WorkflowResult, WorkflowError,
    WorkflowDefinition,
};

/// Workflow manager trait
#[async_trait]
pub trait WorkflowManager: Send + Sync {
    /// Start a new workflow instance
    async fn start_workflow(
        &self,
        workflow_id: &WorkflowId,
        context: WorkflowContext,
    ) -> WorkflowResult<WorkflowInstance>;
    
    /// Get workflow instance
    async fn get_instance(&self, instance_id: &WorkflowInstanceId) -> WorkflowResult<WorkflowInstance>;
    
    /// Advance workflow to next node
    async fn advance_workflow(
        &self,
        instance_id: &WorkflowInstanceId,
        target_node: &NodeId,
        context: Option<WorkflowContext>,
    ) -> WorkflowResult<WorkflowInstance>;
    
    /// Complete current node and advance
    async fn complete_node(
        &self,
        instance_id: &WorkflowInstanceId,
        user_id: Option<Uuid>,
        completion_data: Option<serde_json::Value>,
    ) -> WorkflowResult<WorkflowInstance>;
    
    /// Cancel workflow instance
    async fn cancel_workflow(
        &self,
        instance_id: &WorkflowInstanceId,
        reason: Option<String>,
    ) -> WorkflowResult<WorkflowInstance>;
    
    /// Get workflow history
    async fn get_history(&self, instance_id: &WorkflowInstanceId) -> WorkflowResult<Vec<WorkflowTransition>>;
}

/// Workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInstance {
    /// Instance identifier
    pub id: WorkflowInstanceId,
    /// Workflow definition ID
    pub workflow_id: WorkflowId,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Current node
    pub current_node: NodeId,
    /// Execution context
    pub context: WorkflowContext,
    /// Node statuses
    pub node_statuses: HashMap<NodeId, NodeStatus>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last updated timestamp
    pub updated_at: DateTime<Utc>,
    /// Completion timestamp
    pub completed_at: Option<DateTime<Utc>>,
}

impl WorkflowInstance {
    /// Create new workflow instance
    pub fn new(
        workflow_id: WorkflowId,
        start_node: NodeId,
        context: WorkflowContext,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: WorkflowInstanceId::new(),
            workflow_id,
            status: WorkflowStatus::Running,
            current_node: start_node,
            context,
            node_statuses: HashMap::new(),
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
    
    /// Check if workflow is completed
    pub fn is_completed(&self) -> bool {
        matches!(self.status, WorkflowStatus::Completed)
    }
    
    /// Check if workflow is running
    pub fn is_running(&self) -> bool {
        matches!(self.status, WorkflowStatus::Running)
    }
    
    /// Update node status
    pub fn set_node_status(&mut self, node_id: NodeId, status: NodeStatus) {
        self.node_statuses.insert(node_id, status);
        self.updated_at = Utc::now();
    }
    
    /// Get node status
    pub fn get_node_status(&self, node_id: &NodeId) -> NodeStatus {
        self.node_statuses.get(node_id)
            .cloned()
            .unwrap_or(NodeStatus::Pending)
    }
}

/// Mock workflow manager for testing
pub struct MockWorkflowManager {
    definitions: Arc<RwLock<HashMap<WorkflowId, WorkflowDefinition>>>,
    instances: Arc<RwLock<HashMap<WorkflowInstanceId, WorkflowInstance>>>,
    transitions: Arc<RwLock<HashMap<WorkflowInstanceId, Vec<WorkflowTransition>>>>,
}

impl MockWorkflowManager {
    pub fn new() -> Self {
        Self {
            definitions: Arc::new(RwLock::new(HashMap::new())),
            instances: Arc::new(RwLock::new(HashMap::new())),
            transitions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn add_definition(&self, definition: WorkflowDefinition) {
        let mut definitions = self.definitions.write().await;
        definitions.insert(definition.id.clone(), definition);
    }

    async fn get_definition(&self, workflow_id: &WorkflowId) -> WorkflowResult<WorkflowDefinition> {
        let definitions = self.definitions.read().await;
        definitions.get(workflow_id).cloned().ok_or_else(|| WorkflowError::WorkflowNotFound {
            workflow_id: workflow_id.as_str(),
        })
    }
}

impl Default for MockWorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowManager for MockWorkflowManager {
    async fn start_workflow(
        &self,
        workflow_id: &WorkflowId,
        context: WorkflowContext,
    ) -> WorkflowResult<WorkflowInstance> {
        let definition = self.get_definition(workflow_id).await?;

        let mut instance = WorkflowInstance::new(
            workflow_id.clone(),
            definition.start_node.clone(),
            context,
        );

        // Set start node as active
        instance.set_node_status(definition.start_node.clone(), NodeStatus::Active);

        let instance_id = instance.id;
        let mut instances = self.instances.write().await;
        instances.insert(instance_id, instance.clone());
        let mut transitions = self.transitions.write().await;
        transitions.insert(instance_id, Vec::new());

        Ok(instance)
    }
    
    async fn get_instance(&self, instance_id: &WorkflowInstanceId) -> WorkflowResult<WorkflowInstance> {
        let instances = self.instances.read().await;
        instances.get(instance_id)
            .cloned()
            .ok_or_else(|| WorkflowError::WorkflowNotFound {
                workflow_id: instance_id.as_uuid().to_string(),
            })
    }
    
    async fn advance_workflow(
        &self,
        instance_id: &WorkflowInstanceId,
        target_node: &NodeId,
        context: Option<WorkflowContext>,
    ) -> WorkflowResult<WorkflowInstance> {
        let mut instance = self.get_instance(instance_id).await?;
        let definition = self.get_definition(&instance.workflow_id).await?;
        
        // Validate transition is allowed
        let current_node = definition.get_node(&instance.current_node)
            .ok_or_else(|| WorkflowError::InvalidTransition {
                from: instance.current_node.as_str().to_string(),
                to: target_node.as_str().to_string(),
                reason: "Current node not found".to_string(),
            })?;
        
        if !current_node.can_transition_to(target_node) {
            return Err(WorkflowError::InvalidTransition {
                from: instance.current_node.as_str().to_string(),
                to: target_node.as_str().to_string(),
                reason: "Transition not allowed".to_string(),
            });
        }
        
        // Record transition
        let transition = WorkflowTransition {
            id: Uuid::new_v4(),
            from_node: instance.current_node.clone(),
            to_node: target_node.clone(),
            transitioned_at: Utc::now(),
            transitioned_by: instance.context.initiated_by,
            reason: None,
            data: HashMap::new(),
        };
        
        // Update instance
        instance.set_node_status(instance.current_node.clone(), NodeStatus::Completed);
        instance.current_node = target_node.clone();
        instance.set_node_status(target_node.clone(), NodeStatus::Active);
        
        if let Some(new_context) = context {
            instance.context = new_context;
        }
        
        // Check if workflow is complete
        if definition.end_nodes.contains(target_node) {
            instance.status = WorkflowStatus::Completed;
            instance.completed_at = Some(Utc::now());
        }
        
        // Store updates
        let mut instances = self.instances.write().await;
        instances.insert(*instance_id, instance.clone());
        let mut transitions = self.transitions.write().await;
        transitions.entry(*instance_id).or_default().push(transition);

        Ok(instance)
    }
    
    async fn complete_node(
        &self,
        instance_id: &WorkflowInstanceId,
        _user_id: Option<Uuid>,
        _completion_data: Option<serde_json::Value>,
    ) -> WorkflowResult<WorkflowInstance> {
        let instance = self.get_instance(instance_id).await?;
        let definition = self.get_definition(&instance.workflow_id).await?;
        
        // Find next node based on transitions
        let current_node = definition.get_node(&instance.current_node)
            .ok_or_else(|| WorkflowError::InvalidTransition {
                from: instance.current_node.as_str().to_string(),
                to: "unknown".to_string(),
                reason: "Current node not found".to_string(),
            })?;
        
        // For simplicity, take first available transition
        if let Some(transition) = current_node.transitions.first() {
            self.advance_workflow(instance_id, &transition.to_node, None).await
        } else {
            // No transitions available, mark as completed
            let mut updated_instance = instance;
            updated_instance.status = WorkflowStatus::Completed;
            updated_instance.completed_at = Some(Utc::now());
            updated_instance.set_node_status(updated_instance.current_node.clone(), NodeStatus::Completed);
            
            let mut instances = self.instances.write().await;
            instances.insert(*instance_id, updated_instance.clone());
            Ok(updated_instance)
        }
    }
    
    async fn cancel_workflow(
        &self,
        instance_id: &WorkflowInstanceId,
        _reason: Option<String>,
    ) -> WorkflowResult<WorkflowInstance> {
        let mut instance = self.get_instance(instance_id).await?;
        
        instance.status = WorkflowStatus::Cancelled;
        instance.completed_at = Some(Utc::now());
        instance.updated_at = Utc::now();
        
        let mut instances = self.instances.write().await;
        instances.insert(*instance_id, instance.clone());

        Ok(instance)
    }

    async fn get_history(&self, instance_id: &WorkflowInstanceId) -> WorkflowResult<Vec<WorkflowTransition>> {
        let transitions = self.transitions.read().await;
        Ok(transitions.get(instance_id).cloned().unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowDefinition, WorkflowNode, NodeType, NodeTransition, TransitionCondition};

    #[tokio::test]
    async fn test_workflow_execution() {
        let mut manager = MockWorkflowManager::new();
        
        // Create simple workflow definition
        let workflow_id = WorkflowId::new();
        let start_node = NodeId::from("start");
        let end_node = NodeId::from("end");
        
        let mut nodes = HashMap::new();
        nodes.insert(start_node.clone(), WorkflowNode {
            id: start_node.clone(),
            name: "Start".to_string(),
            description: None,
            node_type: NodeType::Start,
            transitions: vec![NodeTransition {
                to_node: end_node.clone(),
                condition: Some(TransitionCondition::Always),
                label: Some("Complete".to_string()),
            }],
            actions: vec![],
            required_permissions: vec![],
        });
        
        nodes.insert(end_node.clone(), WorkflowNode {
            id: end_node.clone(),
            name: "End".to_string(),
            description: None,
            node_type: NodeType::End,
            transitions: vec![],
            actions: vec![],
            required_permissions: vec![],
        });
        
        let definition = WorkflowDefinition {
            id: workflow_id.clone(),
            name: "Test Workflow".to_string(),
            description: None,
            version: "1.0".to_string(),
            nodes,
            start_node: start_node.clone(),
            end_nodes: vec![end_node.clone()],
            created_at: Utc::now(),
            created_by: Uuid::new_v4(),
        };
        
        manager.add_definition(definition);
        
        // Start workflow
        let context = WorkflowContext::new();
        let instance = manager.start_workflow(&workflow_id, context).await.unwrap();
        
        assert_eq!(instance.status, WorkflowStatus::Running);
        assert_eq!(instance.current_node, start_node);
        
        // Advance to end node
        let completed_instance = manager.advance_workflow(&instance.id, &end_node, None).await.unwrap();
        
        assert_eq!(completed_instance.status, WorkflowStatus::Completed);
        assert_eq!(completed_instance.current_node, end_node);
        assert!(completed_instance.completed_at.is_some());
    }
}