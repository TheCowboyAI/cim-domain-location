# CIM Location Domain - Troubleshooting and Monitoring Guide

This guide provides comprehensive troubleshooting procedures, monitoring setup, and diagnostic tools for the CIM Location Domain.

---

## Quick Diagnosis

### System Health Check
```rust
use cim_domain_location::*;
use tokio_test;

#[tokio::test]
async fn quick_health_check() {
    // Test basic coordinate validation
    let coords = GeoCoordinates::new(37.7749, -122.4194);
    assert!(coords.validate().is_ok());
    
    // Test geocoding service connectivity
    let service = MockGeocodingService::new();
    let address = Address::new(
        "123 Test St".to_string(),
        "San Francisco".to_string(),
        "CA".to_string(),
        "US".to_string(),
        "94102".to_string(),
    );
    
    let result = service.geocode(&address).await;
    assert!(result.is_ok());
    
    // Test spatial search functionality
    let spatial_service = MockSpatialSearchService::new();
    let search_result = spatial_service.find_within_radius(
        &coords, 
        1000.0, 
        None
    ).await;
    assert!(search_result.is_ok());
}
```

### Common Symptoms and Quick Fixes

| Symptom | Likely Cause | Quick Fix |
|---------|--------------|-----------|
| Invalid coordinate errors | Lat/Lng out of bounds | Check coordinate validation |
| Geocoding timeouts | Service unavailable | Check API limits and connectivity |
| Slow spatial queries | Missing spatial index | Rebuild spatial indexes |
| Inconsistent distances | Wrong coordinate system | Verify CRS compatibility |
| Address format errors | Incomplete address data | Validate required fields |

---

## Coordinate System Issues

### Problem: Invalid Coordinate Ranges

**Symptoms:**
- `ValidationError`: Latitude/Longitude out of range
- Coordinates appearing in wrong locations
- Distance calculations returning unexpected values

**Diagnosis:**
```rust
use cim_domain_location::GeoCoordinates;

// Check coordinate validation
let coords = GeoCoordinates::new(91.0, -181.0); // Invalid
match coords.validate() {
    Ok(_) => println!("Valid coordinates"),
    Err(e) => println!("Invalid coordinates: {}", e),
}

// Valid ranges:
// Latitude: -90.0 to 90.0
// Longitude: -180.0 to 180.0
```

**Solutions:**
1. **Input Validation:**
   ```rust
   fn validate_user_coordinates(lat: f64, lng: f64) -> Result<GeoCoordinates, String> {
       if lat < -90.0 || lat > 90.0 {
           return Err(format!("Invalid latitude: {} (must be -90 to 90)", lat));
       }
       if lng < -180.0 || lng > 180.0 {
           return Err(format!("Invalid longitude: {} (must be -180 to 180)", lng));
       }
       Ok(GeoCoordinates::new(lat, lng))
   }
   ```

2. **Coordinate Normalization:**
   ```rust
   fn normalize_longitude(mut lng: f64) -> f64 {
       while lng > 180.0 { lng -= 360.0; }
       while lng < -180.0 { lng += 360.0; }
       lng
   }
   ```

### Problem: Coordinate System Mismatch

**Symptoms:**
- Locations appearing in wrong countries
- Distance calculations wildly incorrect
- Spatial queries returning no results

**Diagnosis:**
```rust
// Check coordinate system metadata
let coords = GeoCoordinates::new(37.7749, -122.4194)
    .with_coordinate_system("EPSG:4326".to_string()); // WGS84

println!("Using coordinate system: {}", coords.coordinate_system);
```

**Solutions:**
1. **Standardize on WGS84:**
   ```rust
   fn ensure_wgs84(coords: &mut GeoCoordinates) {
       if coords.coordinate_system != "WGS84" {
           // Convert to WGS84 if needed
           coords.coordinate_system = "WGS84".to_string();
           // Apply transformation if necessary
       }
   }
   ```

2. **Coordinate Transformation Pipeline:**
   ```rust
   use proj::Proj;
   
   fn transform_coordinates(
       from_crs: &str, 
       to_crs: &str, 
       coords: &GeoCoordinates
   ) -> Result<GeoCoordinates, proj::ProjError> {
       let transformer = Proj::new_known_crs(from_crs, to_crs, None)?;
       let (x, y) = transformer.convert((coords.longitude, coords.latitude))?;
       
       Ok(GeoCoordinates::new(y, x).with_coordinate_system(to_crs.to_string()))
   }
   ```

### Problem: Precision Loss

**Symptoms:**
- Coordinates truncated to fewer decimal places
- Small location errors accumulating
- Inconsistent distance calculations

**Solutions:**
1. **Use High-Precision Storage:**
   ```rust
   use rust_decimal::Decimal;
   
   #[derive(Debug, Clone)]
   pub struct HighPrecisionCoordinates {
       pub latitude: Decimal,
       pub longitude: Decimal,
       pub altitude: Option<Decimal>,
   }
   ```

2. **Precision Tracking:**
   ```rust
   #[derive(Debug, Clone)]
   pub struct CoordinateMetadata {
       pub precision_meters: f64,
       pub source: String,
       pub measurement_time: chrono::DateTime<chrono::Utc>,
   }
   ```

---

## Geocoding Problems

### Problem: Geocoding Service Failures

**Symptoms:**
- `ServiceUnavailable` errors
- Intermittent geocoding failures
- High latency geocoding requests

**Diagnosis:**
```rust
async fn diagnose_geocoding_service() {
    let service = MockGeocodingService::new();
    let start = std::time::Instant::now();
    
    let test_address = Address::new(
        "1600 Amphitheatre Parkway".to_string(),
        "Mountain View".to_string(),
        "CA".to_string(),
        "US".to_string(),
        "94043".to_string(),
    );
    
    match service.geocode(&test_address).await {
        Ok(result) => {
            println!("Geocoding successful in {:?}", start.elapsed());
            println!("Confidence: {}", result.confidence_score);
            println!("Provider: {}", result.additional_info.provider);
        }
        Err(e) => {
            println!("Geocoding failed: {}", e);
        }
    }
}
```

**Solutions:**
1. **Implement Circuit Breaker:**
   ```rust
   use tokio::time::{timeout, Duration};
   
   pub struct CircuitBreakerGeocodingService {
       inner: Box<dyn GeocodingService>,
       failure_count: std::sync::atomic::AtomicU32,
       last_failure_time: std::sync::Mutex<Option<std::time::Instant>>,
       failure_threshold: u32,
       recovery_timeout: Duration,
   }
   
   impl CircuitBreakerGeocodingService {
       async fn geocode_with_circuit_breaker(&self, address: &Address) 
           -> Result<GeocodeResult, GeocodingError> {
           
           if self.is_circuit_open() {
               return Err(GeocodingError::ServiceUnavailable(
                   "Circuit breaker open".to_string()
               ));
           }
           
           match timeout(Duration::from_secs(10), self.inner.geocode(address)).await {
               Ok(Ok(result)) => {
                   self.reset_circuit();
                   Ok(result)
               }
               Ok(Err(e)) => {
                   self.record_failure();
                   Err(e)
               }
               Err(_) => {
                   self.record_failure();
                   Err(GeocodingError::Timeout)
               }
           }
       }
       
       fn is_circuit_open(&self) -> bool {
           let failure_count = self.failure_count.load(std::sync::atomic::Ordering::Relaxed);
           if failure_count < self.failure_threshold {
               return false;
           }
           
           let last_failure = self.last_failure_time.lock().unwrap();
           match *last_failure {
               Some(time) => time.elapsed() < self.recovery_timeout,
               None => false,
           }
       }
   }
   ```

2. **Implement Retry Logic:**
   ```rust
   async fn geocode_with_retry(
       service: &dyn GeocodingService,
       address: &Address,
       max_attempts: u32,
   ) -> Result<GeocodeResult, GeocodingError> {
       let mut last_error = None;
       
       for attempt in 1..=max_attempts {
           match service.geocode(address).await {
               Ok(result) => return Ok(result),
               Err(e) => {
                   last_error = Some(e);
                   if attempt < max_attempts {
                       let delay = Duration::from_millis(100 * 2_u64.pow(attempt - 1));
                       tokio::time::sleep(delay).await;
                   }
               }
           }
       }
       
       Err(last_error.unwrap())
   }
   ```

### Problem: Poor Geocoding Quality

**Symptoms:**
- Low confidence scores
- Incorrect location matches
- Inconsistent results for similar addresses

**Diagnosis:**
```rust
async fn analyze_geocoding_quality() {
    let service = MockGeocodingService::new();
    let test_addresses = vec![
        // Well-formatted address
        Address::new("123 Main St".to_string(), "Springfield".to_string(), "IL".to_string(), "US".to_string(), "62701".to_string()),
        // Poorly formatted address
        Address::new("main st".to_string(), "springfield".to_string(), "illinois".to_string(), "usa".to_string(), "62701".to_string()),
        // Ambiguous address
        Address::new("Main St".to_string(), "Springfield".to_string(), "".to_string(), "US".to_string(), "".to_string()),
    ];
    
    for (i, address) in test_addresses.iter().enumerate() {
        match service.geocode(address).await {
            Ok(result) => {
                println!("Address {}: Confidence {:.2}, Precision {:?}", 
                    i + 1, result.confidence_score, result.precision_level);
            }
            Err(e) => println!("Address {}: Failed - {}", i + 1, e),
        }
    }
}
```

