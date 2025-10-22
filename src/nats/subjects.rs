//! Location Domain Subject Algebra
//!
//! This module defines the complete subject algebra for the Location domain following CIM principles.
//! All NATS communication within and across domain boundaries must use these subject patterns.
//!
//! Subject Structure:
//! - `domain.location.{aggregate}.{operation}.{entity_id}`
//! - `events.location.{aggregate}.{event_type}.{entity_id}`
//! - `commands.location.{aggregate}.{command_type}.{entity_id}`
//! - `queries.location.{view}.{query_type}.{entity_id?}`
//!
//! Extended Subject Patterns:
//! - `events.location.user.{event_type}.{user_id}` - User-scoped events
//! - `events.location.user.{user_id}.{aggregate}.{event_type}.{entity_id}` - User + entity events
//! - `events.location.region.{region_id}.{aggregate}.{event_type}` - Region-scoped events
//! - `events.location.coordinates.{lat}.{lng}.{aggregate}.{event_type}` - Geographic events
//!
//! This algebra ensures:
//! - Perfect domain isolation through event boundaries
//! - Geographic coordinate-based subscriptions for spatial queries
//! - User-scoped event streams for personalized location views
//! - Region-based event aggregation for hierarchical location management
//! - Multi-level subscription granularity (global, user, location, region, coordinates)
//! - Hierarchical subject organization for efficient routing
//! - Semantic clarity for AI-driven understanding
//! - NATS wildcard support for subscription patterns

use serde::{Serialize, Deserialize};
use std::fmt;
use uuid::Uuid;

/// Root subject algebra for the Location domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationSubject {
    /// Subject namespace (domain, events, commands, queries)
    pub namespace: SubjectNamespace,
    /// Location domain identifier
    pub domain: LocationDomain,
    /// Subject scope (standard, user-scoped, region-scoped, coordinate-scoped)
    pub scope: SubjectScope,
    /// Operation or event type
    pub operation: SubjectOperation,
    /// Entity identifier (optional for broadcasts)
    pub entity_id: Option<String>,
}

/// Subject scope for different subscription patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectScope {
    /// Standard aggregate-based scope
    Aggregate(LocationAggregate),
    /// User-scoped events and operations
    User {
        user_id: String,
        aggregate: Option<LocationAggregate>,
    },
    /// Region-scoped events (hierarchical location boundaries)
    Region {
        region_id: String,
        aggregate: Option<LocationAggregate>,
    },
    /// Geographic coordinate-scoped events for spatial queries
    Coordinates {
        latitude: String,
        longitude: String,
        aggregate: Option<LocationAggregate>,
    },
    /// Combined user + location scope
    UserLocation {
        user_id: String,
        location_id: String,
    },
    /// Combined region + user scope
    RegionUser {
        region_id: String,
        user_id: String,
    },
    /// Hierarchical parent-child location scope
    Hierarchy {
        parent_id: String,
        child_id: String,
    },
}

impl LocationSubject {
    /// Create a new location subject
    pub fn new(
        namespace: SubjectNamespace,
        scope: SubjectScope,
        operation: SubjectOperation,
        entity_id: Option<String>,
    ) -> Self {
        Self {
            namespace,
            domain: LocationDomain::Location,
            scope,
            operation,
            entity_id,
        }
    }

