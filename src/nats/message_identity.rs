//! Message Identity for CIM-compliant Events and Commands
//!
//! This module implements the MANDATORY correlation and causation ID system required by CIM.
//! All events, commands, and queries MUST include these identifiers for proper event tracing
//! and system coherence.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;
use cid::Cid;

/// Message identifiers required by CIM principles
/// Every message in the system MUST have correlation and causation IDs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageIdentity {
    /// Unique identifier for this specific message
    pub message_id: MessageId,
    /// Groups related messages across the entire workflow/operation (REQUIRED)
    pub correlation_id: CorrelationId,
    /// What caused this message - parent message ID (REQUIRED)
    pub causation_id: CausationId,
}

impl MessageIdentity {
    /// Create a root message identity (for starting new workflows/operations)
    /// Root messages have MessageId = CorrelationId = CausationId (self-correlation)
    pub fn new_root() -> Self {
        let id = MessageId::new();
        Self {
            message_id: id.clone(),
            correlation_id: CorrelationId::from(id.clone()),
            causation_id: CausationId::from(id),
        }
    }

    /// Create a caused message identity (for messages triggered by other messages)
    /// Caused messages inherit CorrelationId from parent, CausationId = parent.MessageId
    pub fn new_caused_by(parent: &MessageIdentity) -> Self {
        Self {
            message_id: MessageId::new(),
            correlation_id: parent.correlation_id.clone(),
            causation_id: CausationId::from(parent.message_id.clone()),
        }
    }

    /// Validate that this message identity is properly formed
    pub fn validate(&self) -> Result<(), IdentityError> {
        // Root messages: MessageId = CorrelationId = CausationId
        if self.message_id.0 == self.correlation_id.0 && self.correlation_id.0 == self.causation_id.0 {
            return Ok(()); // Valid root message
        }

        // Caused messages: CausationId != MessageId, CorrelationId should be consistent
        if self.causation_id.0 != self.message_id.0 {
            return Ok(()); // Valid caused message
        }

        Err(IdentityError::InvalidIdentityStructure {
            message_id: self.message_id.0,
            correlation_id: self.correlation_id.0,
            causation_id: self.causation_id.0,
        })
    }

    /// Check if this is a root message
    pub fn is_root(&self) -> bool {
        self.message_id.0 == self.correlation_id.0 && self.correlation_id.0 == self.causation_id.0
    }

    /// Get the correlation chain depth (0 for root, 1+ for caused messages)
    pub fn chain_depth(&self) -> u32 {
        if self.is_root() {
            0
        } else {
            // In a real system, this would traverse the causation chain
            // For now, we indicate it's at least 1 level deep
            1
        }
    }
}

/// Unique identifier for each message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MessageId(pub Uuid);

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for MessageId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for MessageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Correlation identifier to group related messages
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CorrelationId(pub Uuid);

impl CorrelationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<MessageId> for CorrelationId {
    fn from(message_id: MessageId) -> Self {
        Self(message_id.0)
    }
}

impl Default for CorrelationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CorrelationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Causation identifier to track what caused this message
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CausationId(pub Uuid);

impl CausationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl From<MessageId> for CausationId {
    fn from(message_id: MessageId) -> Self {
        Self(message_id.0)
    }
}

impl Default for CausationId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for CausationId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Event metadata including message identity and timestamp
/// Timestamp is separate from correlation algebra as per CIM principles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Message identity (REQUIRED)
    pub identity: MessageIdentity,
    /// When the event occurred (separate from correlation algebra)
    pub timestamp: SystemTime,
    /// Who/what triggered this event (optional)
    pub actor: Option<ActorId>,
    /// Version of the event schema for evolution
    pub schema_version: String,
}

impl EventMetadata {
    /// Create metadata for a root event
    pub fn new_root(actor: Option<ActorId>) -> Self {
        Self {
            identity: MessageIdentity::new_root(),
            timestamp: SystemTime::now(),
            actor,
            schema_version: "1.0".to_string(),
        }
    }