**Solutions:**
1. **Address Preprocessing:**
   ```rust
   fn preprocess_address(address: &mut Address) {
       // Normalize case
       address.street1 = title_case(&address.street1);
       address.locality = title_case(&address.locality);
       address.region = address.region.to_uppercase();
       address.country = address.country.to_uppercase();
       
       // Standardize abbreviations
       address.street1 = standardize_street_types(&address.street1);
       address.region = standardize_region_codes(&address.region, &address.country);
   }
   
   fn title_case(s: &str) -> String {
       s.split_whitespace()
           .map(|word| {
               let mut chars = word.chars();
               match chars.next() {
                   None => String::new(),
                   Some(first) => first.to_uppercase().collect::<String>() + 
                       &chars.as_str().to_lowercase(),
               }
           })
           .collect::<Vec<_>>()
           .join(" ")
   }
   ```

2. **Quality Scoring:**
   ```rust
   fn calculate_address_quality_score(address: &Address) -> f64 {
       let mut score = 0.0;
       
       // Check completeness
       if !address.street1.trim().is_empty() { score += 0.3; }
       if !address.locality.trim().is_empty() { score += 0.2; }
       if !address.region.trim().is_empty() { score += 0.2; }
       if !address.country.trim().is_empty() { score += 0.2; }
       if !address.postal_code.trim().is_empty() { score += 0.1; }
       
       // Check format quality
       if has_numbers(&address.street1) { score += 0.1; }
       if is_valid_postal_code(&address.postal_code, &address.country) { score += 0.1; }
       
       score.min(1.0)
   }
   ```

### Problem: Rate Limiting Issues

**Symptoms:**
- `RateLimitExceeded` errors
- Degraded geocoding performance
- Quota exceeded errors

**Solutions:**
1. **Implement Rate Limiter:**
   ```rust
   use governor::{Quota, RateLimiter, state::{InMemoryState, NotKeyed}};
   use std::num::NonZeroU32;
   
   pub struct RateLimitedGeocodingService {
       inner: Box<dyn GeocodingService>,
       rate_limiter: RateLimiter<NotKeyed, InMemoryState, clock::DefaultClock>,
   }
   
   impl RateLimitedGeocodingService {
       pub fn new(service: Box<dyn GeocodingService>, requests_per_second: u32) -> Self {
           let quota = Quota::per_second(NonZeroU32::new(requests_per_second).unwrap());
           let rate_limiter = RateLimiter::direct(quota);
           
           Self { inner: service, rate_limiter }
       }
       
       pub async fn geocode(&self, address: &Address) -> Result<GeocodeResult, GeocodingError> {
           // Wait until we can make a request
           self.rate_limiter.until_ready().await;
           
           self.inner.geocode(address).await
       }
   }
   ```

2. **Batch Processing:**
   ```rust
   pub async fn batch_geocode_with_batching(
       service: &dyn GeocodingService,
       addresses: Vec<Address>,
       batch_size: usize,
       delay_between_batches: Duration,
   ) -> Result<Vec<GeocodeResult>, GeocodingError> {
       let mut results = Vec::new();
       
       for chunk in addresses.chunks(batch_size) {
           let batch_results = service.batch_geocode(chunk).await?;
           results.extend(batch_results);
           
           if results.len() < addresses.len() {
               tokio::time::sleep(delay_between_batches).await;
           }
       }
       
       Ok(results)
   }
   ```

---

## Spatial Indexing Issues

### Problem: Slow Spatial Queries

**Symptoms:**
- High latency for proximity searches
- Timeouts on bounding box queries
- Poor performance with large datasets

**Diagnosis:**
```rust
use std::time::Instant;

async fn diagnose_spatial_performance() {
    let service = MockSpatialSearchService::new();
    let center = GeoCoordinates::new(37.7749, -122.4194);
    
    // Test different query types
    let queries = vec![
        ("Small radius", 100.0),
        ("Medium radius", 1000.0),
        ("Large radius", 10000.0),
    ];
    
    for (name, radius) in queries {
        let start = Instant::now();
        match service.find_within_radius(&center, radius, None).await {
            Ok(result) => {
                println!("{}: {} results in {:?}", 
                    name, result.locations.len(), start.elapsed());
                println!("  Search time: {}ms", result.search_time_ms);
                println!("  Locations scanned: {}", 
                    result.search_metadata.performance_metrics.locations_scanned);
            }
            Err(e) => println!("{}: Error - {}", name, e),
        }
    }
}
```

**Solutions:**
1. **Spatial Index Optimization:**
   ```rust
   // R-tree index implementation
   use rstar::{RTree, AABB};
   
   #[derive(Debug, Clone)]
   struct LocationPoint {
       id: uuid::Uuid,
       coordinates: GeoCoordinates,
       metadata: LocationMetadata,
   }
   
   impl rstar::RTreeObject for LocationPoint {
       type Envelope = AABB<[f64; 2]>;
       
       fn envelope(&self) -> Self::Envelope {
           AABB::from_point([self.coordinates.longitude, self.coordinates.latitude])
       }
   }
   
   pub struct SpatialIndex {
       rtree: RTree<LocationPoint>,
   }
   
   impl SpatialIndex {
       pub fn find_within_radius(&self, center: &GeoCoordinates, radius_meters: f64) 
           -> Vec<LocationPoint> {
           
           // Convert radius to degrees (approximate)
           let radius_degrees = radius_meters / 111_320.0; // meters per degree at equator
           
           let search_envelope = AABB::from_corners(
               [center.longitude - radius_degrees, center.latitude - radius_degrees],
               [center.longitude + radius_degrees, center.latitude + radius_degrees],
           );
           
           self.rtree
               .locate_in_envelope_intersecting(&search_envelope)
               .filter(|point| point.coordinates.distance_to(center) <= radius_meters)
               .cloned()
               .collect()
       }
   }
   ```

2. **Query Optimization:**
   ```rust
   pub struct OptimizedSpatialQuery {
       pub use_index: bool,
       pub max_results: Option<usize>,
       pub early_termination: bool,
       pub cache_results: bool,
   }
   
   impl OptimizedSpatialQuery {
       pub async fn execute_within_radius(
           &self,
           service: &dyn SpatialSearchService,
           center: &GeoCoordinates,
           radius_meters: f64,
           filters: Option<SpatialSearchFilters>,
       ) -> Result<SpatialSearchResult, SpatialSearchError> {
           
           // Use progressive search for large areas
           if radius_meters > 10000.0 {
               return self.progressive_search(service, center, radius_meters, filters).await;
           }
           
           service.find_within_radius(center, radius_meters, filters).await
       }
       
       async fn progressive_search(
           &self,
           service: &dyn SpatialSearchService,
           center: &GeoCoordinates,
           target_radius: f64,
           filters: Option<SpatialSearchFilters>,
       ) -> Result<SpatialSearchResult, SpatialSearchError> {
           
           let mut current_radius = 1000.0; // Start with 1km
           let mut all_results = Vec::new();
           
           while current_radius <= target_radius {
               let result = service.find_within_radius(center, current_radius, filters.clone()).await?;
               
               // Filter out duplicates from previous searches
               let new_results: Vec<_> = result.locations.into_iter()
                   .filter(|loc| {
                       if let Some(distance) = loc.distance_meters {
                           distance > current_radius - 1000.0
                       } else {
                           true
                       }
                   })
                   .collect();
               
               all_results.extend(new_results);
               
               if self.early_termination {
                   if let Some(max) = self.max_results {
                       if all_results.len() >= max {
                           break;
                       }
                   }
               }
               
               current_radius += 1000.0; // Expand by 1km each iteration
           }
           
           // Create consolidated result
           Ok(SpatialSearchResult {
               request_id: uuid::Uuid::new_v4(),
               query: result.query,
               locations: all_results,
               total_count: all_results.len() as u64,
               search_time_ms: 0, // TODO: track actual time
               has_more_results: false,
               next_page_token: None,
               search_metadata: result.search_metadata,
           })
       }
   }
   ```

### Problem: Index Corruption

**Symptoms:**
- Inconsistent query results
- Missing locations in search results
- Index rebuild failures

**Solutions:**
1. **Index Integrity Validation:**
   ```rust
   pub struct IndexValidator {
       expected_locations: HashSet<uuid::Uuid>,
       spatial_index: SpatialIndex,
   }
   
   impl IndexValidator {
       pub fn validate_completeness(&self) -> Result<(), String> {
           let indexed_ids: HashSet<_> = self.spatial_index
               .get_all_locations()
               .map(|loc| loc.id)
               .collect();
           
           let missing: Vec<_> = self.expected_locations
               .difference(&indexed_ids)
               .collect();
           
           if !missing.is_empty() {
               return Err(format!("Missing {} locations from index", missing.len()));
           }
           
           let extra: Vec<_> = indexed_ids
               .difference(&self.expected_locations)
               .collect();
           
           if !extra.is_empty() {
               return Err(format!("Found {} extra locations in index", extra.len()));
           }
           
           Ok(())
       }
       
       pub fn validate_spatial_accuracy(&self) -> Result<(), String> {
           let sample_size = 100;
           let locations: Vec<_> = self.spatial_index
               .get_all_locations()
               .take(sample_size)
               .collect();
           
           for location in locations {
               // Test that the location can be found in a small radius around itself
               let nearby = self.spatial_index.find_within_radius(&location.coordinates, 1.0);
               
               if !nearby.iter().any(|loc| loc.id == location.id) {
                   return Err(format!(
                       "Location {} not found in spatial search around its own coordinates", 
                       location.id
                   ));
               }
           }
           
           Ok(())
       }
   }
   ```

