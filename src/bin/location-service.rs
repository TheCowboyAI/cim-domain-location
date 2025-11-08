//! Location Service - NATS-enabled event-sourced location management
//!
//! This service provides location domain functionality over NATS messaging,
//! using event sourcing with JetStream for durable storage.
//!
//! ## Environment Variables
//!
//! - `NATS_URL` - NATS server URL (default: nats://localhost:4222)
//! - `STREAM_NAME` - JetStream stream name (default: LOCATION_EVENTS)
//! - `LOG_LEVEL` - Logging level (default: info)
//! - `SNAPSHOT_FREQUENCY` - Events between snapshots (default: 100)
//!
//! ## NATS Subjects
//!
//! ### Commands (Request/Reply)
//! - `location.commands.define` - Define a new location
//! - `location.commands.update` - Update location details
//! - `location.commands.set_parent` - Set parent location
//! - `location.commands.remove_parent` - Remove parent location
//! - `location.commands.add_metadata` - Add metadata
//! - `location.commands.archive` - Archive location
//!
//! ### Events (Publish)
//! - `events.location.{location_id}.defined` - Location defined
//! - `events.location.{location_id}.updated` - Location updated
//! - `events.location.{location_id}.parent.set` - Parent set
//! - `events.location.{location_id}.parent.removed` - Parent removed
//! - `events.location.{location_id}.metadata.added` - Metadata added
//! - `events.location.{location_id}.archived` - Location archived
//!
//! ## Example Usage
//!
//! ```bash
//! # Start the service
//! NATS_URL=nats://localhost:4222 location-service
//!
//! # With custom configuration
//! NATS_URL=nats://nats.example.com:4222 \
//! STREAM_NAME=PROD_LOCATION_EVENTS \
//! LOG_LEVEL=debug \
//! SNAPSHOT_FREQUENCY=50 \
//! location-service
//! ```