    /// Create an event subject with standard aggregate scope
    pub fn event(
        aggregate: LocationAggregate,
        event_type: EventType,
        entity_id: String,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Event(event_type),
            Some(entity_id),
        )
    }

    /// Create a command subject with standard aggregate scope
    pub fn command(
        aggregate: LocationAggregate,
        command_type: CommandType,
        entity_id: String,
    ) -> Self {
        Self::new(
            SubjectNamespace::Commands,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Command(command_type),
            Some(entity_id),
        )
    }

    /// Create a query subject with standard aggregate scope
    pub fn query(
        aggregate: LocationAggregate,
        query_type: QueryType,
        entity_id: Option<String>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Queries,
            SubjectScope::Aggregate(aggregate),
            SubjectOperation::Query(query_type),
            entity_id,
        )
    }

    /// Create a user-scoped event subject
    pub fn user_event(
        user_id: &Uuid,
        event_type: EventType,
        aggregate: Option<LocationAggregate>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::User {
                user_id: user_id.to_string(),
                aggregate,
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a region-scoped event subject
    pub fn region_event(
        region_id: &Uuid,
        event_type: EventType,
        aggregate: Option<LocationAggregate>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Region {
                region_id: region_id.to_string(),
                aggregate,
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a coordinate-scoped event subject for spatial events
    pub fn coordinate_event(
        latitude: f64,
        longitude: f64,
        event_type: EventType,
        aggregate: Option<LocationAggregate>,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Coordinates {
                latitude: format!("{:.6}", latitude),
                longitude: format!("{:.6}", longitude),
                aggregate,
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a user + location scoped event subject
    pub fn user_location_event(
        user_id: &Uuid,
        location_id: &Uuid,
        event_type: EventType,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::UserLocation {
                user_id: user_id.to_string(),
                location_id: location_id.to_string(),
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Create a hierarchy-scoped event subject for parent-child relationships
    pub fn hierarchy_event(
        parent_id: &Uuid,
        child_id: &Uuid,
        event_type: EventType,
    ) -> Self {
        Self::new(
            SubjectNamespace::Events,
            SubjectScope::Hierarchy {
                parent_id: parent_id.to_string(),
                child_id: child_id.to_string(),
            },
            SubjectOperation::Event(event_type),
            None,
        )
    }

    /// Get subject for wildcard subscription
    pub fn wildcard_pattern(&self) -> String {
        let base_pattern = self.build_base_subject();
        match &self.entity_id {
            Some(_) => format!("{}.>", base_pattern),
            None => format!("{}.*", base_pattern),
        }
    }

    /// Convert to NATS subject string
    pub fn to_subject(&self) -> String {
        let base_subject = self.build_base_subject();
        match &self.entity_id {
            Some(id) => format!("{}.{}", base_subject, id),
            None => base_subject,
        }
    }

    /// Build the base subject without entity ID
    fn build_base_subject(&self) -> String {
        let namespace = self.namespace.as_str();
        let domain = self.domain.as_str();
        let operation = self.operation.as_str();

        match &self.scope {
            SubjectScope::Aggregate(aggregate) => {
                format!("{}.{}.{}.{}", namespace, domain, aggregate.as_str(), operation)
            }
            SubjectScope::User { user_id, aggregate } => {
                match aggregate {
                    Some(agg) => format!("{}.{}.user.{}.{}.{}", namespace, domain, user_id, agg.as_str(), operation),
                    None => format!("{}.{}.user.{}.{}", namespace, domain, user_id, operation),
                }
            }
            SubjectScope::Region { region_id, aggregate } => {
                match aggregate {
                    Some(agg) => format!("{}.{}.region.{}.{}.{}", namespace, domain, region_id, agg.as_str(), operation),
                    None => format!("{}.{}.region.{}.{}", namespace, domain, region_id, operation),
                }
            }
            SubjectScope::Coordinates { latitude, longitude, aggregate } => {
                match aggregate {
                    Some(agg) => format!("{}.{}.coordinates.{}.{}.{}.{}", namespace, domain, latitude, longitude, agg.as_str(), operation),
                    None => format!("{}.{}.coordinates.{}.{}.{}", namespace, domain, latitude, longitude, operation),
                }
            }
            SubjectScope::UserLocation { user_id, location_id } => {
                format!("{}.{}.user.{}.location.{}.{}", namespace, domain, user_id, location_id, operation)
            }
            SubjectScope::RegionUser { region_id, user_id } => {
                format!("{}.{}.region.{}.user.{}.{}", namespace, domain, region_id, user_id, operation)
            }
            SubjectScope::Hierarchy { parent_id, child_id } => {
                format!("{}.{}.hierarchy.{}.child.{}.{}", namespace, domain, parent_id, child_id, operation)
            }
        }
    }
}

impl fmt::Display for LocationSubject {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_subject())
    }
}

/// Subject namespaces for different message types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectNamespace {
    /// Domain-internal operations
    Domain,
    /// Event publications (past tense)
    Events,
    /// Command requests (imperative)
    Commands,
    /// Query requests (interrogative)
    Queries,
    /// Cross-domain integration
    Integration,
}

impl SubjectNamespace {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Domain => "domain",
            Self::Events => "events",
            Self::Commands => "commands",
            Self::Queries => "queries",
            Self::Integration => "integration",
        }
    }
}

/// Location domain identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationDomain {
    Location,
}

impl LocationDomain {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Location => "location",
        }
    }
}