    /// Create metadata for an event caused by another message
    pub fn new_caused_by(parent: &MessageIdentity, actor: Option<ActorId>) -> Self {
        Self {
            identity: MessageIdentity::new_caused_by(parent),
            timestamp: SystemTime::now(),
            actor,
            schema_version: "1.0".to_string(),
        }
    }

    /// Validate the event metadata
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.identity.validate()?;
        
        // Ensure timestamp is reasonable (not too far in the future)
        let now = SystemTime::now();
        if let Ok(duration) = self.timestamp.duration_since(now) {
            if duration.as_secs() > 300 { // 5 minutes tolerance
                return Err(IdentityError::FutureTimestamp {
                    timestamp: self.timestamp,
                    now,
                });
            }
        }

        Ok(())
    }
}

/// Actor identifier (who/what performed the action)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActorId {
    /// Human user
    User(Uuid),
    /// System service
    System(String),
    /// External system
    External(String),
    /// Location tracking service
    LocationTracker(String),
    /// Geocoding service
    Geocoder(String),
}

impl ActorId {
    pub fn user(user_id: Uuid) -> Self {
        Self::User(user_id)
    }

    pub fn system(service_name: impl Into<String>) -> Self {
        Self::System(service_name.into())
    }

    pub fn location_tracker(tracker_name: impl Into<String>) -> Self {
        Self::LocationTracker(tracker_name.into())
    }

    pub fn geocoder(geocoder_name: impl Into<String>) -> Self {
        Self::Geocoder(geocoder_name.into())
    }
}

impl std::fmt::Display for ActorId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User(uuid) => write!(f, "user:{}", uuid),
            Self::System(name) => write!(f, "system:{}", name),
            Self::External(name) => write!(f, "external:{}", name),
            Self::LocationTracker(name) => write!(f, "location-tracker:{}", name),
            Self::Geocoder(name) => write!(f, "geocoder:{}", name),
        }
    }
}

/// CIM-compliant domain event with mandatory correlation/causation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimDomainEvent {
    /// Event metadata (REQUIRED - includes correlation/causation)
    pub metadata: EventMetadata,
    
    /// Aggregate identifier
    pub aggregate_id: String,
    
    /// Event sequence number within aggregate
    pub sequence: u64,
    
    /// CID chain for cryptographic integrity
    pub event_cid: Option<Cid>,
    pub previous_cid: Option<Cid>,
    
    /// Event type identifier
    pub event_type: String,
    
    /// Event payload
    pub payload: serde_json::Value,
}

impl CimDomainEvent {
    /// Create a new domain event with proper identity
    pub fn new(
        aggregate_id: String,
        sequence: u64,
        event_type: String,
        payload: serde_json::Value,
        parent_identity: Option<&MessageIdentity>,
        actor: Option<ActorId>,
    ) -> Self {
        let metadata = match parent_identity {
            Some(parent) => EventMetadata::new_caused_by(parent, actor),
            None => EventMetadata::new_root(actor),
        };

        Self {
            metadata,
            aggregate_id,
            sequence,
            event_cid: None,
            previous_cid: None,
            event_type,
            payload,
        }
    }

    /// Set CID information for the event
    pub fn with_cid(mut self, event_cid: Cid, previous_cid: Option<Cid>) -> Self {
        self.event_cid = Some(event_cid);
        self.previous_cid = previous_cid;
        self
    }

    /// Validate the event structure
    pub fn validate(&self) -> Result<(), IdentityError> {
        self.metadata.validate()?;
        
        if self.aggregate_id.is_empty() {
            return Err(IdentityError::EmptyAggregateId);
        }
        
        if self.event_type.is_empty() {
            return Err(IdentityError::EmptyEventType);
        }

        Ok(())
    }

    /// Get the correlation ID for this event
    pub fn correlation_id(&self) -> &CorrelationId {
        &self.metadata.identity.correlation_id
    }

    /// Get the causation ID for this event
    pub fn causation_id(&self) -> &CausationId {
        &self.metadata.identity.causation_id
    }