use cim_domain_location::{
    DefineLocation, UpdateLocation, SetParentLocation, RemoveParentLocation,
    AddLocationMetadata, ArchiveLocation, LocationDomainEvent,
    NatsEventStore, LocationRepository, NatsEventPublisher,
};
use async_nats::jetstream;
use futures::StreamExt;
use std::env;
use std::sync::Arc;
use tokio::signal;
use tracing::{info, error, warn, debug};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    let log_level = env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string());
    tracing_subscriber::fmt()
        .with_max_level(match log_level.as_str() {
            "trace" => tracing::Level::TRACE,
            "debug" => tracing::Level::DEBUG,
            "info" => tracing::Level::INFO,
            "warn" => tracing::Level::WARN,
            "error" => tracing::Level::ERROR,
            _ => tracing::Level::INFO,
        })
        .init();

    info!("Starting Location Service...");

    // Load configuration from environment
    let nats_url = env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
    let stream_name = env::var("STREAM_NAME").unwrap_or_else(|_| "LOCATION_EVENTS".to_string());
    let snapshot_frequency: u64 = env::var("SNAPSHOT_FREQUENCY")
        .unwrap_or_else(|_| "100".to_string())
        .parse()
        .unwrap_or(100);

    info!("Configuration:");
    info!("  NATS URL: {}", nats_url);
    info!("  Stream Name: {}", stream_name);
    info!("  Snapshot Frequency: {}", snapshot_frequency);

    // Connect to NATS
    info!("Connecting to NATS at {}...", nats_url);
    let client = async_nats::connect(&nats_url).await?;
    info!("Connected to NATS");

    // Create JetStream context
    let jetstream = jetstream::new(client.clone());

    // Create event store
    info!("Initializing event store...");
    let event_store = Arc::new(
        NatsEventStore::new(jetstream.clone(), stream_name.clone()).await?
    );
    info!("Event store initialized");

    // Create repository
    let repository = Arc::new(
        LocationRepository::new(event_store.clone())
            .with_snapshot_frequency(snapshot_frequency)
    );

    // Create event publisher
    let event_publisher = Arc::new(
        NatsEventPublisher::new(jetstream.clone(), stream_name.clone())
    );

    info!("Location service is ready");
    info!("Listening for commands on: location.commands.>");

    // Subscribe to command subjects
    let mut define_sub = client.subscribe("location.commands.define").await?;
    let mut update_sub = client.subscribe("location.commands.update").await?;
    let mut set_parent_sub = client.subscribe("location.commands.set_parent").await?;
    let mut remove_parent_sub = client.subscribe("location.commands.remove_parent").await?;
    let mut add_metadata_sub = client.subscribe("location.commands.add_metadata").await?;
    let mut archive_sub = client.subscribe("location.commands.archive").await?;

    // Clone Arc references for task handlers
    let repo_define = repository.clone();
    let repo_update = repository.clone();
    let repo_set_parent = repository.clone();
    let repo_remove_parent = repository.clone();
    let repo_add_metadata = repository.clone();
    let repo_archive = repository.clone();

    let pub_define = event_publisher.clone();
    let pub_update = event_publisher.clone();
    let pub_set_parent = event_publisher.clone();
    let pub_remove_parent = event_publisher.clone();
    let pub_add_metadata = event_publisher.clone();
    let pub_archive = event_publisher.clone();

    let client_define = client.clone();
    let client_update = client.clone();
    let client_set_parent = client.clone();
    let client_remove_parent = client.clone();
    let client_add_metadata = client.clone();
    let client_archive = client.clone();

    // Spawn command handlers
    tokio::spawn(async move {
        while let Some(msg) = define_sub.next().await {
            handle_define_location(msg, repo_define.clone(), pub_define.clone(), client_define.clone()).await;
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = update_sub.next().await {
            handle_update_location(msg, repo_update.clone(), pub_update.clone(), client_update.clone()).await;
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = set_parent_sub.next().await {
            handle_set_parent(msg, repo_set_parent.clone(), pub_set_parent.clone(), client_set_parent.clone()).await;
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = remove_parent_sub.next().await {
            handle_remove_parent(msg, repo_remove_parent.clone(), pub_remove_parent.clone(), client_remove_parent.clone()).await;
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = add_metadata_sub.next().await {
            handle_add_metadata(msg, repo_add_metadata.clone(), pub_add_metadata.clone(), client_add_metadata.clone()).await;
        }
    });

    tokio::spawn(async move {
        while let Some(msg) = archive_sub.next().await {
            handle_archive_location(msg, repo_archive.clone(), pub_archive.clone(), client_archive.clone()).await;
        }
    });

    // Wait for shutdown signal
    match signal::ctrl_c().await {
        Ok(()) => {
            info!("Received shutdown signal");
        }
        Err(err) => {
            error!("Unable to listen for shutdown signal: {}", err);
        }
    }

    info!("Location service shutting down...");
    Ok(())
}

// Command Handlers

async fn handle_define_location(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received DefineLocation command");

    // Deserialize command
    let command: DefineLocation = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize DefineLocation: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    // TODO: Implement command handler logic
    // For now, just acknowledge
    info!("DefineLocation: {} (id: {})", command.name, command.location_id);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}

async fn handle_update_location(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received UpdateLocation command");

    let command: UpdateLocation = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize UpdateLocation: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    info!("UpdateLocation: {} - {}", command.location_id, command.reason);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}

async fn handle_set_parent(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received SetParentLocation command");

    let command: SetParentLocation = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize SetParentLocation: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    info!("SetParentLocation: {} -> {} ({})", command.location_id, command.parent_id, command.reason);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}

async fn handle_remove_parent(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received RemoveParentLocation command");

    let command: RemoveParentLocation = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize RemoveParentLocation: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    info!("RemoveParentLocation: {} ({})", command.location_id, command.reason);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}

async fn handle_add_metadata(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received AddLocationMetadata command");

    let command: AddLocationMetadata = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize AddLocationMetadata: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    info!("AddLocationMetadata: {} ({} entries) - {}",
        command.location_id, command.metadata.len(), command.reason);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}

async fn handle_archive_location(
    msg: async_nats::Message,
    repository: Arc<LocationRepository>,
    publisher: Arc<NatsEventPublisher>,
    client: async_nats::Client,
) {
    debug!("Received ArchiveLocation command");

    let command: ArchiveLocation = match serde_json::from_slice(&msg.payload) {
        Ok(cmd) => cmd,
        Err(e) => {
            error!("Failed to deserialize ArchiveLocation: {}", e);
            if let Some(reply) = msg.reply {
                let _ = client.publish(reply, format!("Error: {}", e).into()).await;
            }
            return;
        }
    };

    info!("ArchiveLocation: {} ({})", command.location_id, command.reason);

    if let Some(reply) = msg.reply {
        let response = serde_json::json!({
            "status": "accepted",
            "location_id": command.location_id.to_string(),
        });
        let _ = client.publish(reply, serde_json::to_vec(&response).unwrap().into()).await;
    }
}