/// Aggregates within the Location domain
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LocationAggregate {
    /// Core location aggregate
    Location,
    /// Physical addresses
    Address,
    /// Geographic coordinates
    Coordinates,
    /// Virtual locations (URLs, platforms)
    Virtual,
    /// Location hierarchies and relationships
    Hierarchy,
    /// Location metadata and attributes
    Metadata,
    /// Location-based regions and boundaries
    Region,
    /// Location access and permissions
    Access,
    /// Location history and tracking
    History,
    /// Location search and indexing
    Search,
}

impl LocationAggregate {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Location => "location",
            Self::Address => "address",
            Self::Coordinates => "coordinates",
            Self::Virtual => "virtual",
            Self::Hierarchy => "hierarchy",
            Self::Metadata => "metadata",
            Self::Region => "region",
            Self::Access => "access",
            Self::History => "history",
            Self::Search => "search",
        }
    }
}

/// Operations within subject algebra
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SubjectOperation {
    /// Event notification
    Event(EventType),
    /// Command execution
    Command(CommandType),
    /// Query execution
    Query(QueryType),
}

impl SubjectOperation {
    pub fn as_str(&self) -> String {
        match self {
            Self::Event(event_type) => event_type.as_str().to_string(),
            Self::Command(command_type) => command_type.as_str().to_string(),
            Self::Query(query_type) => query_type.as_str().to_string(),
        }
    }
}

/// Event types in the Location domain (past tense - things that happened)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    // Core location events
    Defined,
    Updated,
    Archived,
    Restored,
    Deleted,
    
    // Address events
    AddressUpdated,
    AddressValidated,
    AddressGeocoded,
    
    // Coordinate events
    CoordinatesUpdated,
    CoordinatesValidated,
    LocationMoved,
    
    // Hierarchy events
    ParentSet,
    ParentRemoved,
    ChildAdded,
    ChildRemoved,
    HierarchyReorganized,
    
    // Metadata events
    MetadataAdded,
    MetadataUpdated,
    MetadataRemoved,
    Tagged,
    Categorized,
    
    // Virtual location events
    VirtualLocationCreated,
    VirtualLocationUpdated,
    PlatformChanged,
    UrlUpdated,
    
    // Region events
    RegionCreated,
    RegionUpdated,
    BoundaryChanged,
    RegionMerged,
    RegionSplit,
    
    // Access events
    AccessGranted,
    AccessRevoked,
    PermissionChanged,
    Shared,
    
    // History events
    VisitRecorded,
    CheckedIn,
    CheckedOut,
    TrackingStarted,
    TrackingStopped,
    
    // Search events
    Indexed,
    SearchPerformed,
    NearbySearched,
    
    // Verification events
    Verified,
    VerificationFailed,
    
    // Integration events
    ExternalSystemLinked,
    ExternalSystemUnlinked,
    DataSynchronized,
}

