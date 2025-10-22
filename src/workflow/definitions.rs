//! Workflow definitions for location workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::{WorkflowId, NodeId, WorkflowResult, WorkflowError};

/// Workflow definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowDefinition {
    /// Workflow identifier
    pub id: WorkflowId,
    /// Human-readable name
    pub name: String,
    /// Workflow description
    pub description: Option<String>,
    /// Version of this workflow
    pub version: String,
    /// Workflow nodes
    pub nodes: HashMap<NodeId, WorkflowNode>,
    /// Initial node
    pub start_node: NodeId,
    /// Final nodes
    pub end_nodes: Vec<NodeId>,
    /// Created timestamp
    pub created_at: DateTime<Utc>,
    /// Created by user
    pub created_by: Uuid,
}

/// Workflow node definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Node identifier
    pub id: NodeId,
    /// Human-readable name
    pub name: String,
    /// Node description
    pub description: Option<String>,
    /// Node type
    pub node_type: NodeType,
    /// Possible transitions from this node
    pub transitions: Vec<NodeTransition>,
    /// Actions to execute when entering this node
    pub actions: Vec<WorkflowAction>,
    /// Required permissions to complete this node
    pub required_permissions: Vec<String>,
}

/// Types of workflow nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeType {
    /// Start node (entry point)
    Start,
    /// Task node (requires user action)
    Task,
    /// Decision node (automatic routing based on conditions)
    Decision,
    /// End node (terminal state)
    End,
    /// Parallel gateway (split into multiple paths)
    ParallelGateway,
    /// Merge gateway (join multiple paths)
    MergeGateway,
}

/// Node transition definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTransition {
    /// Target node
    pub to_node: NodeId,
    /// Condition that must be met for this transition
    pub condition: Option<TransitionCondition>,
    /// Human-readable label
    pub label: Option<String>,
}

/// Conditions for workflow transitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionCondition {
    /// Always allow transition
    Always,
    /// Check variable value
    VariableEquals { name: String, value: serde_json::Value },
    /// Check if user has permission
    HasPermission { permission: String },
    /// Custom expression
    Expression { expression: String },
}

/// Workflow actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowAction {
    /// Action type identifier
    pub action_type: String,
    /// Action parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

impl WorkflowDefinition {
    /// Validate workflow definition
    pub fn validate(&self) -> WorkflowResult<()> {
        // Check that start node exists
        if !self.nodes.contains_key(&self.start_node) {
            return Err(WorkflowError::InvalidDefinition {
                reason: format!("Start node '{}' not found", self.start_node.as_str()),
            });
        }
        
        // Check that all end nodes exist
        for end_node in &self.end_nodes {
            if !self.nodes.contains_key(end_node) {
                return Err(WorkflowError::InvalidDefinition {
                    reason: format!("End node '{}' not found", end_node.as_str()),
                });
            }
        }
        
        // Check that all transition targets exist
        for (node_id, node) in &self.nodes {
            for transition in &node.transitions {
                if !self.nodes.contains_key(&transition.to_node) {
                    return Err(WorkflowError::InvalidDefinition {
                        reason: format!(
                            "Node '{}' has transition to non-existent node '{}'",
                            node_id.as_str(),
                            transition.to_node.as_str()
                        ),
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Get node by ID
    pub fn get_node(&self, node_id: &NodeId) -> Option<&WorkflowNode> {
        self.nodes.get(node_id)
    }
    
    /// Find all possible next nodes from current node
    pub fn get_next_nodes(&self, current_node: &NodeId) -> Vec<&NodeId> {
        if let Some(node) = self.nodes.get(current_node) {
            node.transitions.iter().map(|t| &t.to_node).collect()
        } else {
            Vec::new()
        }
    }
}

impl WorkflowNode {
    /// Check if this node can transition to target node
    pub fn can_transition_to(&self, target_node: &NodeId) -> bool {
        self.transitions.iter().any(|t| &t.to_node == target_node)
    }
    
    /// Get transition to specific node
    pub fn get_transition_to(&self, target_node: &NodeId) -> Option<&NodeTransition> {
        self.transitions.iter().find(|t| &t.to_node == target_node)
    }
}

impl TransitionCondition {
    /// Evaluate condition against workflow context
    pub fn evaluate(&self, context: &serde_json::Value) -> bool {
        match self {
            TransitionCondition::Always => true,
            TransitionCondition::VariableEquals { name, value } => {
                context.get(name).map_or(false, |v| v == value)
            },
            TransitionCondition::HasPermission { .. } => {
                // Mock implementation - would check user permissions
                true
            },
            TransitionCondition::Expression { .. } => {
                // Mock implementation - would evaluate expression
                true
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow::{WorkflowId, NodeId};

    #[test]
    fn test_workflow_definition_validation() {
        let workflow_id = WorkflowId::new();
        let start_node_id = NodeId::from("start");
        let end_node_id = NodeId::from("end");
        
        let mut nodes = HashMap::new();
        nodes.insert(start_node_id.clone(), WorkflowNode {
            id: start_node_id.clone(),
            name: "Start".to_string(),
            description: None,
            node_type: NodeType::Start,
            transitions: vec![NodeTransition {
                to_node: end_node_id.clone(),
                condition: Some(TransitionCondition::Always),
                label: Some("Complete".to_string()),
            }],
            actions: vec![],
            required_permissions: vec![],
        });
        nodes.insert(end_node_id.clone(), WorkflowNode {
            id: end_node_id.clone(),
            name: "End".to_string(),
            description: None,
            node_type: NodeType::End,
            transitions: vec![],
            actions: vec![],
            required_permissions: vec![],
        });
        
        let workflow = WorkflowDefinition {
            id: workflow_id,
            name: "Test Workflow".to_string(),
            description: None,
            version: "1.0".to_string(),
            nodes,
            start_node: start_node_id,
            end_nodes: vec![end_node_id],
            created_at: chrono::Utc::now(),
            created_by: Uuid::new_v4(),
        };
        
        assert!(workflow.validate().is_ok());
    }
    
    #[test]
    fn test_transition_condition_evaluation() {
        let condition = TransitionCondition::VariableEquals {
            name: "status".to_string(),
            value: serde_json::json!("approved"),
        };
        
        let context = serde_json::json!({
            "status": "approved"
        });
        
        assert!(condition.evaluate(&context));
        
        let wrong_context = serde_json::json!({
            "status": "pending"
        });
        
        assert!(!condition.evaluate(&wrong_context));
    }
}