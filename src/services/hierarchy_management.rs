//! Hierarchy management services for location relationships

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use thiserror::Error;

/// Hierarchy management service trait
#[async_trait]
pub trait HierarchyManagementService: Send + Sync {
    /// Build a hierarchy tree for a location
    async fn build_hierarchy_tree(&self, root_id: &Uuid) -> Result<HierarchyTree, HierarchyError>;
    
    /// Validate hierarchy operations (prevent cycles, etc.)
    async fn validate_hierarchy_operation(&self, operation: &HierarchyOperation) -> Result<ValidationResult, HierarchyError>;
    
    /// Reorganize a branch of the hierarchy
    async fn reorganize_branch(&self, branch_root: &Uuid, new_structure: &HierarchyStructure) -> Result<ReorganizationResult, HierarchyError>;
    
    /// Find all ancestors of a location
    async fn find_ancestors(&self, location_id: &Uuid) -> Result<Vec<HierarchyNode>, HierarchyError>;
    
    /// Find all descendants of a location
    async fn find_descendants(&self, location_id: &Uuid, max_depth: Option<u32>) -> Result<Vec<HierarchyNode>, HierarchyError>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyTree {
    pub root: HierarchyNode,
    pub total_nodes: u32,
    pub max_depth: u32,
    pub metadata: HierarchyMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyNode {
    pub location_id: Uuid,
    pub name: Option<String>,
    pub level: u32,
    pub children: Vec<HierarchyNode>,
    pub node_type: HierarchyNodeType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyNodeType {
    Root,
    Branch,
    Leaf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyMetadata {
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub version: String,
    pub balance_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyOperation {
    pub operation_type: HierarchyOperationType,
    pub target_location: Uuid,
    pub parent_location: Option<Uuid>,
    pub new_parent_location: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HierarchyOperationType {
    SetParent,
    RemoveParent,
    MoveToParent,
    Delete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub issue_type: ValidationIssueType,
    pub message: String,
    pub affected_locations: Vec<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    CircularReference,
    InvalidParent,
    MaxDepthExceeded,
    OrphanedLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HierarchyStructure {
    pub nodes: Vec<StructureNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructureNode {
    pub location_id: Uuid,
    pub parent_id: Option<Uuid>,
    pub order: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReorganizationResult {
    pub success: bool,
    pub affected_locations: Vec<Uuid>,
    pub changes_applied: u32,
    pub execution_time_ms: u64,
}

#[derive(Debug, Error)]
pub enum HierarchyError {
    #[error("Circular reference detected: {0}")]
    CircularReference(String),
    
    #[error("Location not found: {0}")]
    LocationNotFound(Uuid),
    
    #[error("Invalid hierarchy operation: {0}")]
    InvalidOperation(String),
    
    #[error("Service unavailable: {0}")]
    ServiceUnavailable(String),
}

/// Mock hierarchy management service
pub struct MockHierarchyManagementService;

impl Default for MockHierarchyManagementService {
    fn default() -> Self {
        Self
    }
}

#[async_trait]
impl HierarchyManagementService for MockHierarchyManagementService {
    async fn build_hierarchy_tree(&self, root_id: &Uuid) -> Result<HierarchyTree, HierarchyError> {
        // Mock tree structure
        let child_id = Uuid::new_v4();
        let grandchild_id = Uuid::new_v4();
        
        let root = HierarchyNode {
            location_id: *root_id,
            name: Some("Root Location".to_string()),
            level: 0,
            children: vec![
                HierarchyNode {
                    location_id: child_id,
                    name: Some("Child Location".to_string()),
                    level: 1,
                    children: vec![
                        HierarchyNode {
                            location_id: grandchild_id,
                            name: Some("Grandchild Location".to_string()),
                            level: 2,
                            children: vec![],
                            node_type: HierarchyNodeType::Leaf,
                        }
                    ],
                    node_type: HierarchyNodeType::Branch,
                }
            ],
            node_type: HierarchyNodeType::Root,
        };
        
        Ok(HierarchyTree {
            root,
            total_nodes: 3,
            max_depth: 2,
            metadata: HierarchyMetadata {
                created_at: chrono::Utc::now(),
                version: "1.0".to_string(),
                balance_factor: 0.8,
            },
        })
    }
    
    async fn validate_hierarchy_operation(&self, operation: &HierarchyOperation) -> Result<ValidationResult, HierarchyError> {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();
        
        // Mock validation - check for self-parenting
        if let Some(parent_id) = operation.parent_location {
            if parent_id == operation.target_location {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::CircularReference,
                    message: "Location cannot be its own parent".to_string(),
                    affected_locations: vec![operation.target_location],
                });
            }
        }
        
        // Mock warning for deep hierarchies
        warnings.push("This operation may create a deep hierarchy".to_string());
        
        Ok(ValidationResult {
            is_valid: issues.is_empty(),
            issues,
            warnings,
        })
    }
    
    async fn reorganize_branch(&self, branch_root: &Uuid, _new_structure: &HierarchyStructure) -> Result<ReorganizationResult, HierarchyError> {
        Ok(ReorganizationResult {
            success: true,
            affected_locations: vec![*branch_root],
            changes_applied: 1,
            execution_time_ms: 100,
        })
    }
    
    async fn find_ancestors(&self, location_id: &Uuid) -> Result<Vec<HierarchyNode>, HierarchyError> {
        // Mock ancestor chain
        Ok(vec![
            HierarchyNode {
                location_id: Uuid::new_v4(),
                name: Some("Parent Location".to_string()),
                level: 1,
                children: vec![],
                node_type: HierarchyNodeType::Branch,
            },
            HierarchyNode {
                location_id: Uuid::new_v4(),
                name: Some("Grandparent Location".to_string()),
                level: 0,
                children: vec![],
                node_type: HierarchyNodeType::Root,
            },
        ])
    }
    
    async fn find_descendants(&self, _location_id: &Uuid, _max_depth: Option<u32>) -> Result<Vec<HierarchyNode>, HierarchyError> {
        // Mock descendant list
        Ok(vec![
            HierarchyNode {
                location_id: Uuid::new_v4(),
                name: Some("Child Location 1".to_string()),
                level: 1,
                children: vec![],
                node_type: HierarchyNodeType::Leaf,
            },
            HierarchyNode {
                location_id: Uuid::new_v4(),
                name: Some("Child Location 2".to_string()),
                level: 1,
                children: vec![],
                node_type: HierarchyNodeType::Leaf,
            },
        ])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_build_hierarchy_tree() {
        let service = MockHierarchyManagementService;
        let root_id = Uuid::new_v4();
        
        let tree = service.build_hierarchy_tree(&root_id).await.unwrap();
        
        assert_eq!(tree.root.location_id, root_id);
        assert_eq!(tree.total_nodes, 3);
        assert_eq!(tree.max_depth, 2);
    }
    
    #[tokio::test]
    async fn test_validate_hierarchy_operation() {
        let service = MockHierarchyManagementService;
        let location_id = Uuid::new_v4();
        
        // Valid operation
        let valid_operation = HierarchyOperation {
            operation_type: HierarchyOperationType::SetParent,
            target_location: location_id,
            parent_location: Some(Uuid::new_v4()),
            new_parent_location: None,
        };
        
        let result = service.validate_hierarchy_operation(&valid_operation).await.unwrap();
        assert!(result.is_valid);
        
        // Invalid operation (self-parenting)
        let invalid_operation = HierarchyOperation {
            operation_type: HierarchyOperationType::SetParent,
            target_location: location_id,
            parent_location: Some(location_id), // Same as target
            new_parent_location: None,
        };
        
        let result = service.validate_hierarchy_operation(&invalid_operation).await.unwrap();
        assert!(!result.is_valid);
        assert!(!result.issues.is_empty());
    }
}