impl EventType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Defined => "defined",
            Self::Updated => "updated",
            Self::Archived => "archived",
            Self::Restored => "restored",
            Self::Deleted => "deleted",
            Self::AddressUpdated => "address_updated",
            Self::AddressValidated => "address_validated",
            Self::AddressGeocoded => "address_geocoded",
            Self::CoordinatesUpdated => "coordinates_updated",
            Self::CoordinatesValidated => "coordinates_validated",
            Self::LocationMoved => "location_moved",
            Self::ParentSet => "parent_set",
            Self::ParentRemoved => "parent_removed",
            Self::ChildAdded => "child_added",
            Self::ChildRemoved => "child_removed",
            Self::HierarchyReorganized => "hierarchy_reorganized",
            Self::MetadataAdded => "metadata_added",
            Self::MetadataUpdated => "metadata_updated",
            Self::MetadataRemoved => "metadata_removed",
            Self::Tagged => "tagged",
            Self::Categorized => "categorized",
            Self::VirtualLocationCreated => "virtual_location_created",
            Self::VirtualLocationUpdated => "virtual_location_updated",
            Self::PlatformChanged => "platform_changed",
            Self::UrlUpdated => "url_updated",
            Self::RegionCreated => "region_created",
            Self::RegionUpdated => "region_updated",
            Self::BoundaryChanged => "boundary_changed",
            Self::RegionMerged => "region_merged",
            Self::RegionSplit => "region_split",
            Self::AccessGranted => "access_granted",
            Self::AccessRevoked => "access_revoked",
            Self::PermissionChanged => "permission_changed",
            Self::Shared => "shared",
            Self::VisitRecorded => "visit_recorded",
            Self::CheckedIn => "checked_in",
            Self::CheckedOut => "checked_out",
            Self::TrackingStarted => "tracking_started",
            Self::TrackingStopped => "tracking_stopped",
            Self::Indexed => "indexed",
            Self::SearchPerformed => "search_performed",
            Self::NearbySearched => "nearby_searched",
            Self::Verified => "verified",
            Self::VerificationFailed => "verification_failed",
            Self::ExternalSystemLinked => "external_system_linked",
            Self::ExternalSystemUnlinked => "external_system_unlinked",
            Self::DataSynchronized => "data_synchronized",
        }
    }
}

/// Command types in the Location domain (imperative - things to do)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CommandType {
    // Core location commands
    Define,
    Update,
    Archive,
    Restore,
    Delete,
    
    // Address commands
    UpdateAddress,
    ValidateAddress,
    GeocodeAddress,
    
    // Coordinate commands
    UpdateCoordinates,
    ValidateCoordinates,
    MoveLocation,
    
    // Hierarchy commands
    SetParent,
    RemoveParent,
    AddChild,
    RemoveChild,
    ReorganizeHierarchy,
    
    // Metadata commands
    AddMetadata,
    UpdateMetadata,
    RemoveMetadata,
    Tag,
    Categorize,
    
    // Virtual location commands
    CreateVirtualLocation,
    UpdateVirtualLocation,
    ChangePlatform,
    UpdateUrl,
    
    // Region commands
    CreateRegion,
    UpdateRegion,
    ChangeBoundary,
    MergeRegion,
    SplitRegion,
    
    // Access commands
    GrantAccess,
    RevokeAccess,
    ChangePermission,
    Share,
    
    // History commands
    RecordVisit,
    CheckIn,
    CheckOut,
    StartTracking,
    StopTracking,
    
    // Search commands
    Index,
    Search,
    SearchNearby,
    
    // Verification commands
    Verify,
    
    // Integration commands
    LinkExternalSystem,
    UnlinkExternalSystem,
    SynchronizeData,
}

impl CommandType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Define => "define",
            Self::Update => "update",
            Self::Archive => "archive",
            Self::Restore => "restore",
            Self::Delete => "delete",
            Self::UpdateAddress => "update_address",
            Self::ValidateAddress => "validate_address",
            Self::GeocodeAddress => "geocode_address",
            Self::UpdateCoordinates => "update_coordinates",
            Self::ValidateCoordinates => "validate_coordinates",
            Self::MoveLocation => "move_location",
            Self::SetParent => "set_parent",
            Self::RemoveParent => "remove_parent",
            Self::AddChild => "add_child",
            Self::RemoveChild => "remove_child",
            Self::ReorganizeHierarchy => "reorganize_hierarchy",
            Self::AddMetadata => "add_metadata",
            Self::UpdateMetadata => "update_metadata",
            Self::RemoveMetadata => "remove_metadata",
            Self::Tag => "tag",
            Self::Categorize => "categorize",
            Self::CreateVirtualLocation => "create_virtual_location",
            Self::UpdateVirtualLocation => "update_virtual_location",
            Self::ChangePlatform => "change_platform",
            Self::UpdateUrl => "update_url",
            Self::CreateRegion => "create_region",
            Self::UpdateRegion => "update_region",
            Self::ChangeBoundary => "change_boundary",
            Self::MergeRegion => "merge_region",
            Self::SplitRegion => "split_region",
            Self::GrantAccess => "grant_access",
            Self::RevokeAccess => "revoke_access",
            Self::ChangePermission => "change_permission",
            Self::Share => "share",
            Self::RecordVisit => "record_visit",
            Self::CheckIn => "check_in",
            Self::CheckOut => "check_out",
            Self::StartTracking => "start_tracking",
            Self::StopTracking => "stop_tracking",
            Self::Index => "index",
            Self::Search => "search",
            Self::SearchNearby => "search_nearby",
            Self::Verify => "verify",
            Self::LinkExternalSystem => "link_external_system",
            Self::UnlinkExternalSystem => "unlink_external_system",
            Self::SynchronizeData => "synchronize_data",
        }
    }
}

