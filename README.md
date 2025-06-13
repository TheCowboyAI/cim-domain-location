# CIM Domain Location

Location domain for the Composable Information Machine (CIM).

This crate provides location-related functionality including:
- Location aggregate with physical, virtual, and logical location types
- Address and geographic coordinate value objects
- Location commands and events
- Location command handlers

## Features

- **Location Types**: Physical, Virtual, Logical, and Hybrid locations
- **Address Management**: Full address validation and formatting
- **Geographic Coordinates**: Support for lat/long with distance calculations
- **Virtual Locations**: Platform-based virtual location support
- **Hierarchical Locations**: Parent-child location relationships

## Usage

```rust
use cim_domain_location::{Location, LocationType, Address, GeoCoordinates};

// Create a physical location with address
let address = Address::new(
    "123 Main St".to_string(),
    "Springfield".to_string(),
    "IL".to_string(),
    "USA".to_string(),
    "62701".to_string(),
);

let location = Location::new_physical(
    location_id,
    "Office Building".to_string(),
    address,
)?;

// Create a location with coordinates
let coords = GeoCoordinates::new(40.7128, -74.0060);
let location = Location::new_from_coordinates(
    location_id,
    "NYC Office".to_string(),
    coords,
)?;
```

## License

Licensed under either of:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
