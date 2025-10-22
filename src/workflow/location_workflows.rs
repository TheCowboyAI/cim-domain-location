//! Predefined location-specific workflows

use std::collections::HashMap;
use uuid::Uuid;
use chrono::Utc;
use super::{
    WorkflowId, NodeId, WorkflowDefinition, WorkflowNode, NodeType,
    NodeTransition, TransitionCondition, WorkflowAction,
};

/// Create location verification workflow
pub fn create_location_verification_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new_named("location_verification");
    
    let submit_node = NodeId::from("submit");
    let review_node = NodeId::from("review");
    let verify_node = NodeId::from("verify");
    let approved_node = NodeId::from("approved");
    let rejected_node = NodeId::from("rejected");
    
    let mut nodes = HashMap::new();
    
    // Submit node
    nodes.insert(submit_node.clone(), WorkflowNode {
        id: submit_node.clone(),
        name: "Submit Location".to_string(),
        description: Some("Submit location for verification".to_string()),
        node_type: NodeType::Task,
        transitions: vec![NodeTransition {
            to_node: review_node.clone(),
            condition: Some(TransitionCondition::Always),
            label: Some("Submit for Review".to_string()),
        }],
        actions: vec![
            WorkflowAction {
                action_type: "notify_reviewers".to_string(),
                parameters: [("message".to_string(), serde_json::json!("New location submitted for review"))].into(),
            }
        ],
        required_permissions: vec!["location.submit".to_string()],
    });
    
    // Review node
    nodes.insert(review_node.clone(), WorkflowNode {
        id: review_node.clone(),
        name: "Review Location".to_string(),
        description: Some("Review location details".to_string()),
        node_type: NodeType::Task,
        transitions: vec![
            NodeTransition {
                to_node: verify_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "review_result".to_string(),
                    value: serde_json::json!("approved"),
                }),
                label: Some("Approve".to_string()),
            },
            NodeTransition {
                to_node: rejected_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "review_result".to_string(),
                    value: serde_json::json!("rejected"),
                }),
                label: Some("Reject".to_string()),
            },
        ],
        actions: vec![],
        required_permissions: vec!["location.review".to_string()],
    });
    
    // Verify node
    nodes.insert(verify_node.clone(), WorkflowNode {
        id: verify_node.clone(),
        name: "Verify Location".to_string(),
        description: Some("Perform technical verification".to_string()),
        node_type: NodeType::Task,
        transitions: vec![
            NodeTransition {
                to_node: approved_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "verification_result".to_string(),
                    value: serde_json::json!("verified"),
                }),
                label: Some("Verified".to_string()),
            },
            NodeTransition {
                to_node: rejected_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "verification_result".to_string(),
                    value: serde_json::json!("failed"),
                }),
                label: Some("Verification Failed".to_string()),
            },
        ],
        actions: vec![
            WorkflowAction {
                action_type: "geocode_address".to_string(),
                parameters: HashMap::new(),
            },
            WorkflowAction {
                action_type: "validate_coordinates".to_string(),
                parameters: HashMap::new(),
            },
        ],
        required_permissions: vec!["location.verify".to_string()],
    });
    
    // Approved node
    nodes.insert(approved_node.clone(), WorkflowNode {
        id: approved_node.clone(),
        name: "Location Approved".to_string(),
        description: Some("Location has been approved and verified".to_string()),
        node_type: NodeType::End,
        transitions: vec![],
        actions: vec![
            WorkflowAction {
                action_type: "activate_location".to_string(),
                parameters: HashMap::new(),
            },
            WorkflowAction {
                action_type: "notify_submitter".to_string(),
                parameters: [("status".to_string(), serde_json::json!("approved"))].into(),
            },
        ],
        required_permissions: vec![],
    });
    
    // Rejected node
    nodes.insert(rejected_node.clone(), WorkflowNode {
        id: rejected_node.clone(),
        name: "Location Rejected".to_string(),
        description: Some("Location has been rejected".to_string()),
        node_type: NodeType::End,
        transitions: vec![],
        actions: vec![
            WorkflowAction {
                action_type: "notify_submitter".to_string(),
                parameters: [("status".to_string(), serde_json::json!("rejected"))].into(),
            },
        ],
        required_permissions: vec![],
    });
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Location Verification".to_string(),
        description: Some("Workflow for verifying new location submissions".to_string()),
        version: "1.0".to_string(),
        nodes,
        start_node: submit_node,
        end_nodes: vec![approved_node, rejected_node],
        created_at: Utc::now(),
        created_by: Uuid::nil(), // System-created workflow
    }
}