/// Query types in the Location domain (interrogative - things to ask)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QueryType {
    // Location queries
    Get,
    GetHistory,
    List,
    Search,
    
    // Geographic queries
    FindNearby,
    FindWithinRadius,
    FindInRegion,
    GetCoordinates,
    GetDistance,
    
    // Hierarchy queries
    GetParent,
    GetChildren,
    GetAncestors,
    GetDescendants,
    GetHierarchy,
    
    // Address queries
    GetAddress,
    ValidateAddress,
    GeocodeAddress,
    ReverseGeocode,
    
    // Metadata queries
    GetMetadata,
    GetTags,
    GetCategory,
    SearchByTag,
    SearchByCategory,
    
    // Virtual location queries
    GetVirtualLocation,
    GetByUrl,
    GetByPlatform,
    
    // Region queries
    GetRegion,
    GetRegions,
    GetBoundary,
    
    // Access queries
    GetPermissions,
    GetAccessList,
    CheckAccess,
    
    // History queries
    GetVisitHistory,
    GetTracking,
    GetActivity,
    
    // Statistics queries
    GetStats,
    GetUsage,
    GetPopularity,
}

impl QueryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Get => "get",
            Self::GetHistory => "get_history",
            Self::List => "list",
            Self::Search => "search",
            Self::FindNearby => "find_nearby",
            Self::FindWithinRadius => "find_within_radius",
            Self::FindInRegion => "find_in_region",
            Self::GetCoordinates => "get_coordinates",
            Self::GetDistance => "get_distance",
            Self::GetParent => "get_parent",
            Self::GetChildren => "get_children",
            Self::GetAncestors => "get_ancestors",
            Self::GetDescendants => "get_descendants",
            Self::GetHierarchy => "get_hierarchy",
            Self::GetAddress => "get_address",
            Self::ValidateAddress => "validate_address",
            Self::GeocodeAddress => "geocode_address",
            Self::ReverseGeocode => "reverse_geocode",
            Self::GetMetadata => "get_metadata",
            Self::GetTags => "get_tags",
            Self::GetCategory => "get_category",
            Self::SearchByTag => "search_by_tag",
            Self::SearchByCategory => "search_by_category",
            Self::GetVirtualLocation => "get_virtual_location",
            Self::GetByUrl => "get_by_url",
            Self::GetByPlatform => "get_by_platform",
            Self::GetRegion => "get_region",
            Self::GetRegions => "get_regions",
            Self::GetBoundary => "get_boundary",
            Self::GetPermissions => "get_permissions",
            Self::GetAccessList => "get_access_list",
            Self::CheckAccess => "check_access",
            Self::GetVisitHistory => "get_visit_history",
            Self::GetTracking => "get_tracking",
            Self::GetActivity => "get_activity",
            Self::GetStats => "get_stats",
            Self::GetUsage => "get_usage",
            Self::GetPopularity => "get_popularity",
        }
    }
}

/// Predefined subject patterns for common operations
pub struct SubjectPatterns;

impl SubjectPatterns {
    /// All location events
    pub fn all_location_events() -> String {
        "events.location.>".to_string()
    }
    
    /// All commands for a specific location
    pub fn location_commands(location_id: &Uuid) -> String {
        format!("commands.location.location.*.{}", location_id.to_string())
    }
    
    /// All events for a specific location
    pub fn location_events(location_id: &Uuid) -> String {
        format!("events.location.location.*.{}", location_id.to_string())
    }
    
    /// Geographic events within coordinate boundaries
    pub fn coordinate_events(lat: f64, lng: f64) -> String {
        format!("events.location.coordinates.{:.6}.{:.6}.>", lat, lng)
    }
    