2. **Incremental Index Updates:**
   ```rust
   pub struct IncrementalSpatialIndex {
       rtree: RTree<LocationPoint>,
       pending_updates: Vec<IndexUpdate>,
       last_full_rebuild: Instant,
       rebuild_threshold: Duration,
   }
   
   #[derive(Debug)]
   enum IndexUpdate {
       Insert(LocationPoint),
       Update { old: LocationPoint, new: LocationPoint },
       Delete(uuid::Uuid),
   }
   
   impl IncrementalSpatialIndex {
       pub fn add_location(&mut self, location: LocationPoint) {
           self.pending_updates.push(IndexUpdate::Insert(location.clone()));
           
           // Apply immediately for small updates
           if self.pending_updates.len() < 100 {
               self.apply_pending_updates();
           }
       }
       
       pub fn should_rebuild(&self) -> bool {
           self.last_full_rebuild.elapsed() > self.rebuild_threshold ||
           self.pending_updates.len() > 1000
       }
       
       pub fn rebuild_index(&mut self, all_locations: Vec<LocationPoint>) {
           self.rtree = RTree::bulk_load(all_locations);
           self.pending_updates.clear();
           self.last_full_rebuild = Instant::now();
       }
       
       fn apply_pending_updates(&mut self) {
           for update in self.pending_updates.drain(..) {
               match update {
                   IndexUpdate::Insert(location) => {
                       // Remove if exists, then insert
                       self.rtree = self.rtree.clone(); // TODO: Implement proper removal
                   }
                   IndexUpdate::Update { old, new } => {
                       // Remove old, insert new
                       // TODO: Implement proper update
                   }
                   IndexUpdate::Delete(id) => {
                       // Remove by ID
                       // TODO: Implement proper deletion
                   }
               }
           }
       }
   }
   ```

---

## Location Validation Errors

### Problem: Address Validation Failures

**Symptoms:**
- Addresses rejected despite appearing valid
- Inconsistent validation between similar addresses
- High false positive rates in validation

**Diagnosis:**
```rust
async fn diagnose_address_validation() {
    let service = MockGeocodingService::new();
    
    let test_cases = vec![
        ("Valid complete address", Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(), 
            "IL".to_string(),
            "US".to_string(),
            "62701".to_string()
        )),
        ("Missing postal code", Address::new(
            "123 Main St".to_string(),
            "Springfield".to_string(),
            "IL".to_string(), 
            "US".to_string(),
            "".to_string()
        )),
        ("International address", Address::new(
            "10 Downing Street".to_string(),
            "London".to_string(),
            "England".to_string(),
            "UK".to_string(),
            "SW1A 2AA".to_string()
        )),
    ];
    
    for (description, address) in test_cases {
        match service.validate_address(&address).await {
            Ok(result) => {
                println!("{}: Valid={}, Confidence={:.2}", 
                    description, result.is_valid, result.confidence_score);
                for issue in result.validation_issues {
                    println!("  Issue: {} - {}", issue.field, issue.message);
                }
            }
            Err(e) => println!("{}: Validation failed - {}", description, e),
        }
    }
}
```

**Solutions:**
1. **Multi-Level Validation:**
   ```rust
   #[derive(Debug, Clone)]
   pub struct AddressValidationPipeline {
       stages: Vec<Box<dyn AddressValidationStage>>,
   }
   
   #[async_trait]
   pub trait AddressValidationStage: Send + Sync {
       async fn validate(&self, address: &Address) -> AddressValidationResult;
   }
   
   pub struct SyntaxValidationStage;
   
   #[async_trait]
   impl AddressValidationStage for SyntaxValidationStage {
       async fn validate(&self, address: &Address) -> AddressValidationResult {
           let mut issues = Vec::new();
           
           // Check required fields
           if address.street1.trim().is_empty() {
               issues.push(ValidationIssue {
                   issue_type: ValidationIssueType::MissingField,
                   field: "street1".to_string(),
                   message: "Street address is required".to_string(),
                   severity: ValidationSeverity::Critical,
               });
           }
           
           // Check postal code format
           if !is_valid_postal_format(&address.postal_code, &address.country) {
               issues.push(ValidationIssue {
                   issue_type: ValidationIssueType::InvalidFormat,
                   field: "postal_code".to_string(),
                   message: "Invalid postal code format for country".to_string(),
                   severity: ValidationSeverity::Warning,
               });
           }
           
           let is_valid = !issues.iter().any(|i| i.severity == ValidationSeverity::Critical);
           
           AddressValidationResult {
               request_id: uuid::Uuid::new_v4(),
               input_address: address.clone(),
               is_valid,
               validation_issues: issues,
               suggested_corrections: vec![],
               confidence_score: if is_valid { 0.8 } else { 0.2 },
           }
       }
   }
   ```

2. **Country-Specific Validation:**
   ```rust
   pub struct CountrySpecificValidator {
       country_rules: HashMap<String, CountryValidationRules>,
   }
   
   #[derive(Debug, Clone)]
   pub struct CountryValidationRules {
       pub postal_code_pattern: regex::Regex,
       pub required_fields: Vec<String>,
       pub region_codes: HashSet<String>,
       pub address_format: AddressFormat,
   }
   
   impl CountrySpecificValidator {
       pub fn new() -> Self {
           let mut country_rules = HashMap::new();
           
           // US validation rules
           country_rules.insert("US".to_string(), CountryValidationRules {
               postal_code_pattern: regex::Regex::new(r"^\d{5}(-\d{4})?$").unwrap(),
               required_fields: vec!["street1".to_string(), "locality".to_string(), 
                                   "region".to_string(), "postal_code".to_string()],
               region_codes: ["AL", "AK", "AZ", /* ... */].iter()
                   .map(|s| s.to_string()).collect(),
               address_format: AddressFormat::US,
           });
           
           // UK validation rules  
           country_rules.insert("UK".to_string(), CountryValidationRules {
               postal_code_pattern: regex::Regex::new(
                   r"^[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}$"
               ).unwrap(),
               required_fields: vec!["street1".to_string(), "locality".to_string(), 
                                   "postal_code".to_string()],
               region_codes: HashSet::new(),
               address_format: AddressFormat::UK,
           });
           
           Self { country_rules }
       }
       
       pub fn validate_for_country(&self, address: &Address) -> ValidationResult {
           let rules = match self.country_rules.get(&address.country) {
               Some(rules) => rules,
               None => return ValidationResult::unsupported_country(&address.country),
           };
           
           let mut issues = Vec::new();
           
           // Check postal code format
           if !rules.postal_code_pattern.is_match(&address.postal_code) {
               issues.push(ValidationIssue {
                   issue_type: ValidationIssueType::InvalidFormat,
                   field: "postal_code".to_string(),
                   message: format!("Invalid postal code format for {}", address.country),
                   severity: ValidationSeverity::Critical,
               });
           }
           
           // Check region codes for countries that have them
           if !rules.region_codes.is_empty() && 
              !rules.region_codes.contains(&address.region) {
               issues.push(ValidationIssue {
                   issue_type: ValidationIssueType::InvalidFormat,
                   field: "region".to_string(),
                   message: format!("Invalid region code '{}' for {}", 
                       address.region, address.country),
                   severity: ValidationSeverity::Warning,
               });
           }
           
           ValidationResult::from_issues(issues)
       }
   }
   ```

### Problem: Coordinate Precision Issues

**Symptoms:**
- Locations snapping to incorrect positions
- Loss of precision in coordinate storage
- Inconsistent precision between different data sources

**Solutions:**
1. **Precision-Aware Storage:**
   ```rust
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct PreciseGeoCoordinates {
       pub latitude: rust_decimal::Decimal,
       pub longitude: rust_decimal::Decimal,
       pub altitude: Option<rust_decimal::Decimal>,
       pub horizontal_accuracy: Option<f64>, // meters
       pub vertical_accuracy: Option<f64>,   // meters
       pub coordinate_system: String,
       pub precision_source: PrecisionSource,
   }
   
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub enum PrecisionSource {
       GPS { dilution_of_precision: f64 },
       Survey { accuracy_class: String },
       Interpolated { confidence: f64 },
       UserInput { estimated_accuracy: f64 },
   }
   ```

2. **Precision Validation:**
   ```rust
   impl PreciseGeoCoordinates {
       pub fn validate_precision_requirements(&self, requirements: &PrecisionRequirements) 
           -> Result<(), PrecisionError> {
           
           if let Some(required_accuracy) = requirements.max_horizontal_error {
               if let Some(actual_accuracy) = self.horizontal_accuracy {
                   if actual_accuracy > required_accuracy {
                       return Err(PrecisionError::InsufficientAccuracy {
                           required: required_accuracy,
                           actual: actual_accuracy,
                       });
                   }
               } else {
                   return Err(PrecisionError::UnknownAccuracy);
               }
           }
           
           // Check coordinate precision (decimal places)
           let lat_precision = self.latitude.scale();
           let lng_precision = self.longitude.scale();
           
           if lat_precision < requirements.min_decimal_places ||
              lng_precision < requirements.min_decimal_places {
               return Err(PrecisionError::InsufficientPrecision {
                   required_decimal_places: requirements.min_decimal_places,
                   actual_lat_precision: lat_precision,
                   actual_lng_precision: lng_precision,
               });
           }
           
           Ok(())
       }
   }
   ```

---

## Performance Problems

### Problem: High Memory Usage

**Symptoms:**
- Out of memory errors during large dataset processing
- Gradual memory leaks in location services
- Poor performance with large location collections

