//! Location command handler

use crate::{Location, DefineLocation, LocationDefined, LocationType, Address, GeoCoordinates};
use cim_domain::{
    CommandHandler, CommandEnvelope, CommandAcknowledgment, CommandStatus,
    AggregateRepository, EventPublisher, EntityId,
};
use crate::LocationDomainEvent;
use std::sync::Arc;

/// Handles location-related commands
pub struct LocationCommandHandler<R: AggregateRepository<Location>> {
    repository: Arc<R>,
    event_publisher: Arc<dyn EventPublisher>,
}

impl<R: AggregateRepository<Location>> LocationCommandHandler<R> {
    /// Create a new location command handler
    pub fn new(repository: Arc<R>, event_publisher: Arc<dyn EventPublisher>) -> Self {
        Self {
            repository,
            event_publisher,
        }
    }
}

impl<R: AggregateRepository<Location>> CommandHandler<DefineLocation> for LocationCommandHandler<R> {
    fn handle(&mut self, envelope: CommandEnvelope<DefineLocation>) -> CommandAcknowledgment {
        let cmd = &envelope.command;
        let location_id = EntityId::from_uuid(cmd.location_id);

        // Check if location already exists
        match self.repository.load(location_id) {
            Ok(Some(_)) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some("Location already exists".to_string()),
            },
            Ok(None) => {
                // Create new location based on type
                let location = match &cmd.location_type {
                    LocationType::Physical => {
                        if let Some(address) = &cmd.address {
                            match Location::new_physical(location_id, cmd.name.clone(), address.clone()) {
                                Ok(mut loc) => {
                                    // Add coordinates if provided
                                    if let Some(coords) = &cmd.coordinates {
                                        if let Err(e) = loc.set_coordinates(coords.clone()) {
                                            return CommandAcknowledgment {
                                                command_id: envelope.id,
                                                correlation_id: envelope.correlation_id,
                                                status: CommandStatus::Rejected,
                                                reason: Some(format!("Invalid coordinates: {}", e)),
                                            };
                                        }
                                    }
                                    loc
                                }
                                Err(e) => {
                                    return CommandAcknowledgment {
                                        command_id: envelope.id,
                                        correlation_id: envelope.correlation_id,
                                        status: CommandStatus::Rejected,
                                        reason: Some(format!("Failed to create location: {}", e)),
                                    };
                                }
                            }
                        } else if let Some(coords) = &cmd.coordinates {
                            match Location::new_from_coordinates(location_id, cmd.name.clone(), coords.clone()) {
                                Ok(loc) => loc,
                                Err(e) => {
                                    return CommandAcknowledgment {
                                        command_id: envelope.id,
                                        correlation_id: envelope.correlation_id,
                                        status: CommandStatus::Rejected,
                                        reason: Some(format!("Failed to create location: {}", e)),
                                    };
                                }
                            }
                        } else {
                            return CommandAcknowledgment {
                                command_id: envelope.id,
                                correlation_id: envelope.correlation_id,
                                status: CommandStatus::Rejected,
                                reason: Some("Physical location requires either address or coordinates".to_string()),
                            };
                        }
                    }
                    LocationType::Virtual => {
                        if let Some(virtual_loc) = &cmd.virtual_location {
                            match Location::new_virtual(location_id, cmd.name.clone(), virtual_loc.clone()) {
                                Ok(loc) => loc,
                                Err(e) => {
                                    return CommandAcknowledgment {
                                        command_id: envelope.id,
                                        correlation_id: envelope.correlation_id,
                                        status: CommandStatus::Rejected,
                                        reason: Some(format!("Failed to create virtual location: {}", e)),
                                    };
                                }
                            }
                        } else {
                            return CommandAcknowledgment {
                                command_id: envelope.id,
                                correlation_id: envelope.correlation_id,
                                status: CommandStatus::Rejected,
                                reason: Some("Virtual location requires virtual location details".to_string()),
                            };
                        }
                    }
                    _ => {
                        // For Logical and Hybrid types, create a basic location
                        let mut loc = Location::new_from_coordinates(
                            location_id,
                            cmd.name.clone(),
                            GeoCoordinates::new(0.0, 0.0), // Default coordinates
                        ).unwrap();
                        loc.location_type = cmd.location_type.clone();
                        loc
                    }
                };

                // Save location
                if let Err(e) = self.repository.save(&location) {
                    return CommandAcknowledgment {
                        command_id: envelope.id,
                        correlation_id: envelope.correlation_id,
                        status: CommandStatus::Rejected,
                        reason: Some(format!("Failed to save location: {}", e)),
                    };
                }

                // Emit event
                let event = LocationDomainEvent::LocationDefined(LocationDefined {
                    location_id: cmd.location_id,
                    name: cmd.name.clone(),
                    location_type: cmd.location_type.clone(),
                    address: cmd.address.clone(),
                    coordinates: cmd.coordinates.clone(),
                    virtual_location: cmd.virtual_location.clone(),
                    parent_id: cmd.parent_id,
                });

                // For now, we'll skip event publishing since it requires DomainEventEnum
                // TODO: Implement proper event publishing for location domain
                // if let Err(e) = self.event_publisher.publish_events(vec![event], envelope.correlation_id.clone()) {
                //     return CommandAcknowledgment {
                //         command_id: envelope.id,
                //         correlation_id: envelope.correlation_id,
                //         status: CommandStatus::Rejected,
                //         reason: Some(format!("Failed to publish event: {}", e)),
                //     };
                // }

                CommandAcknowledgment {
                    command_id: envelope.id,
                    correlation_id: envelope.correlation_id,
                    status: CommandStatus::Accepted,
                    reason: None,
                }
            }
            Err(e) => CommandAcknowledgment {
                command_id: envelope.id,
                correlation_id: envelope.correlation_id,
                status: CommandStatus::Rejected,
                reason: Some(format!("Repository error: {}", e)),
            },
        }
    }
}