    /// Address-related events
    pub fn address_events() -> String {
        "events.location.address.>".to_string()
    }
    
    /// Hierarchy-related events
    pub fn hierarchy_events() -> String {
        "events.location.hierarchy.>".to_string()
    }
    
    /// Virtual location events
    pub fn virtual_location_events() -> String {
        "events.location.virtual.>".to_string()
    }
    
    /// Region-related events
    pub fn region_events() -> String {
        "events.location.region.>".to_string()
    }
    
    /// Search queries
    pub fn search_queries() -> String {
        "queries.location.search.*".to_string()
    }
    
    /// Integration subjects for cross-domain communication
    pub fn integration_events() -> String {
        "integration.location.>".to_string()
    }

    // ===== USER-BASED PATTERNS =====
    
    /// All events for a specific user
    pub fn user_events(user_id: &Uuid) -> String {
        format!("events.location.user.{}.>", user_id.to_string())
    }
    
    /// Location-related events for a specific user
    pub fn user_location_events(user_id: &Uuid) -> String {
        format!("events.location.user.{}.location.*", user_id.to_string())
    }
    
    /// All events for a user + location combination
    pub fn user_location_activity(user_id: &Uuid, location_id: &Uuid) -> String {
        format!("events.location.user.{}.location.{}.>", 
                user_id.to_string(), 
                location_id.to_string())
    }

    // ===== REGION-BASED PATTERNS =====
    
    /// All events for a specific region
    pub fn region_activity(region_id: &Uuid) -> String {
        format!("events.location.region.{}.>", region_id.to_string())
    }
    
    /// Location events within a specific region
    pub fn region_location_events(region_id: &Uuid) -> String {
        format!("events.location.region.{}.location.*", region_id.to_string())
    }

    // ===== GEOGRAPHIC PATTERNS =====
    
    /// Events within a geographic bounding box (simplified)
    pub fn geographic_area_events(min_lat: f64, max_lat: f64, min_lng: f64, max_lng: f64) -> String {
        // This would need more sophisticated wildcard matching in practice
        format!("events.location.coordinates.*.*.>")
    }
    
    /// All coordinate-based events
    pub fn all_coordinate_activity() -> String {
        "events.location.coordinates.*.*.>".to_string()
    }

    // ===== HIERARCHY PATTERNS =====
    
    /// Events for parent-child relationships
    pub fn hierarchy_relationship_events(parent_id: &Uuid, child_id: &Uuid) -> String {
        format!("events.location.hierarchy.{}.child.{}.>", 
                parent_id.to_string(), 
                child_id.to_string())
    }
    
    /// All hierarchy modification events
    pub fn all_hierarchy_modifications() -> String {
        "events.location.hierarchy.>".to_string()
    }

    // ===== SPECIALIZED PATTERNS =====
    
    /// Events when locations are moved or coordinates change
    pub fn location_movement_events() -> String {
        "events.location.*.{location_moved,coordinates_updated}".to_string()
    }
    
    /// Access and permission events
    pub fn access_events() -> String {
        "events.location.access.>".to_string()
    }
    
    /// User check-in/check-out events across all locations
    pub fn checkin_events() -> String {
        "events.location.history.{checked_in,checked_out}".to_string()
    }
}

/// Subject builder for programmatic subject construction
pub struct SubjectBuilder {
    namespace: Option<SubjectNamespace>,
    scope: Option<SubjectScope>,
    operation: Option<SubjectOperation>,
    entity_id: Option<String>,
}

impl SubjectBuilder {
    pub fn new() -> Self {
        Self {
            namespace: None,
            scope: None,
            operation: None,
            entity_id: None,
        }
    }
    
    pub fn namespace(mut self, namespace: SubjectNamespace) -> Self {
        self.namespace = Some(namespace);
        self
    }
    
    pub fn aggregate(mut self, aggregate: LocationAggregate) -> Self {
        self.scope = Some(SubjectScope::Aggregate(aggregate));
        self
    }
    
    pub fn user_scope(mut self, user_id: &Uuid, aggregate: Option<LocationAggregate>) -> Self {
        self.scope = Some(SubjectScope::User {
            user_id: user_id.to_string(),
            aggregate,
        });
        self
    }
    