**Diagnosis:**
```rust
use std::alloc::{GlobalAlloc, Layout, System};
use std::sync::atomic::{AtomicUsize, Ordering};

// Memory tracking allocator
struct TrackingAllocator;

static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ret = System.alloc(layout);
        if !ret.is_null() {
            ALLOCATED.fetch_add(layout.size(), Ordering::Relaxed);
        }
        ret
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        System.dealloc(ptr, layout);
        ALLOCATED.fetch_sub(layout.size(), Ordering::Relaxed);
    }
}

#[global_allocator]
static GLOBAL: TrackingAllocator = TrackingAllocator;

fn get_current_memory_usage() -> usize {
    ALLOCATED.load(Ordering::Relaxed)
}

// Test memory usage during operations
#[tokio::test]
async fn test_memory_usage() {
    let initial_memory = get_current_memory_usage();
    
    // Create large location dataset
    let mut locations = Vec::new();
    for i in 0..100000 {
        locations.push(GeoCoordinates::new(
            37.7749 + (i as f64 / 100000.0),
            -122.4194 + (i as f64 / 100000.0),
        ));
    }
    
    let after_creation = get_current_memory_usage();
    println!("Memory after creating locations: {} bytes", 
        after_creation - initial_memory);
    
    // Test spatial operations
    let service = MockSpatialSearchService::new();
    let center = GeoCoordinates::new(37.7749, -122.4194);
    
    for radius in [100.0, 1000.0, 10000.0] {
        let before_query = get_current_memory_usage();
        let _result = service.find_within_radius(&center, radius, None).await.unwrap();
        let after_query = get_current_memory_usage();
        
        println!("Memory for radius {}: {} bytes", 
            radius, after_query - before_query);
    }
}
```

**Solutions:**
1. **Streaming Processing:**
   ```rust
   use futures::Stream;
   use std::pin::Pin;
   
   pub struct StreamingLocationProcessor {
       chunk_size: usize,
   }
   
   impl StreamingLocationProcessor {
       pub fn process_locations<'a>(
           &'a self,
           locations: impl Stream<Item = GeoCoordinates> + 'a,
       ) -> Pin<Box<dyn Stream<Item = ProcessedLocation> + 'a>> {
           
           Box::pin(async_stream::stream! {
               let mut batch = Vec::with_capacity(self.chunk_size);
               
               pin_mut!(locations);
               while let Some(location) = locations.next().await {
                   batch.push(location);
                   
                   if batch.len() >= self.chunk_size {
                       for processed in self.process_batch(&batch).await {
                           yield processed;
                       }
                       batch.clear();
                   }
               }
               
               // Process remaining items
               if !batch.is_empty() {
                   for processed in self.process_batch(&batch).await {
                       yield processed;
                   }
               }
           })
       }
       
       async fn process_batch(&self, locations: &[GeoCoordinates]) 
           -> Vec<ProcessedLocation> {
           // Process chunk without keeping all data in memory
           locations.iter()
               .map(|coord| ProcessedLocation {
                   coordinates: *coord,
                   processed_at: chrono::Utc::now(),
                   // ... other fields
               })
               .collect()
       }
   }
   ```

2. **Memory Pool Management:**
   ```rust
   use std::sync::Mutex;
   use std::collections::VecDeque;
   
   pub struct LocationObjectPool<T> {
       pool: Mutex<VecDeque<T>>,
       factory: fn() -> T,
       max_size: usize,
   }
   
   impl<T> LocationObjectPool<T> {
       pub fn new(factory: fn() -> T, max_size: usize) -> Self {
           Self {
               pool: Mutex::new(VecDeque::new()),
               factory,
               max_size,
           }
       }
       
       pub fn acquire(&self) -> PooledObject<T> {
           let mut pool = self.pool.lock().unwrap();
           let obj = pool.pop_front().unwrap_or_else(|| (self.factory)());
           
           PooledObject {
               object: Some(obj),
               pool: self,
           }
       }
       
       fn return_object(&self, obj: T) {
           let mut pool = self.pool.lock().unwrap();
           if pool.len() < self.max_size {
               pool.push_back(obj);
           }
       }
   }
   
   pub struct PooledObject<'a, T> {
       object: Option<T>,
       pool: &'a LocationObjectPool<T>,
   }
   
   impl<'a, T> Drop for PooledObject<'a, T> {
       fn drop(&mut self) {
           if let Some(obj) = self.object.take() {
               self.pool.return_object(obj);
           }
       }
   }
   
   impl<'a, T> std::ops::Deref for PooledObject<'a, T> {
       type Target = T;
       fn deref(&self) -> &Self::Target {
           self.object.as_ref().unwrap()
       }
   }
   
   impl<'a, T> std::ops::DerefMut for PooledObject<'a, T> {
       fn deref_mut(&mut self) -> &mut Self::Target {
           self.object.as_mut().unwrap()
       }
   }
   ```

### Problem: Slow Distance Calculations

**Symptoms:**
- High CPU usage during proximity searches
- Slow nearest neighbor queries
- Performance degradation with large datasets

**Solutions:**
1. **Optimized Distance Calculations:**
   ```rust
   use std::f64::consts::PI;
   
   pub struct FastDistanceCalculator {
       // Pre-computed values for common operations
       earth_radius_m: f64,
       deg_to_rad: f64,
   }
   
   impl FastDistanceCalculator {
       pub fn new() -> Self {
           Self {
               earth_radius_m: 6_371_000.0,
               deg_to_rad: PI / 180.0,
           }
       }
       
       // Fast approximate distance for sorting/filtering
       pub fn approximate_distance_squared(&self, p1: &GeoCoordinates, p2: &GeoCoordinates) -> f64 {
           let dx = (p2.longitude - p1.longitude) * self.deg_to_rad;
           let dy = (p2.latitude - p1.latitude) * self.deg_to_rad;
           
           // For small distances, this is much faster than Haversine
           // and maintains correct ordering for sorting
           let avg_lat = (p1.latitude + p2.latitude) * 0.5 * self.deg_to_rad;
           let dx_corrected = dx * avg_lat.cos();
           
           dx_corrected * dx_corrected + dy * dy
       }
       
       // Vectorized distance calculation for multiple points
       pub fn batch_distances(&self, center: &GeoCoordinates, points: &[GeoCoordinates]) 
           -> Vec<f64> {
           
           let center_lat_rad = center.latitude * self.deg_to_rad;
           let center_lng_rad = center.longitude * self.deg_to_rad;
           let sin_center_lat = center_lat_rad.sin();
           let cos_center_lat = center_lat_rad.cos();
           
           points.iter().map(|point| {
               let lat_rad = point.latitude * self.deg_to_rad;
               let lng_rad = point.longitude * self.deg_to_rad;
               let delta_lng = lng_rad - center_lng_rad;
               
               let sin_lat = lat_rad.sin();
               let cos_lat = lat_rad.cos();
               let cos_delta_lng = delta_lng.cos();
               
               let a = (sin_center_lat * sin_lat + cos_center_lat * cos_lat * cos_delta_lng).acos();
               
               self.earth_radius_m * a
           }).collect()
       }
   }
   ```

2. **Distance Caching:**
   ```rust
   use std::collections::HashMap;
   use std::hash::{Hash, Hasher};
   
   #[derive(Debug, Clone, Copy)]
   struct CoordinatePair {
       lat1: OrderedFloat<f64>,
       lng1: OrderedFloat<f64>, 
       lat2: OrderedFloat<f64>,
       lng2: OrderedFloat<f64>,
   }
   
   // Helper for hashable floats
   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
   struct OrderedFloat<T>(T);
   
   impl Hash for OrderedFloat<f64> {
       fn hash<H: Hasher>(&self, state: &mut H) {
           // Hash the bytes of the float for consistent hashing
           self.0.to_bits().hash(state);
       }
   }
   
   impl From<f64> for OrderedFloat<f64> {
       fn from(f: f64) -> Self {
           // Round to reduce cache key variations
           OrderedFloat((f * 1000000.0).round() / 1000000.0)
       }
   }
   
   pub struct DistanceCache {
       cache: HashMap<CoordinatePair, f64>,
       max_entries: usize,
       hits: u64,
       misses: u64,
   }
   
   impl DistanceCache {
       pub fn new(max_entries: usize) -> Self {
           Self {
               cache: HashMap::new(),
               max_entries,
               hits: 0,
               misses: 0,
           }
       }
       
       pub fn get_distance(&mut self, p1: &GeoCoordinates, p2: &GeoCoordinates) -> f64 {
           let key = CoordinatePair {
               lat1: p1.latitude.into(),
               lng1: p1.longitude.into(),
               lat2: p2.latitude.into(),
               lng2: p2.longitude.into(),
           };
           
           // Try both orders (distance is symmetric)
           let alt_key = CoordinatePair {
               lat1: key.lat2,
               lng1: key.lng2,
               lat2: key.lat1,
               lng2: key.lng1,
           };
           
           if let Some(&distance) = self.cache.get(&key).or_else(|| self.cache.get(&alt_key)) {
               self.hits += 1;
               return distance;
           }
           
           // Calculate and cache
           let distance = p1.distance_to(p2);
           
           if self.cache.len() < self.max_entries {
               self.cache.insert(key, distance);
           }
           
           self.misses += 1;
           distance
       }
       
       pub fn cache_hit_rate(&self) -> f64 {
           if self.hits + self.misses == 0 {
               0.0
           } else {
               self.hits as f64 / (self.hits + self.misses) as f64
           }
       }
   }
   ```

---

## Data Integrity Issues

### Problem: Inconsistent Location Data

**Symptoms:**
- Same location appearing with different coordinates
- Duplicate locations with slight variations
- Inconsistent address formatting