    /// Get the message ID for this event
    pub fn message_id(&self) -> &MessageId {
        &self.metadata.identity.message_id
    }

    /// Check if this is a root event
    pub fn is_root_event(&self) -> bool {
        self.metadata.identity.is_root()
    }
}

/// Message factory for creating properly correlated messages
/// ALWAYS use this factory - NEVER create messages directly
pub struct MessageFactory;

impl MessageFactory {
    /// Create a root message (starts a new correlation chain)
    pub fn create_root<T>(payload: T) -> CimMessage<T> {
        CimMessage {
            metadata: EventMetadata::new_root(None),
            payload,
        }
    }

    /// Create a root message with specific actor
    pub fn create_root_with_actor<T>(payload: T, actor: ActorId) -> CimMessage<T> {
        CimMessage {
            metadata: EventMetadata::new_root(Some(actor)),
            payload,
        }
    }

    /// Create a message caused by another message
    pub fn create_caused_by<T>(payload: T, parent: &MessageIdentity) -> CimMessage<T> {
        CimMessage {
            metadata: EventMetadata::new_caused_by(parent, None),
            payload,
        }
    }

    /// Create a message caused by another message with specific actor
    pub fn create_caused_by_with_actor<T>(
        payload: T, 
        parent: &MessageIdentity, 
        actor: ActorId
    ) -> CimMessage<T> {
        CimMessage {
            metadata: EventMetadata::new_caused_by(parent, Some(actor)),
            payload,
        }
    }
}

/// Generic CIM message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CimMessage<T> {
    pub metadata: EventMetadata,
    pub payload: T,
}

impl<T> CimMessage<T> {
    /// Get the message identity
    pub fn identity(&self) -> &MessageIdentity {
        &self.metadata.identity
    }

    /// Convert to domain event
    pub fn to_domain_event(
        self,
        aggregate_id: String,
        sequence: u64,
        event_type: String,
    ) -> Result<CimDomainEvent, serde_json::Error> 
    where 
        T: Serialize 
    {
        let payload = serde_json::to_value(self.payload)?;
        
        Ok(CimDomainEvent {
            metadata: self.metadata,
            aggregate_id,
            sequence,
            event_cid: None,
            previous_cid: None,
            event_type,
            payload,
        })
    }
}

/// Errors in message identity handling
#[derive(Debug, thiserror::Error)]
pub enum IdentityError {
    #[error("Invalid identity structure: message_id={message_id}, correlation_id={correlation_id}, causation_id={causation_id}")]
    InvalidIdentityStructure {
        message_id: Uuid,
        correlation_id: Uuid,
        causation_id: Uuid,
    },

    #[error("Future timestamp not allowed: timestamp={timestamp:?}, now={now:?}")]
    FutureTimestamp {
        timestamp: SystemTime,
        now: SystemTime,
    },

    #[error("Empty aggregate ID not allowed")]
    EmptyAggregateId,

    #[error("Empty event type not allowed")]
    EmptyEventType,

    #[error("Causation cycle detected in correlation chain")]
    CausationCycle,

    #[error("Duplicate message ID in correlation chain: {0}")]
    DuplicateMessage(Uuid),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_root_message_identity() {
        let identity = MessageIdentity::new_root();
        
        // Root messages have all IDs equal
        assert_eq!(identity.message_id.0, identity.correlation_id.0);
        assert_eq!(identity.correlation_id.0, identity.causation_id.0);
        assert!(identity.is_root());
        assert_eq!(identity.chain_depth(), 0);
        assert!(identity.validate().is_ok());
    }

    #[test]
    fn test_caused_message_identity() {
        let root = MessageIdentity::new_root();
        let caused = MessageIdentity::new_caused_by(&root);
        
        // Caused messages inherit correlation ID, have different message/causation IDs
        assert_ne!(caused.message_id.0, root.message_id.0);
        assert_eq!(caused.correlation_id.0, root.correlation_id.0);
        assert_eq!(caused.causation_id.0, root.message_id.0);
        assert!(!caused.is_root());
        assert_eq!(caused.chain_depth(), 1);
        assert!(caused.validate().is_ok());
    }