    pub fn region_scope(mut self, region_id: &Uuid, aggregate: Option<LocationAggregate>) -> Self {
        self.scope = Some(SubjectScope::Region {
            region_id: region_id.to_string(),
            aggregate,
        });
        self
    }
    
    pub fn coordinate_scope(mut self, latitude: f64, longitude: f64, aggregate: Option<LocationAggregate>) -> Self {
        self.scope = Some(SubjectScope::Coordinates {
            latitude: format!("{:.6}", latitude),
            longitude: format!("{:.6}", longitude),
            aggregate,
        });
        self
    }
    
    pub fn user_location_scope(mut self, user_id: &Uuid, location_id: &Uuid) -> Self {
        self.scope = Some(SubjectScope::UserLocation {
            user_id: user_id.to_string(),
            location_id: location_id.to_string(),
        });
        self
    }
    
    pub fn hierarchy_scope(mut self, parent_id: &Uuid, child_id: &Uuid) -> Self {
        self.scope = Some(SubjectScope::Hierarchy {
            parent_id: parent_id.to_string(),
            child_id: child_id.to_string(),
        });
        self
    }
    
    pub fn operation(mut self, operation: SubjectOperation) -> Self {
        self.operation = Some(operation);
        self
    }
    
    pub fn entity_id(mut self, entity_id: impl Into<String>) -> Self {
        self.entity_id = Some(entity_id.into());
        self
    }
    
    pub fn location_id(mut self, location_id: &Uuid) -> Self {
        self.entity_id = Some(location_id.to_string());
        self
    }
    
    pub fn uuid(mut self, uuid: &Uuid) -> Self {
        self.entity_id = Some(uuid.to_string());
        self
    }
    
    pub fn build(self) -> Result<LocationSubject, SubjectError> {
        let namespace = self.namespace.ok_or(SubjectError::MissingNamespace)?;
        let scope = self.scope.ok_or(SubjectError::MissingScope)?;
        let operation = self.operation.ok_or(SubjectError::MissingOperation)?;
        
        Ok(LocationSubject::new(namespace, scope, operation, self.entity_id))
    }
}

impl Default for SubjectBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Errors in subject construction
#[derive(Debug, thiserror::Error)]
pub enum SubjectError {
    #[error("Missing namespace in subject")]
    MissingNamespace,
    
    #[error("Missing scope in subject")]
    MissingScope,
    
    #[error("Missing operation in subject")]
    MissingOperation,
    
    #[error("Invalid subject format: {0}")]
    InvalidFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_subject_creation() {
        let location_id = Uuid::new_v4();
        let subject = LocationSubject::event(
            LocationAggregate::Location,
            EventType::Defined,
            location_id.to_string(),
        );
        
        assert_eq!(subject.namespace, SubjectNamespace::Events);
        assert!(matches!(subject.scope, SubjectScope::Aggregate(LocationAggregate::Location)));
        assert!(matches!(subject.operation, SubjectOperation::Event(EventType::Defined)));
        assert_eq!(subject.entity_id, Some(location_id.to_string()));
    }
    