**Solutions:**
1. **Location Deduplication:**
   ```rust
   use std::collections::HashMap;
   
   pub struct LocationDeduplicator {
       distance_threshold: f64, // meters
       similarity_threshold: f64, // 0.0 to 1.0
   }
   
   impl LocationDeduplicator {
       pub fn deduplicate_locations(&self, locations: Vec<Location>) -> Vec<Location> {
           let mut unique_locations = Vec::new();
           let mut processed = Vec::new();
           
           for location in locations {
               let mut is_duplicate = false;
               
               for (i, existing) in unique_locations.iter().enumerate() {
                   if self.are_duplicates(&location, existing) {
                       // Merge with existing location
                       unique_locations[i] = self.merge_locations(existing, &location);
                       processed.push(location.id);
                       is_duplicate = true;
                       break;
                   }
               }
               
               if !is_duplicate {
                   unique_locations.push(location);
               }
           }
           
           unique_locations
       }
       
       fn are_duplicates(&self, loc1: &Location, loc2: &Location) -> bool {
           // Check coordinate proximity
           let distance = loc1.coordinates.distance_to(&loc2.coordinates);
           if distance > self.distance_threshold {
               return false;
           }
           
           // Check address similarity if both have addresses
           if let (Some(addr1), Some(addr2)) = (&loc1.address, &loc2.address) {
               let similarity = self.calculate_address_similarity(addr1, addr2);
               if similarity < self.similarity_threshold {
                   return false;
               }
           }
           
           true
       }
       
       fn calculate_address_similarity(&self, addr1: &Address, addr2: &Address) -> f64 {
           let street_sim = jaro_winkler_similarity(&addr1.street1, &addr2.street1);
           let locality_sim = jaro_winkler_similarity(&addr1.locality, &addr2.locality);
           let region_sim = if addr1.region == addr2.region { 1.0 } else { 0.0 };
           let country_sim = if addr1.country == addr2.country { 1.0 } else { 0.0 };
           
           (street_sim + locality_sim + region_sim + country_sim) / 4.0
       }
       
       fn merge_locations(&self, primary: &Location, secondary: &Location) -> Location {
           // Merge strategy: use most complete/accurate data
           Location {
               id: primary.id, // Keep primary ID
               coordinates: self.choose_best_coordinates(&primary.coordinates, &secondary.coordinates),
               address: self.merge_addresses(&primary.address, &secondary.address),
               name: primary.name.clone().or(secondary.name.clone()),
               description: primary.description.clone().or(secondary.description.clone()),
               tags: {
                   let mut tags = primary.tags.clone();
                   tags.extend(secondary.tags.clone());
                   tags.sort();
                   tags.dedup();
                   tags
               },
               // Keep most recent update time
               last_updated: primary.last_updated.max(secondary.last_updated),
           }
       }
   }
   ```

2. **Data Validation Pipeline:**
   ```rust
   #[async_trait]
   pub trait LocationValidator: Send + Sync {
       async fn validate(&self, location: &Location) -> ValidationResult;
   }
   
   pub struct CompositeLocationValidator {
       validators: Vec<Box<dyn LocationValidator>>,
   }
   
   impl CompositeLocationValidator {
       pub async fn validate_location(&self, location: &Location) -> AggregateValidationResult {
           let mut results = Vec::new();
           
           for validator in &self.validators {
               results.push(validator.validate(location).await);
           }
           
           AggregateValidationResult::new(results)
       }
   }
   
   pub struct CoordinateRangeValidator;
   
   #[async_trait]
   impl LocationValidator for CoordinateRangeValidator {
       async fn validate(&self, location: &Location) -> ValidationResult {
           match location.coordinates.validate() {
               Ok(_) => ValidationResult::valid(),
               Err(e) => ValidationResult::invalid(format!("Invalid coordinates: {}", e)),
           }
       }
   }
   
   pub struct AddressGeocodingValidator {
       geocoding_service: Box<dyn GeocodingService>,
       max_distance_threshold: f64, // meters
   }
   
   #[async_trait]
   impl LocationValidator for AddressGeocodingValidator {
       async fn validate(&self, location: &Location) -> ValidationResult {
           let Some(address) = &location.address else {
               return ValidationResult::warning("No address to validate against coordinates");
           };
           
           match self.geocoding_service.geocode(address).await {
               Ok(geocode_result) => {
                   let distance = location.coordinates.distance_to(&geocode_result.coordinates);
                   
                   if distance > self.max_distance_threshold {
                       ValidationResult::warning(format!(
                           "Coordinates are {}m from geocoded address location (threshold: {}m)",
                           distance, self.max_distance_threshold
                       ))
                   } else {
                       ValidationResult::valid()
                   }
               }
               Err(e) => ValidationResult::error(format!("Failed to validate address: {}", e)),
           }
       }
   }
   ```

### Problem: Location Hierarchy Corruption

**Symptoms:**
- Circular references in location hierarchies
- Orphaned child locations
- Inconsistent parent-child relationships

**Solutions:**
1. **Hierarchy Validation:**
   ```rust
   pub struct LocationHierarchyValidator {
       max_depth: usize,
   }
   
   impl LocationHierarchyValidator {
       pub fn validate_hierarchy(&self, locations: &HashMap<Uuid, Location>) -> HierarchyValidationResult {
           let mut issues = Vec::new();
           let mut visited = HashSet::new();
           let mut path = HashSet::new();
           
           // Check each location that has no parent (potential roots)
           for location in locations.values().filter(|l| l.parent_id.is_none()) {
               if let Err(cycle_issues) = self.validate_subtree(locations, location, &mut visited, &mut path, 0) {
                   issues.extend(cycle_issues);
               }
           }
           
           // Check for orphaned locations
           for location in locations.values() {
               if let Some(parent_id) = location.parent_id {
                   if !locations.contains_key(&parent_id) {
                       issues.push(HierarchyIssue::OrphanedLocation {
                           location_id: location.id,
                           missing_parent_id: parent_id,
                       });
                   }
               }
           }
           
           HierarchyValidationResult { issues }
       }
       
       fn validate_subtree(
           &self,
           locations: &HashMap<Uuid, Location>,
           location: &Location,
           visited: &mut HashSet<Uuid>,
           path: &mut HashSet<Uuid>,
           depth: usize,
       ) -> Result<(), Vec<HierarchyIssue>> {
           let mut issues = Vec::new();
           
           // Check for cycles
           if path.contains(&location.id) {
               issues.push(HierarchyIssue::CyclicReference {
                   location_id: location.id,
                   cycle_path: path.iter().copied().collect(),
               });
               return Err(issues);
           }
           
           // Check depth limits
           if depth > self.max_depth {
               issues.push(HierarchyIssue::ExcessiveDepth {
                   location_id: location.id,
                   depth,
                   max_allowed: self.max_depth,
               });
           }
           
           path.insert(location.id);
           visited.insert(location.id);
           
           // Validate children
           for child_location in locations.values().filter(|l| l.parent_id == Some(location.id)) {
               if let Err(child_issues) = self.validate_subtree(locations, child_location, visited, path, depth + 1) {
                   issues.extend(child_issues);
               }
           }
           
           path.remove(&location.id);
           
           if issues.is_empty() { Ok(()) } else { Err(issues) }
       }
   }
   
   #[derive(Debug, Clone)]
   pub enum HierarchyIssue {
       CyclicReference { location_id: Uuid, cycle_path: Vec<Uuid> },
       OrphanedLocation { location_id: Uuid, missing_parent_id: Uuid },
       ExcessiveDepth { location_id: Uuid, depth: usize, max_allowed: usize },
       InconsistentRelationship { parent_id: Uuid, child_id: Uuid },
   }
   ```

2. **Automatic Hierarchy Repair:**
   ```rust
   pub struct HierarchyRepairService {
       validator: LocationHierarchyValidator,
   }
   
   impl HierarchyRepairService {
       pub fn repair_hierarchy(&self, locations: &mut HashMap<Uuid, Location>) -> RepairResult {
           let validation_result = self.validator.validate_hierarchy(locations);
           let mut repairs_made = Vec::new();
           
           for issue in validation_result.issues {
               match issue {
                   HierarchyIssue::CyclicReference { location_id, cycle_path } => {
                       // Break cycle by removing parent reference from one location
                       if let Some(location) = locations.get_mut(&location_id) {
                           location.parent_id = None;
                           repairs_made.push(RepairAction::BrokeCycle { location_id, previous_parent: location.parent_id });
                       }
                   }
                   
                   HierarchyIssue::OrphanedLocation { location_id, missing_parent_id } => {
                       // Clear invalid parent reference
                       if let Some(location) = locations.get_mut(&location_id) {
                           location.parent_id = None;
                           repairs_made.push(RepairAction::ClearedInvalidParent { 
                               location_id, 
                               invalid_parent_id: missing_parent_id 
                           });
                       }
                   }
                   
                   HierarchyIssue::ExcessiveDepth { location_id, .. } => {
                       // Move to root level to reduce depth
                       if let Some(location) = locations.get_mut(&location_id) {
                           let old_parent = location.parent_id;
                           location.parent_id = None;
                           repairs_made.push(RepairAction::MovedToRoot { location_id, old_parent });
                       }
                   }
                   
                   HierarchyIssue::InconsistentRelationship { parent_id, child_id } => {
                       // Fix bidirectional relationship consistency
                       // Implementation depends on specific consistency rules
                   }
               }
           }
           
           RepairResult { actions: repairs_made }
       }
   }
   ```

---

## Service Integration Problems

### Problem: External Geocoding API Failures

**Symptoms:**
- Intermittent geocoding failures
- API key authentication errors
- Rate limiting from geocoding providers