    #[test]
    fn test_event_metadata_creation() {
        let metadata = EventMetadata::new_root(Some(ActorId::system("location-service")));
        
        assert!(metadata.identity.is_root());
        assert!(metadata.actor.is_some());
        assert!(metadata.validate().is_ok());
    }

    #[test]
    fn test_message_factory() {
        #[derive(Serialize, Deserialize, Clone)]
        struct TestPayload {
            data: String,
        }

        let payload = TestPayload { data: "test".to_string() };
        let root_msg = MessageFactory::create_root(payload.clone());
        
        assert!(root_msg.identity().is_root());
        assert_eq!(root_msg.payload.data, "test");

        let caused_msg = MessageFactory::create_caused_by(payload, root_msg.identity());
        
        assert!(!caused_msg.identity().is_root());
        assert_eq!(caused_msg.identity().correlation_id, root_msg.identity().correlation_id);
        assert_eq!(caused_msg.identity().causation_id.0, root_msg.identity().message_id.0);
    }

    #[test]
    fn test_domain_event_creation() {
        let event = CimDomainEvent::new(
            "location-123".to_string(),
            1,
            "LocationDefined".to_string(),
            serde_json::json!({"name": "Test Location"}),
            None,
            Some(ActorId::user(Uuid::new_v4())),
        );

        assert!(event.validate().is_ok());
        assert!(event.is_root_event());
        assert_eq!(event.aggregate_id, "location-123");
        assert_eq!(event.sequence, 1);
        assert_eq!(event.event_type, "LocationDefined");
    }

    #[test]
    fn test_location_specific_actor_ids() {
        let user_id = Uuid::new_v4();
        let user_actor = ActorId::user(user_id);
        let tracker_actor = ActorId::location_tracker("gps-tracker");
        let geocoder_actor = ActorId::geocoder("google-maps");

        assert_eq!(user_actor.to_string(), format!("user:{}", user_id));
        assert_eq!(tracker_actor.to_string(), "location-tracker:gps-tracker");
        assert_eq!(geocoder_actor.to_string(), "geocoder:google-maps");
    }

    #[test]
    fn test_correlation_chain() {
        // Create a chain of 3 messages
        let root = MessageIdentity::new_root();
        let child1 = MessageIdentity::new_caused_by(&root);
        let child2 = MessageIdentity::new_caused_by(&child1);

        // All should have the same correlation ID
        assert_eq!(root.correlation_id, child1.correlation_id);
        assert_eq!(child1.correlation_id, child2.correlation_id);

        // Causation should form a chain
        assert_eq!(child1.causation_id.0, root.message_id.0);
        assert_eq!(child2.causation_id.0, child1.message_id.0);

        // Only root is root
        assert!(root.is_root());
        assert!(!child1.is_root());
        assert!(!child2.is_root());
    }

    #[test]
    fn test_cim_message_to_domain_event() {
        #[derive(Serialize)]
        struct LocationEvent {
            name: String,
            latitude: f64,
            longitude: f64,
        }

        let payload = LocationEvent {
            name: "Test Location".to_string(),
            latitude: 37.7749,
            longitude: -122.4194,
        };

        let msg = MessageFactory::create_root_with_actor(
            payload,
            ActorId::system("location-service")
        );

        let domain_event = msg.to_domain_event(
            "location-aggregate".to_string(),
            1,
            "LocationDefined".to_string(),
        ).unwrap();

        assert_eq!(domain_event.aggregate_id, "location-aggregate");
        assert_eq!(domain_event.sequence, 1);
        assert_eq!(domain_event.event_type, "LocationDefined");
        assert_eq!(domain_event.payload["name"], "Test Location");
        assert_eq!(domain_event.payload["latitude"], 37.7749);
        assert_eq!(domain_event.payload["longitude"], -122.4194);
        assert!(domain_event.validate().is_ok());
    }
}