    #[test]
    fn test_subject_to_string() {
        let location_id = Uuid::new_v4();
        let subject = LocationSubject::event(
            LocationAggregate::Location,
            EventType::Defined,
            location_id.to_string(),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.location.location.defined.{}", location_id.to_string()));
    }
    
    #[test]
    fn test_coordinate_subject() {
        let subject = LocationSubject::coordinate_event(
            37.7749,
            -122.4194,
            EventType::LocationMoved,
            Some(LocationAggregate::Coordinates),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, "events.location.coordinates.37.774900.-122.419400.coordinates.location_moved");
    }
    
    #[test]
    fn test_user_subject() {
        let user_id = Uuid::new_v4();
        let subject = LocationSubject::user_event(
            &user_id,
            EventType::CheckedIn,
            Some(LocationAggregate::History),
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.location.user.{}.history.checked_in", user_id.to_string()));
    }
    
    #[test]
    fn test_hierarchy_subject() {
        let parent_id = Uuid::new_v4();
        let child_id = Uuid::new_v4();
        let subject = LocationSubject::hierarchy_event(
            &parent_id,
            &child_id,
            EventType::ChildAdded,
        );
        
        let subject_str = subject.to_subject();
        assert_eq!(subject_str, format!("events.location.hierarchy.{}.child.{}.child_added", 
                                        parent_id.to_string(), 
                                        child_id.to_string()));
    }
    
    #[test]
    fn test_subject_builder() {
        let location_id = Uuid::new_v4();
        
        let subject = SubjectBuilder::new()
            .namespace(SubjectNamespace::Commands)
            .aggregate(LocationAggregate::Location)
            .operation(SubjectOperation::Command(CommandType::Update))
            .location_id(&location_id)
            .build()
            .unwrap();
        
        assert_eq!(subject.namespace, SubjectNamespace::Commands);
        assert!(matches!(subject.scope, SubjectScope::Aggregate(LocationAggregate::Location)));
        assert!(matches!(subject.operation, SubjectOperation::Command(CommandType::Update)));
        assert_eq!(subject.entity_id, Some(location_id.to_string()));
    }
    
    #[test]
    fn test_wildcard_patterns() {
        let subject = LocationSubject::event(
            LocationAggregate::Location,
            EventType::Defined,
            "loc123".to_string(),
        );
        
        let wildcard = subject.wildcard_pattern();
        assert_eq!(wildcard, "events.location.location.defined.>");
    }
    
    #[test]
    fn test_predefined_patterns() {
        let location_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        
        assert_eq!(SubjectPatterns::all_location_events(), "events.location.>");
        assert_eq!(SubjectPatterns::address_events(), "events.location.address.>");
        assert_eq!(
            SubjectPatterns::location_events(&location_id), 
            format!("events.location.location.*.{}", location_id.to_string())
        );
        assert_eq!(
            SubjectPatterns::user_events(&user_id),
            format!("events.location.user.{}.>", user_id.to_string())
        );
        assert_eq!(
            SubjectPatterns::coordinate_events(37.7749, -122.4194),
            "events.location.coordinates.37.774900.-122.419400.>"
        );
    }
    
    #[test]
    fn test_subject_builder_validation() {
        let result = SubjectBuilder::new()
            .aggregate(LocationAggregate::Location)
            .build();
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SubjectError::MissingNamespace));
        
        let result2 = SubjectBuilder::new()
            .namespace(SubjectNamespace::Events)
            .build();
        
        assert!(result2.is_err());
        assert!(matches!(result2.unwrap_err(), SubjectError::MissingScope));
    }
    
    #[test]
    fn test_all_event_types_have_string_representation() {
        let event_types = vec![
            EventType::Defined, EventType::Updated, EventType::Archived,
            EventType::AddressUpdated, EventType::CoordinatesUpdated,
            EventType::ParentSet, EventType::MetadataAdded,
            EventType::CheckedIn, EventType::RegionCreated,
        ];
        
        for event_type in event_types {
            let str_repr = event_type.as_str();
            assert!(!str_repr.is_empty());
            assert!(!str_repr.contains(' ')); // No spaces in subject components
            assert!(!str_repr.contains('.')); // No dots in subject components
        }
    }
    
    #[test]
    fn test_all_command_types_have_string_representation() {
        let command_types = vec![
            CommandType::Define, CommandType::Update, CommandType::Archive,
            CommandType::UpdateAddress, CommandType::UpdateCoordinates,
            CommandType::SetParent, CommandType::AddMetadata,
            CommandType::CheckIn, CommandType::CreateRegion,
        ];
        
        for command_type in command_types {
            let str_repr = command_type.as_str();
            assert!(!str_repr.is_empty());
            assert!(!str_repr.contains(' ')); // No spaces in subject components
            assert!(!str_repr.contains('.')); // No dots in subject components
        }
    }
    
    #[test]
    fn test_subject_uniqueness() {
        let location_id_1 = Uuid::new_v4();
        let location_id_2 = Uuid::new_v4();
        
        let subject_1 = LocationSubject::event(
            LocationAggregate::Location,
            EventType::Defined,
            location_id_1.to_string(),
        );
        
        let subject_2 = LocationSubject::event(
            LocationAggregate::Location,
            EventType::Defined,
            location_id_2.to_string(),
        );
        
        assert_ne!(subject_1.to_subject(), subject_2.to_subject());
    }
}