**Solutions:**
1. **Multi-Provider Fallback:**
   ```rust
   pub struct FallbackGeocodingService {
       providers: Vec<GeocodingProvider>,
       current_provider_index: std::sync::atomic::AtomicUsize,
       failure_tracking: std::sync::Mutex<HashMap<String, ProviderHealth>>,
   }
   
   #[derive(Debug, Clone)]
   struct GeocodingProvider {
       name: String,
       service: Box<dyn GeocodingService>,
       priority: u32,
       health_check_interval: Duration,
   }
   
   #[derive(Debug, Clone)]
   struct ProviderHealth {
       last_success: Option<Instant>,
       consecutive_failures: u32,
       is_healthy: bool,
       average_response_time: Duration,
   }
   
   impl FallbackGeocodingService {
       async fn geocode_with_fallback(&self, address: &Address) -> Result<GeocodeResult, GeocodingError> {
           let mut last_error = None;
           
           // Try each provider in order of priority and health
           let mut providers = self.get_sorted_providers().await;
           
           for provider in providers {
               if !self.is_provider_healthy(&provider.name).await {
                   continue;
               }
               
               match provider.service.geocode(address).await {
                   Ok(result) => {
                       self.record_success(&provider.name).await;
                       return Ok(result);
                   }
                   Err(e) => {
                       self.record_failure(&provider.name, &e).await;
                       last_error = Some(e);
                       
                       // Don't try other providers for certain errors
                       if matches!(e, GeocodingError::InvalidAddress(_) | GeocodingError::NoResults) {
                           break;
                       }
                   }
               }
           }
           
           Err(last_error.unwrap_or(GeocodingError::ServiceUnavailable(
               "All providers failed".to_string()
           )))
       }
       
       async fn get_sorted_providers(&self) -> Vec<&GeocodingProvider> {
           let mut providers: Vec<_> = self.providers.iter().collect();
           
           // Sort by health and priority
           providers.sort_by(|a, b| {
               let health_a = self.is_provider_healthy(&a.name).await;
               let health_b = self.is_provider_healthy(&b.name).await;
               
               health_b.cmp(&health_a) // Healthy providers first
                   .then_with(|| a.priority.cmp(&b.priority)) // Then by priority
           });
           
           providers
       }
   }
   ```

2. **Provider Health Monitoring:**
   ```rust
   pub struct GeocodingHealthMonitor {
       providers: Vec<String>,
       health_status: Arc<Mutex<HashMap<String, ProviderHealth>>>,
       monitoring_interval: Duration,
   }
   
   impl GeocodingHealthMonitor {
       pub async fn start_monitoring(&self) {
           let mut interval = tokio::time::interval(self.monitoring_interval);
           
           loop {
               interval.tick().await;
               self.check_all_providers().await;
           }
       }
       
       async fn check_all_providers(&self) {
           let futures: Vec<_> = self.providers.iter()
               .map(|provider| self.health_check_provider(provider))
               .collect();
           
           let results = futures::future::join_all(futures).await;
           
           let mut health_status = self.health_status.lock().unwrap();
           for (provider, health) in self.providers.iter().zip(results.iter()) {
               health_status.insert(provider.clone(), health.clone());
           }
       }
       
       async fn health_check_provider(&self, provider_name: &str) -> ProviderHealth {
           let test_address = Address::new(
               "1600 Amphitheatre Parkway".to_string(),
               "Mountain View".to_string(),
               "CA".to_string(),
               "US".to_string(),
               "94043".to_string(),
           );
           
           let start_time = Instant::now();
           
           // Get provider service and test it
           match self.get_provider_service(provider_name) {
               Some(service) => {
                   match timeout(Duration::from_secs(10), service.geocode(&test_address)).await {
                       Ok(Ok(_)) => ProviderHealth {
                           last_success: Some(Instant::now()),
                           consecutive_failures: 0,
                           is_healthy: true,
                           average_response_time: start_time.elapsed(),
                       },
                       Ok(Err(_)) | Err(_) => ProviderHealth {
                           last_success: None,
                           consecutive_failures: 1,
                           is_healthy: false,
                           average_response_time: Duration::from_secs(0),
                       }
                   }
               }
               None => ProviderHealth {
                   last_success: None,
                   consecutive_failures: u32::MAX,
                   is_healthy: false,
                   average_response_time: Duration::from_secs(0),
               }
           }
       }
   }
   ```

### Problem: NATS Communication Failures

**Symptoms:**
- Lost location update events  
- Inconsistent state across services
- Event ordering issues

**Solutions:**
1. **Reliable Event Publishing:**
   ```rust
   use async_nats::{Client, jetstream::Context};
   
   pub struct ReliableLocationEventPublisher {
       js_context: Context,
       retry_policy: RetryPolicy,
       dead_letter_queue: String,
   }
   
   #[derive(Debug, Clone)]
   pub struct RetryPolicy {
       pub max_retries: u32,
       pub initial_delay: Duration,
       pub max_delay: Duration,
       pub backoff_multiplier: f64,
   }
   
   impl ReliableLocationEventPublisher {
       pub async fn publish_event(&self, event: &LocationDomainEvent) -> Result<(), PublishError> {
           let subject = self.get_subject_for_event(event);
           let payload = serde_json::to_vec(event)?;
           
           let mut attempt = 0;
           let mut delay = self.retry_policy.initial_delay;
           
           loop {
               match self.js_context.publish(&subject, payload.clone().into()).await {
                   Ok(publish_ack) => {
                       // Wait for acknowledgment
                       match publish_ack.await {
                           Ok(_) => return Ok(()),
                           Err(e) => {
                               if attempt >= self.retry_policy.max_retries {
                                   return self.send_to_dead_letter_queue(event, &e).await;
                               }
                           }
                       }
                   }
                   Err(e) => {
                       if attempt >= self.retry_policy.max_retries {
                           return self.send_to_dead_letter_queue(event, &e).await;
                       }
                   }
               }
               
               attempt += 1;
               tokio::time::sleep(delay).await;
               delay = std::cmp::min(
                   Duration::from_millis((delay.as_millis() as f64 * self.retry_policy.backoff_multiplier) as u64),
                   self.retry_policy.max_delay
               );
           }
       }
       
       async fn send_to_dead_letter_queue(&self, event: &LocationDomainEvent, error: &impl std::error::Error) 
           -> Result<(), PublishError> {
           
           let dlq_message = DeadLetterMessage {
               original_event: event.clone(),
               error_message: error.to_string(),
               failed_at: chrono::Utc::now(),
               retry_attempts: self.retry_policy.max_retries,
           };
           
           let payload = serde_json::to_vec(&dlq_message)?;
           
           // Don't retry DLQ publishing - if this fails, log and continue
           match self.js_context.publish(&self.dead_letter_queue, payload.into()).await {
               Ok(ack) => {
                   ack.await.ok(); // Best effort
                   Err(PublishError::SentToDeadLetterQueue)
               }
               Err(_) => Err(PublishError::DeadLetterQueueFailed),
           }
       }
   }
   ```

2. **Event Ordering Guarantees:**
   ```rust
   pub struct OrderedEventProcessor {
       last_processed_sequence: AtomicU64,
       out_of_order_buffer: Arc<Mutex<BTreeMap<u64, LocationDomainEvent>>>,
       max_buffer_size: usize,
   }
   
   impl OrderedEventProcessor {
       pub async fn process_event(&self, event: LocationDomainEvent, sequence: u64) -> ProcessResult {
           let expected_sequence = self.last_processed_sequence.load(Ordering::Relaxed) + 1;
           
           if sequence == expected_sequence {
               // Process immediately
               self.process_event_immediately(event).await?;
               self.last_processed_sequence.store(sequence, Ordering::Relaxed);
               
               // Check if we can process buffered events
               self.process_buffered_events().await?;
               
               Ok(ProcessResult::Processed)
           } else if sequence > expected_sequence {
               // Buffer for later processing
               let mut buffer = self.out_of_order_buffer.lock().unwrap();
               
               if buffer.len() >= self.max_buffer_size {
                   // Buffer full, drop oldest or reject
                   return Err(ProcessError::BufferFull);
               }
               
               buffer.insert(sequence, event);
               Ok(ProcessResult::Buffered)
           } else {
               // Duplicate or very old event
               Ok(ProcessResult::Ignored)
           }
       }
       
       async fn process_buffered_events(&self) -> Result<(), ProcessError> {
           let mut to_process = Vec::new();
           
           {
               let mut buffer = self.out_of_order_buffer.lock().unwrap();
               let expected = self.last_processed_sequence.load(Ordering::Relaxed) + 1;
               
               // Collect consecutive events starting from expected sequence
               let mut current_expected = expected;
               while let Some(event) = buffer.remove(&current_expected) {
                   to_process.push((current_expected, event));
                   current_expected += 1;
               }
           }
           
           // Process collected events
           for (sequence, event) in to_process {
               self.process_event_immediately(event).await?;
               self.last_processed_sequence.store(sequence, Ordering::Relaxed);
           }
           
           Ok(())
       }
   }
   ```

---

## Common Error Patterns

### Error Pattern: "Location Not Found" During Updates

**Root Cause:** Race conditions between location creation and update operations.

**Detection:**
```rust
// Monitor for this pattern in logs
async fn detect_race_condition_pattern() {
    let error_pattern = r"Location .* not found";
    let time_window = Duration::from_secs(60);
    
    // Check if location creation happened recently before the error
    // This suggests a race condition
}
```

**Solution:**
```rust
pub struct LocationOperationCoordinator {
    pending_operations: Arc<Mutex<HashMap<Uuid, PendingOperation>>>,
}

#[derive(Debug)]
enum PendingOperation {
    Create { initiated_at: Instant },
    Update { initiated_at: Instant },
    Delete { initiated_at: Instant },
}

impl LocationOperationCoordinator {
    pub async fn coordinate_update(&self, location_id: Uuid, operation: impl FnOnce() -> BoxFuture<'static, Result<(), LocationError>>) 
        -> Result<(), LocationError> {
        
        // Check if there's a pending create operation
        {
            let pending = self.pending_operations.lock().unwrap();
            if let Some(PendingOperation::Create { initiated_at }) = pending.get(&location_id) {
                if initiated_at.elapsed() < Duration::from_secs(5) {
                    // Wait for create to complete
                    drop(pending);
                    tokio::time::sleep(Duration::from_millis(100)).await;
                    return self.coordinate_update(location_id, operation).await;
                }
            }
        }
        
        // Register this operation
        {
            let mut pending = self.pending_operations.lock().unwrap();
            pending.insert(location_id, PendingOperation::Update { initiated_at: Instant::now() });
        }
        
        let result = operation().await;
        
        // Clean up
        {
            let mut pending = self.pending_operations.lock().unwrap();
            pending.remove(&location_id);
        }
        
        result
    }
}
```