/// Create hierarchy reorganization workflow
pub fn create_hierarchy_reorganization_workflow() -> WorkflowDefinition {
    let workflow_id = WorkflowId::new_named("hierarchy_reorganization");
    
    let plan_node = NodeId::from("plan");
    let validate_node = NodeId::from("validate");
    let execute_node = NodeId::from("execute");
    let completed_node = NodeId::from("completed");
    let failed_node = NodeId::from("failed");
    
    let mut nodes = HashMap::new();
    
    // Plan reorganization
    nodes.insert(plan_node.clone(), WorkflowNode {
        id: plan_node.clone(),
        name: "Plan Reorganization".to_string(),
        description: Some("Plan the hierarchy reorganization".to_string()),
        node_type: NodeType::Task,
        transitions: vec![NodeTransition {
            to_node: validate_node.clone(),
            condition: Some(TransitionCondition::Always),
            label: Some("Validate Plan".to_string()),
        }],
        actions: vec![],
        required_permissions: vec!["hierarchy.plan".to_string()],
    });
    
    // Validate reorganization
    nodes.insert(validate_node.clone(), WorkflowNode {
        id: validate_node.clone(),
        name: "Validate Reorganization".to_string(),
        description: Some("Validate the reorganization plan".to_string()),
        node_type: NodeType::Decision,
        transitions: vec![
            NodeTransition {
                to_node: execute_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "validation_result".to_string(),
                    value: serde_json::json!("valid"),
                }),
                label: Some("Valid - Execute".to_string()),
            },
            NodeTransition {
                to_node: failed_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "validation_result".to_string(),
                    value: serde_json::json!("invalid"),
                }),
                label: Some("Invalid - Fail".to_string()),
            },
        ],
        actions: vec![
            WorkflowAction {
                action_type: "validate_hierarchy_changes".to_string(),
                parameters: HashMap::new(),
            },
        ],
        required_permissions: vec![],
    });
    
    // Execute reorganization
    nodes.insert(execute_node.clone(), WorkflowNode {
        id: execute_node.clone(),
        name: "Execute Reorganization".to_string(),
        description: Some("Execute the hierarchy reorganization".to_string()),
        node_type: NodeType::Task,
        transitions: vec![
            NodeTransition {
                to_node: completed_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "execution_result".to_string(),
                    value: serde_json::json!("success"),
                }),
                label: Some("Success".to_string()),
            },
            NodeTransition {
                to_node: failed_node.clone(),
                condition: Some(TransitionCondition::VariableEquals {
                    name: "execution_result".to_string(),
                    value: serde_json::json!("failed"),
                }),
                label: Some("Failed".to_string()),
            },
        ],
        actions: vec![
            WorkflowAction {
                action_type: "update_parent_child_relationships".to_string(),
                parameters: HashMap::new(),
            },
            WorkflowAction {
                action_type: "rebuild_hierarchy_index".to_string(),
                parameters: HashMap::new(),
            },
        ],
        required_permissions: vec!["hierarchy.execute".to_string()],
    });
    
    // Completed node
    nodes.insert(completed_node.clone(), WorkflowNode {
        id: completed_node.clone(),
        name: "Reorganization Completed".to_string(),
        description: Some("Hierarchy reorganization completed successfully".to_string()),
        node_type: NodeType::End,
        transitions: vec![],
        actions: vec![
            WorkflowAction {
                action_type: "notify_stakeholders".to_string(),
                parameters: [("status".to_string(), serde_json::json!("completed"))].into(),
            },
        ],
        required_permissions: vec![],
    });
    
    // Failed node
    nodes.insert(failed_node.clone(), WorkflowNode {
        id: failed_node.clone(),
        name: "Reorganization Failed".to_string(),
        description: Some("Hierarchy reorganization failed".to_string()),
        node_type: NodeType::End,
        transitions: vec![],
        actions: vec![
            WorkflowAction {
                action_type: "rollback_changes".to_string(),
                parameters: HashMap::new(),
            },
            WorkflowAction {
                action_type: "notify_stakeholders".to_string(),
                parameters: [("status".to_string(), serde_json::json!("failed"))].into(),
            },
        ],
        required_permissions: vec![],
    });
    
    WorkflowDefinition {
        id: workflow_id,
        name: "Hierarchy Reorganization".to_string(),
        description: Some("Workflow for reorganizing location hierarchies".to_string()),
        version: "1.0".to_string(),
        nodes,
        start_node: plan_node,
        end_nodes: vec![completed_node, failed_node],
        created_at: Utc::now(),
        created_by: Uuid::nil(), // System-created workflow
    }
}

/// Get all predefined location workflows
pub fn get_predefined_workflows() -> Vec<WorkflowDefinition> {
    vec![
        create_location_verification_workflow(),
        create_hierarchy_reorganization_workflow(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_verification_workflow() {
        let workflow = create_location_verification_workflow();
        
        assert!(workflow.validate().is_ok());
        assert_eq!(workflow.name, "Location Verification");
        assert_eq!(workflow.end_nodes.len(), 2); // approved and rejected
        
        // Check that all nodes exist
        assert!(workflow.nodes.contains_key(&NodeId::from("submit")));
        assert!(workflow.nodes.contains_key(&NodeId::from("review")));
        assert!(workflow.nodes.contains_key(&NodeId::from("verify")));
        assert!(workflow.nodes.contains_key(&NodeId::from("approved")));
        assert!(workflow.nodes.contains_key(&NodeId::from("rejected")));
    }
    
    #[test]
    fn test_hierarchy_reorganization_workflow() {
        let workflow = create_hierarchy_reorganization_workflow();
        
        assert!(workflow.validate().is_ok());
        assert_eq!(workflow.name, "Hierarchy Reorganization");
        assert_eq!(workflow.end_nodes.len(), 2); // completed and failed
        
        // Check that all nodes exist
        assert!(workflow.nodes.contains_key(&NodeId::from("plan")));
        assert!(workflow.nodes.contains_key(&NodeId::from("validate")));
        assert!(workflow.nodes.contains_key(&NodeId::from("execute")));
        assert!(workflow.nodes.contains_key(&NodeId::from("completed")));
        assert!(workflow.nodes.contains_key(&NodeId::from("failed")));
    }
    
    #[test]
    fn test_predefined_workflows() {
        let workflows = get_predefined_workflows();
        
        assert_eq!(workflows.len(), 2);
        
        for workflow in workflows {
            assert!(workflow.validate().is_ok());
        }
    }
}