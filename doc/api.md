# Location API Documentation

## Overview

The Location domain API provides commands, queries, and events for {domain purpose}.

## Commands

### CreateLocation

Creates a new location in the system.

```rust
use cim_domain_location::commands::CreateLocation;

let command = CreateLocation {
    id: LocationId::new(),
    // ... fields
};
```

**Fields:**
- `id`: Unique identifier for the location
- `field1`: Description
- `field2`: Description

**Validation:**
- Field1 must be non-empty
- Field2 must be valid

**Events Emitted:**
- `LocationCreated`

### UpdateLocation

Updates an existing location.

```rust
use cim_domain_location::commands::UpdateLocation;

let command = UpdateLocation {
    id: entity_id,
    // ... fields to update
};
```

**Fields:**
- `id`: Identifier of the location to update
- `field1`: New value (optional)

**Events Emitted:**
- `LocationUpdated`

## Queries

### GetLocationById

Retrieves a location by its identifier.

```rust
use cim_domain_location::queries::GetLocationById;

let query = GetLocationById {
    id: entity_id,
};
```

**Returns:** `Option<LocationView>`

### List{Entities}

Lists all {entities} with optional filtering.

```rust
use cim_domain_location::queries::List{Entities};

let query = List{Entities} {
    filter: Some(Filter {
        // ... filter criteria
    }),
    pagination: Some(Pagination {
        page: 1,
        per_page: 20,
    }),
};
```

**Returns:** `Vec<LocationView>`

## Events

### LocationCreated

Emitted when a new location is created.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationCreated {
    pub id: LocationId,
    pub timestamp: SystemTime,
    // ... other fields
}
```

### LocationUpdated

Emitted when a location is updated.

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationUpdated {
    pub id: LocationId,
    pub changes: Vec<FieldChange>,
    pub timestamp: SystemTime,
}
```

## Value Objects

### LocationId

Unique identifier for {entities}.

```rust
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LocationId(Uuid);

impl LocationId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}
```

### {ValueObject}

Represents {description}.

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct {ValueObject} {
    pub field1: String,
    pub field2: i32,
}
```

## Error Handling

The domain uses the following error types:

```rust
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
    #[error("location not found: {id}")]
    NotFound { id: LocationId },
    
    #[error("Invalid {field}: {reason}")]
    ValidationError { field: String, reason: String },
    
    #[error("Operation not allowed: {reason}")]
    Forbidden { reason: String },
}
```

## Usage Examples

### Creating a New Location

```rust
use cim_domain_location::{
    commands::CreateLocation,
    handlers::handle_create_location,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let command = CreateLocation {
        id: LocationId::new(),
        name: "Example".to_string(),
        // ... other fields
    };
    
    let events = handle_create_location(command).await?;
    
    for event in events {
        println!("Event emitted: {:?}", event);
    }
    
    Ok(())
}
```

### Querying {Entities}

```rust
use cim_domain_location::{
    queries::{List{Entities}, execute_query},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let query = List{Entities} {
        filter: None,
        pagination: Some(Pagination {
            page: 1,
            per_page: 10,
        }),
    };
    
    let results = execute_query(query).await?;
    
    for item in results {
        println!("{:?}", item);
    }
    
    Ok(())
}
```

## Integration with Other Domains

This domain integrates with:

- **{Other Domain}**: Description of integration
- **{Other Domain}**: Description of integration

## Performance Considerations

- Commands are processed asynchronously
- Queries use indexed projections for fast retrieval
- Events are published to NATS for distribution

## Security Considerations

- All commands require authentication
- Authorization is enforced at the aggregate level
- Sensitive data is encrypted in events 