### Error Pattern: Inconsistent Distance Calculations

**Root Cause:** Mixing different coordinate systems or precision levels.

**Detection:**
```rust
fn detect_distance_inconsistencies(location_pairs: &[(Location, Location)]) -> Vec<DistanceInconsistency> {
    let mut inconsistencies = Vec::new();
    
    for (loc1, loc2) in location_pairs {
        let distance1 = loc1.coordinates.distance_to(&loc2.coordinates);
        let distance2 = loc2.coordinates.distance_to(&loc1.coordinates);
        
        let difference = (distance1 - distance2).abs();
        let relative_error = difference / distance1.max(distance2);
        
        if relative_error > 0.001 { // 0.1% tolerance
            inconsistencies.push(DistanceInconsistency {
                location1: loc1.id,
                location2: loc2.id,
                distance1,
                distance2,
                difference,
                relative_error,
            });
        }
    }
    
    inconsistencies
}
```

**Solution:**
```rust
pub struct ConsistentDistanceCalculator {
    precision: u32, // decimal places
    earth_radius: f64,
}

impl ConsistentDistanceCalculator {
    pub fn calculate_distance(&self, p1: &GeoCoordinates, p2: &GeoCoordinates) -> f64 {
        // Ensure consistent precision
        let lat1 = self.round_to_precision(p1.latitude);
        let lng1 = self.round_to_precision(p1.longitude);
        let lat2 = self.round_to_precision(p2.latitude);
        let lng2 = self.round_to_precision(p2.longitude);
        
        // Use consistent algorithm (Haversine)
        let lat1_rad = lat1.to_radians();
        let lat2_rad = lat2.to_radians();
        let delta_lat = (lat2 - lat1).to_radians();
        let delta_lng = (lng2 - lng1).to_radians();
        
        let a = (delta_lat / 2.0).sin().powi(2) +
                lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
        
        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        
        // Round result to avoid floating point precision issues
        self.round_to_precision(self.earth_radius * c)
    }
    
    fn round_to_precision(&self, value: f64) -> f64 {
        let multiplier = 10_f64.powi(self.precision as i32);
        (value * multiplier).round() / multiplier
    }
}
```

---

## Debugging Tools

### Location Inspector Tool

```rust
pub struct LocationInspector {
    geocoding_service: Box<dyn GeocodingService>,
    spatial_service: Box<dyn SpatialSearchService>,
}

impl LocationInspector {
    pub async fn inspect_location(&self, location: &Location) -> LocationInspectionReport {
        let mut report = LocationInspectionReport::new(location.id);
        
        // Basic validation
        report.coordinate_validation = self.validate_coordinates(&location.coordinates);
        
        // Address consistency check
        if let Some(address) = &location.address {
            report.address_geocoding = self.check_address_consistency(address, &location.coordinates).await;
        }
        
        // Spatial context
        report.spatial_context = self.analyze_spatial_context(&location.coordinates).await;
        
        // Hierarchy validation
        report.hierarchy_status = self.validate_hierarchy_position(location).await;
        
        report
    }
    
    async fn check_address_consistency(&self, address: &Address, coordinates: &GeoCoordinates) 
        -> AddressConsistencyReport {
        
        match self.geocoding_service.geocode(address).await {
            Ok(geocode_result) => {
                let distance = coordinates.distance_to(&geocode_result.coordinates);
                
                AddressConsistencyReport {
                    geocoding_successful: true,
                    geocoded_coordinates: Some(geocode_result.coordinates),
                    distance_from_provided: distance,
                    confidence_score: geocode_result.confidence_score,
                    precision_level: geocode_result.precision_level,
                    consistency_rating: self.rate_consistency(distance, geocode_result.confidence_score),
                }
            }
            Err(e) => AddressConsistencyReport {
                geocoding_successful: false,
                geocoding_error: Some(e.to_string()),
                ..Default::default()
            }
        }
    }
    
    async fn analyze_spatial_context(&self, coordinates: &GeoCoordinates) -> SpatialContextReport {
        // Find nearby locations
        let nearby_locations = self.spatial_service
            .find_within_radius(coordinates, 1000.0, None)
            .await
            .unwrap_or_else(|_| SpatialSearchResult::empty());
        
        // Analyze density
        let density = nearby_locations.total_count as f64 / (3.14159 * 1000.0 * 1000.0 / 1_000_000.0); // per km²
        
        SpatialContextReport {
            nearby_count: nearby_locations.total_count,
            density_per_km2: density,
            nearest_location: nearby_locations.locations.first().map(|loc| (loc.location_id, loc.distance_meters.unwrap_or(0.0))),
            context_type: self.classify_spatial_context(density, nearby_locations.total_count),
        }
    }
}

#[derive(Debug)]
pub struct LocationInspectionReport {
    pub location_id: Uuid,
    pub coordinate_validation: CoordinateValidationResult,
    pub address_geocoding: Option<AddressConsistencyReport>,
    pub spatial_context: SpatialContextReport,
    pub hierarchy_status: HierarchyValidationResult,
    pub overall_health_score: f64,
}
```

### Performance Profiler

```rust
pub struct LocationPerformanceProfiler {
    metrics: Arc<Mutex<PerformanceMetrics>>,
}

#[derive(Debug, Default)]
struct PerformanceMetrics {
    geocoding_times: Vec<Duration>,
    spatial_search_times: Vec<Duration>,
    distance_calculation_times: Vec<Duration>,
    memory_usage_samples: Vec<usize>,
}

impl LocationPerformanceProfiler {
    pub async fn profile_operation<T, F, Fut>(&self, operation_name: &str, operation: F) -> T
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        let start_time = Instant::now();
        let start_memory = self.get_memory_usage();
        
        let result = operation().await;
        
        let duration = start_time.elapsed();
        let end_memory = self.get_memory_usage();
        
        self.record_performance(operation_name, duration, start_memory, end_memory);
        
        result
    }
    
    fn get_memory_usage(&self) -> usize {
        // Platform-specific memory usage detection
        // This is a simplified version
        0 // TODO: Implement actual memory tracking
    }
    
    pub fn generate_performance_report(&self) -> PerformanceReport {
        let metrics = self.metrics.lock().unwrap();
        
        PerformanceReport {
            geocoding_stats: self.calculate_stats(&metrics.geocoding_times),
            spatial_search_stats: self.calculate_stats(&metrics.spatial_search_times),
            distance_calculation_stats: self.calculate_stats(&metrics.distance_calculation_times),
            memory_usage_stats: self.calculate_memory_stats(&metrics.memory_usage_samples),
        }
    }
    
    fn calculate_stats(&self, times: &[Duration]) -> OperationStats {
        if times.is_empty() {
            return OperationStats::default();
        }
        
        let total: Duration = times.iter().sum();
        let count = times.len();
        let average = total / count as u32;
        
        let mut sorted_times = times.to_vec();
        sorted_times.sort();
        
        let p50 = sorted_times[count / 2];
        let p95 = sorted_times[(count * 95) / 100];
        let p99 = sorted_times[(count * 99) / 100];
        
        OperationStats {
            count,
            average,
            median: p50,
            p95,
            p99,
            min: *sorted_times.first().unwrap(),
            max: *sorted_times.last().unwrap(),
        }
    }
}
```

---

## Monitoring and Alerting

### Health Monitoring

```rust
use prometheus::{Counter, Histogram, Gauge, register_counter, register_histogram, register_gauge};

lazy_static::lazy_static! {
    static ref GEOCODING_REQUESTS: Counter = register_counter!(
        "location_geocoding_requests_total",
        "Total number of geocoding requests"
    ).unwrap();
    
    static ref GEOCODING_ERRORS: Counter = register_counter!(
        "location_geocoding_errors_total", 
        "Total number of geocoding errors"
    ).unwrap();
    
    static ref SPATIAL_SEARCH_DURATION: Histogram = register_histogram!(
        "location_spatial_search_duration_seconds",
        "Time spent on spatial searches"
    ).unwrap();
    
    static ref LOCATIONS_COUNT: Gauge = register_gauge!(
        "location_total_locations",
        "Total number of locations in system"
    ).unwrap();
}

pub struct LocationHealthMonitor {
    health_check_interval: Duration,
    alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_geocoding_error_rate: f64,     // 0.0 to 1.0
    pub max_spatial_search_latency: Duration,
    pub max_memory_usage_mb: usize,
    pub min_spatial_index_efficiency: f64, // 0.0 to 1.0
}

impl LocationHealthMonitor {
    pub async fn start_monitoring(&self) {
        let mut interval = tokio::time::interval(self.health_check_interval);
        
        loop {
            interval.tick().await;
            
            let health_report = self.collect_health_metrics().await;
            self.evaluate_alerts(&health_report).await;
        }
    }
    
    async fn collect_health_metrics(&self) -> HealthReport {
        HealthReport {
            geocoding_error_rate: self.calculate_geocoding_error_rate().await,
            avg_spatial_search_latency: self.calculate_avg_search_latency().await,
            current_memory_usage: self.get_current_memory_usage(),
            spatial_index_efficiency: self.measure_index_efficiency().await,
            total_locations: self.count_total_locations().await,
        }
    }
    
    async fn evaluate_alerts(&self, health: &HealthReport) {
        // Check geocoding error rate
        if health.geocoding_error_rate > self.alert_thresholds.max_geocoding_error_rate {
            self.send_alert(Alert::HighGeocodingErrorRate {
                current_rate: health.geocoding_error_rate,
                threshold: self.alert_thresholds.max_geocoding_error_rate,
            }).await;
        }
        
        // Check spatial search performance
        if health.avg_spatial_search_latency > self.alert_thresholds.max_spatial_search_latency {
            self.send_alert(Alert::HighSpatialSearchLatency {
                current_latency: health.avg_spatial_search_latency,
                threshold: self.alert_thresholds.max_spatial_search_latency,
            }).await;
        }
        
        // Check memory usage
        if health.current_memory_usage > self.alert_thresholds.max_memory_usage_mb {
            self.send_alert(Alert::HighMemoryUsage {
                current_usage: health.current_memory_usage,
                threshold: self.alert_thresholds.max_memory_usage_mb,
            }).await;
        }
        
        // Check index efficiency
        if health.spatial_index_efficiency < self.alert_thresholds.min_spatial_index_efficiency {
            self.send_alert(Alert::LowIndexEfficiency {
                current_efficiency: health.spatial_index_efficiency,
                threshold: self.alert_thresholds.min_spatial_index_efficiency,
            }).await;
        }
    }
    
    async fn send_alert(&self, alert: Alert) {
        // Send to monitoring system (PagerDuty, Slack, etc.)
        eprintln!("LOCATION DOMAIN ALERT: {:?}", alert);
        
        // Could integrate with:
        // - PagerDuty API
        // - Slack webhook
        // - Email service
        // - NATS alerting topic
    }
}

#[derive(Debug)]
pub enum Alert {
    HighGeocodingErrorRate { current_rate: f64, threshold: f64 },
    HighSpatialSearchLatency { current_latency: Duration, threshold: Duration },
    HighMemoryUsage { current_usage: usize, threshold: usize },
    LowIndexEfficiency { current_efficiency: f64, threshold: f64 },
}
```

### Automated Recovery

```rust
pub struct LocationDomainRecoverySystem {
    health_monitor: LocationHealthMonitor,
    recovery_strategies: Vec<Box<dyn RecoveryStrategy>>,
}

#[async_trait]
pub trait RecoveryStrategy: Send + Sync {
    async fn can_handle(&self, alert: &Alert) -> bool;
    async fn attempt_recovery(&self, alert: &Alert) -> RecoveryResult;
}

pub struct SpatialIndexRecoveryStrategy {
    index_manager: SpatialIndexManager,
}

#[async_trait]
impl RecoveryStrategy for SpatialIndexRecoveryStrategy {
    async fn can_handle(&self, alert: &Alert) -> bool {
        matches!(alert, Alert::LowIndexEfficiency { .. } | Alert::HighSpatialSearchLatency { .. })
    }
    
    async fn attempt_recovery(&self, alert: &Alert) -> RecoveryResult {
        match alert {
            Alert::LowIndexEfficiency { .. } => {
                // Rebuild spatial index
                match self.index_manager.rebuild_index().await {
                    Ok(_) => RecoveryResult::Success {
                        action_taken: "Rebuilt spatial index".to_string(),
                    },
                    Err(e) => RecoveryResult::Failed {
                        error: format!("Failed to rebuild index: {}", e),
                    }
                }
            }
            Alert::HighSpatialSearchLatency { .. } => {
                // Try to optimize current index
                match self.index_manager.optimize_index().await {
                    Ok(_) => RecoveryResult::Success {
                        action_taken: "Optimized spatial index".to_string(),
                    },
                    Err(e) => RecoveryResult::Failed {
                        error: format!("Failed to optimize index: {}", e),
                    }
                }
            }
            _ => RecoveryResult::NotApplicable,
        }
    }
}

#[derive(Debug)]
pub enum RecoveryResult {
    Success { action_taken: String },
    Failed { error: String },
    NotApplicable,
}
```

---

## Emergency Procedures

### Data Recovery Procedures

```rust
pub struct LocationDataRecoveryService {
    backup_locations: Vec<BackupSource>,
    validation_service: LocationValidationService,
}

#[derive(Debug, Clone)]
pub struct BackupSource {
    pub name: String,
    pub priority: u32,
    pub last_backup_time: chrono::DateTime<chrono::Utc>,
    pub backup_type: BackupType,
}

#[derive(Debug, Clone)]
pub enum BackupType {
    DatabaseSnapshot,
    EventSourcing,
    ExternalSystem,
}

impl LocationDataRecoveryService {
    pub async fn emergency_data_recovery(&self, recovery_request: RecoveryRequest) 
        -> Result<RecoveryReport, RecoveryError> {
        
        let mut recovery_report = RecoveryReport::new(recovery_request.clone());
        
        // Step 1: Assess data loss
        let damage_assessment = self.assess_data_damage(&recovery_request).await?;
        recovery_report.damage_assessment = Some(damage_assessment);
        
        // Step 2: Select best recovery sources
        let recovery_sources = self.select_recovery_sources(&damage_assessment);
        
        // Step 3: Recover data from sources
        let mut recovered_locations = Vec::new();
        
        for source in recovery_sources {
            match self.recover_from_source(&source, &recovery_request).await {
                Ok(locations) => {
                    recovered_locations.extend(locations);
                    recovery_report.successful_sources.push(source.name.clone());
                }
                Err(e) => {
                    recovery_report.failed_sources.push(FailedSource {
                        name: source.name.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }
        
        // Step 4: Deduplicate and validate recovered data
        let validated_locations = self.validate_and_deduplicate(recovered_locations).await?;
        
        // Step 5: Restore to system
        let restoration_result = self.restore_locations(validated_locations).await?;
        recovery_report.restoration_result = Some(restoration_result);
        
        Ok(recovery_report)
    }
    
    async fn assess_data_damage(&self, request: &RecoveryRequest) -> Result<DamageAssessment, RecoveryError> {
        match &request.recovery_scope {
            RecoveryScope::FullSystem => {
                // Check system-wide data integrity
                Ok(DamageAssessment {
                    scope: RecoveryScope::FullSystem,
                    estimated_lost_locations: self.count_total_locations().await,
                    affected_regions: vec![], // All regions affected
                    data_corruption_level: CorruptionLevel::Complete,
                })
            }
            RecoveryScope::Region { bounding_box } => {
                // Assess regional damage
                let affected_count = self.count_locations_in_region(bounding_box).await?;
                Ok(DamageAssessment {
                    scope: request.recovery_scope.clone(),
                    estimated_lost_locations: affected_count,
                    affected_regions: vec![bounding_box.clone()],
                    data_corruption_level: self.assess_corruption_level(affected_count).await,
                })
            }
            RecoveryScope::LocationIds { ids } => {
                // Assess specific locations
                let missing_count = self.count_missing_locations(ids).await?;
                Ok(DamageAssessment {
                    scope: request.recovery_scope.clone(),
                    estimated_lost_locations: missing_count,
                    affected_regions: vec![],
                    data_corruption_level: if missing_count == ids.len() { 
                        CorruptionLevel::Complete 
                    } else { 
                        CorruptionLevel::Partial 
                    },
                })
            }
        }
    }
    
    async fn restore_locations(&self, locations: Vec<Location>) -> Result<RestorationResult, RecoveryError> {
        let mut successfully_restored = 0;
        let mut failed_restorations = Vec::new();
        
        // Restore in batches to avoid overwhelming the system
        for batch in locations.chunks(100) {
            match self.restore_batch(batch).await {
                Ok(count) => successfully_restored += count,
                Err(e) => {
                    for location in batch {
                        failed_restorations.push(FailedRestoration {
                            location_id: location.id,
                            error: e.to_string(),
                        });
                    }
                }
            }
            
            // Brief pause between batches
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Ok(RestorationResult {
            total_attempted: locations.len(),
            successfully_restored,
            failed_restorations,
        })
    }
}
```

### System Rollback Procedures

```rust
pub struct LocationSystemRollback {
    event_store: EventStore,
    snapshot_manager: SnapshotManager,
}

impl LocationSystemRollback {
    pub async fn rollback_to_point_in_time(&self, target_time: chrono::DateTime<chrono::Utc>) 
        -> Result<RollbackResult, RollbackError> {
        
        // Step 1: Find appropriate snapshot before target time
        let snapshot = self.find_snapshot_before(target_time).await?;
        
        // Step 2: Restore from snapshot
        self.restore_from_snapshot(&snapshot).await?;
        
        // Step 3: Replay events from snapshot time to target time
        let events = self.event_store.get_events_between(
            snapshot.timestamp, 
            target_time
        ).await?;
        
        let mut replayed_events = 0;
        for event in events {
            if self.should_replay_event(&event, target_time) {
                self.replay_event(event).await?;
                replayed_events += 1;
            }
        }
        
        // Step 4: Rebuild indices and validate consistency
        self.rebuild_system_indices().await?;
        let validation_result = self.validate_system_consistency().await?;
        
        Ok(RollbackResult {
            target_time,
            snapshot_used: snapshot.id,
            events_replayed: replayed_events,
            validation_result,
        })
    }
    
    async fn create_emergency_snapshot(&self) -> Result<SnapshotId, SnapshotError> {
        // Create immediate snapshot before any recovery operations
        let snapshot = SystemSnapshot {
            id: SnapshotId::new(),
            timestamp: chrono::Utc::now(),
            snapshot_type: SnapshotType::Emergency,
            metadata: SnapshotMetadata {
                location_count: self.count_total_locations().await,
                index_versions: self.get_current_index_versions().await,
                system_health: self.assess_system_health().await,
            },
        };
        
        self.snapshot_manager.create_snapshot(snapshot).await
    }
}
```

This comprehensive troubleshooting guide covers the major issues that can occur in a location domain system, providing both diagnostic tools and practical solutions for each category of problem. The documentation emphasizes proactive monitoring, automated recovery, and emergency procedures to maintain system reliability.