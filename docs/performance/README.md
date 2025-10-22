# CIM Location Domain - Performance and Scalability Guide

## Table of Contents
1. [Overview](#overview)
2. [Performance Characteristics](#performance-characteristics)
3. [Spatial Indexing](#spatial-indexing)
4. [Geospatial Query Optimization](#geospatial-query-optimization)
5. [Memory Management](#memory-management)
6. [Hierarchical Processing](#hierarchical-processing)
7. [Benchmarking](#benchmarking)
8. [Production Optimization](#production-optimization)
9. [Monitoring and Observability](#monitoring-and-observability)
10. [Troubleshooting Performance Issues](#troubleshooting-performance-issues)

## Overview

The CIM Location Domain is designed for high-performance geospatial processing and hierarchical location management. This guide covers performance optimization strategies, benchmarking methodologies, and production deployment considerations for location-based systems.

### Key Performance Areas
- **Spatial Queries**: Optimizing coordinate-based searches and proximity calculations
- **Hierarchy Management**: Efficient tree traversal and relationship management
- **Geocoding Operations**: Batch processing and caching strategies
- **Memory Usage**: Spatial index optimization and coordinate storage
- **Concurrent Access**: Thread-safe spatial operations and data structures

## Performance Characteristics

### Spatial Query Performance

```rust
use cim_domain_location::services::{SpatialSearchService, MockSpatialSearchService};
use cim_domain_location::value_objects::Coordinates;
use std::time::Instant;

#[tokio::main]
async fn spatial_query_benchmark() {
    let service = MockSpatialSearchService::new();
    let center = Coordinates::new(37.7749, -122.4194).unwrap();
    
    // Benchmark radius searches
    let start = Instant::now();
    for _ in 0..1000 {
        let _result = service.find_within_radius(&center, 1000.0, None).await;
    }
    let duration = start.elapsed();
    println!("1000 radius searches: {:?} ({:.2} μs/query)", 
             duration, duration.as_micros() as f64 / 1000.0);
    
    // Benchmark nearest neighbor searches
    let start = Instant::now();
    for _ in 0..1000 {
        let _result = service.find_nearest(&center, 10, None, None).await;
    }
    let duration = start.elapsed();
    println!("1000 nearest searches: {:?} ({:.2} μs/query)", 
             duration, duration.as_micros() as f64 / 1000.0);
}
```

### Geocoding Performance Metrics

```rust
use cim_domain_location::services::{GeocodingService, MockGeocodingService};
use cim_domain_location::value_objects::Address;

async fn geocoding_benchmark() {
    let service = MockGeocodingService::new().with_delay(10); // 10ms mock delay
    
    let addresses = vec![
        Address::new(Some("123 Main St".to_string()), Some("San Francisco".to_string()), 
                    Some("CA".to_string()), Some("94102".to_string()), Some("US".to_string())),
        Address::new(Some("456 Market St".to_string()), Some("San Francisco".to_string()), 
                    Some("CA".to_string()), Some("94105".to_string()), Some("US".to_string())),
    ];
    
    // Single geocoding performance
    let start = Instant::now();
    let _result = service.geocode(&addresses[0]).await;
    println!("Single geocode: {:?}", start.elapsed());
    
    // Batch geocoding performance
    let start = Instant::now();
    let _results = service.batch_geocode(&addresses).await;
    println!("Batch geocode (2 addresses): {:?}", start.elapsed());
}
```

## Spatial Indexing

### R-tree Implementation for Spatial Queries

```rust
use std::collections::HashMap;
use uuid::Uuid;
use cim_domain_location::value_objects::Coordinates;

// Simplified R-tree node for spatial indexing
#[derive(Debug, Clone)]
struct SpatialNode {
    bounds: BoundingBox,
    locations: Vec<LocationEntry>,
    children: Vec<SpatialNode>,
}

#[derive(Debug, Clone)]
struct BoundingBox {
    min_lat: f64,
    max_lat: f64,
    min_lng: f64,
    max_lng: f64,
}

#[derive(Debug, Clone)]
struct LocationEntry {
    id: Uuid,
    coordinates: Coordinates,
    metadata: HashMap<String, String>,
}

impl BoundingBox {
    fn contains(&self, point: &Coordinates) -> bool {
        point.latitude >= self.min_lat && point.latitude <= self.max_lat &&
        point.longitude >= self.min_lng && point.longitude <= self.max_lng
    }
    
    fn intersects(&self, other: &BoundingBox) -> bool {
        !(self.max_lat < other.min_lat || self.min_lat > other.max_lat ||
          self.max_lng < other.min_lng || self.min_lng > other.max_lng)
    }
    
    fn area(&self) -> f64 {
        (self.max_lat - self.min_lat) * (self.max_lng - self.min_lng)
    }
}

impl SpatialNode {
    fn new() -> Self {
        Self {
            bounds: BoundingBox {
                min_lat: f64::MAX,
                max_lat: f64::MIN,
                min_lng: f64::MAX,
                max_lng: f64::MIN,
            },
            locations: Vec::new(),
            children: Vec::new(),
        }
    }
    
    fn insert(&mut self, location: LocationEntry) {
        self.update_bounds(&location.coordinates);
        
        if self.children.is_empty() {
            self.locations.push(location);
            
            // Split node if it becomes too large
            if self.locations.len() > 10 {
                self.split();
            }
        } else {
            // Find best child node for insertion
            let best_child = self.choose_leaf(&location.coordinates);
            self.children[best_child].insert(location);
        }
    }
    
    fn search_radius(&self, center: &Coordinates, radius_meters: f64) -> Vec<&LocationEntry> {
        let mut results = Vec::new();
        
        // Check if bounding box intersects with search circle (simplified)
        let bounds_center = Coordinates::new(
            (self.bounds.min_lat + self.bounds.max_lat) / 2.0,
            (self.bounds.min_lng + self.bounds.max_lng) / 2.0
        ).unwrap();
        
        let bounds_distance = haversine_distance(center, &bounds_center);
        let bounds_radius = (self.bounds.area().sqrt()) * 111_000.0; // Approximate conversion
        
        if bounds_distance <= radius_meters + bounds_radius {
            // Check leaf locations
            for location in &self.locations {
                let distance = haversine_distance(center, &location.coordinates);
                if distance <= radius_meters {
                    results.push(location);
                }
            }
            
            // Recursively search children
            for child in &self.children {
                results.extend(child.search_radius(center, radius_meters));
            }
        }
        
        results
    }
    
    fn update_bounds(&mut self, point: &Coordinates) {
        self.bounds.min_lat = self.bounds.min_lat.min(point.latitude);
        self.bounds.max_lat = self.bounds.max_lat.max(point.latitude);
        self.bounds.min_lng = self.bounds.min_lng.min(point.longitude);
        self.bounds.max_lng = self.bounds.max_lng.max(point.longitude);
    }
    
    fn choose_leaf(&self, point: &Coordinates) -> usize {
        // Choose child that would require least enlargement
        self.children.iter().enumerate()
            .min_by_key(|(_, child)| {
                let current_area = child.bounds.area();
                let mut new_bounds = child.bounds.clone();
                new_bounds.min_lat = new_bounds.min_lat.min(point.latitude);
                new_bounds.max_lat = new_bounds.max_lat.max(point.latitude);
                new_bounds.min_lng = new_bounds.min_lng.min(point.longitude);
                new_bounds.max_lng = new_bounds.max_lng.max(point.longitude);
                ((new_bounds.area() - current_area) * 1_000_000.0) as u64
            })
            .map(|(idx, _)| idx)
            .unwrap_or(0)
    }
    
    fn split(&mut self) {
        // Simple split: divide locations between two children
        if !self.locations.is_empty() && self.children.is_empty() {
            let mut left_child = SpatialNode::new();
            let mut right_child = SpatialNode::new();
            
            // Sort by latitude and split
            self.locations.sort_by(|a, b| a.coordinates.latitude.partial_cmp(&b.coordinates.latitude).unwrap());
            let mid = self.locations.len() / 2;
            
            for location in self.locations.drain(..mid) {
                left_child.insert(location);
            }
            for location in self.locations.drain(..) {
                right_child.insert(location);
            }
            
            self.children.push(left_child);
            self.children.push(right_child);
        }
    }
}

// Haversine distance calculation for geo coordinates
fn haversine_distance(coord1: &Coordinates, coord2: &Coordinates) -> f64 {
    const EARTH_RADIUS: f64 = 6_371_000.0; // Earth's radius in meters
    
    let lat1_rad = coord1.latitude.to_radians();
    let lat2_rad = coord2.latitude.to_radians();
    let delta_lat = (coord2.latitude - coord1.latitude).to_radians();
    let delta_lng = (coord2.longitude - coord1.longitude).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2) +
            lat1_rad.cos() * lat2_rad.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS * c
}
```

## Geospatial Query Optimization

### Query Planning and Execution

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

// Optimized spatial query planner
#[derive(Debug)]
struct QueryPlanner {
    spatial_index: Arc<RwLock<SpatialNode>>,
    query_cache: Arc<RwLock<HashMap<String, Vec<LocationEntry>>>>,
}

impl QueryPlanner {
    fn new() -> Self {
        Self {
            spatial_index: Arc::new(RwLock::new(SpatialNode::new())),
            query_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    async fn plan_radius_query(&self, center: &Coordinates, radius: f64) -> QueryPlan {
        let cache_key = format!("radius_{:.6}_{:.6}_{}", center.latitude, center.longitude, radius);
        
        // Check cache first
        if let Some(_cached) = self.query_cache.read().await.get(&cache_key) {
            return QueryPlan::CachedResult { cache_key };
        }
        
        // Determine optimal query strategy based on radius
        match radius {
            r if r <= 100.0 => QueryPlan::PointQuery { center: *center, radius },
            r if r <= 10_000.0 => QueryPlan::SpatialIndex { center: *center, radius },
            _ => QueryPlan::FullScan { center: *center, radius },
        }
    }
    
    async fn execute_query(&self, plan: QueryPlan) -> Vec<LocationEntry> {
        match plan {
            QueryPlan::CachedResult { cache_key } => {
                self.query_cache.read().await.get(&cache_key).cloned().unwrap_or_default()
            },
            QueryPlan::PointQuery { center, radius } => {
                self.execute_point_query(&center, radius).await
            },
            QueryPlan::SpatialIndex { center, radius } => {
                self.execute_spatial_index_query(&center, radius).await
            },
            QueryPlan::FullScan { center, radius } => {
                self.execute_full_scan(&center, radius).await
            },
        }
    }
    
    async fn execute_point_query(&self, center: &Coordinates, radius: f64) -> Vec<LocationEntry> {
        // Optimized for very small radius queries
        let index = self.spatial_index.read().await;
        index.search_radius(center, radius).into_iter().cloned().collect()
    }
    
    async fn execute_spatial_index_query(&self, center: &Coordinates, radius: f64) -> Vec<LocationEntry> {
        // Use spatial index for medium radius queries
        let index = self.spatial_index.read().await;
        let results = index.search_radius(center, radius).into_iter().cloned().collect();
        
        // Cache result for future queries
        let cache_key = format!("radius_{:.6}_{:.6}_{}", center.latitude, center.longitude, radius);
        self.query_cache.write().await.insert(cache_key, results.clone());
        
        results
    }
    
    async fn execute_full_scan(&self, center: &Coordinates, radius: f64) -> Vec<LocationEntry> {
        // For very large radius queries, might be more efficient to scan all locations
        // This is a simplified implementation
        let index = self.spatial_index.read().await;
        index.search_radius(center, radius).into_iter().cloned().collect()
    }
}

#[derive(Debug, Clone)]
enum QueryPlan {
    CachedResult { cache_key: String },
    PointQuery { center: Coordinates, radius: f64 },
    SpatialIndex { center: Coordinates, radius: f64 },
    FullScan { center: Coordinates, radius: f64 },
}
```

## Memory Management

### Efficient Coordinate Storage

```rust
use std::mem;

// Compact coordinate representation for memory efficiency
#[derive(Debug, Clone, Copy, PartialEq)]
struct CompactCoordinate {
    // Store coordinates as fixed-point integers for better cache performance
    lat_fixed: i32,  // Latitude * 1,000,000
    lng_fixed: i32,  // Longitude * 1,000,000
}

impl CompactCoordinate {
    fn new(lat: f64, lng: f64) -> Result<Self, &'static str> {
        if lat.abs() > 90.0 || lng.abs() > 180.0 {
            return Err("Invalid coordinates");
        }
        
        Ok(Self {
            lat_fixed: (lat * 1_000_000.0).round() as i32,
            lng_fixed: (lng * 1_000_000.0).round() as i32,
        })
    }
    
    fn latitude(&self) -> f64 {
        self.lat_fixed as f64 / 1_000_000.0
    }
    
    fn longitude(&self) -> f64 {
        self.lng_fixed as f64 / 1_000_000.0
    }
    
    fn distance_to(&self, other: &CompactCoordinate) -> f64 {
        haversine_distance_compact(self, other)
    }
}

fn haversine_distance_compact(coord1: &CompactCoordinate, coord2: &CompactCoordinate) -> f64 {
    const EARTH_RADIUS: f64 = 6_371_000.0;
    
    let lat1 = coord1.latitude().to_radians();
    let lat2 = coord2.latitude().to_radians();
    let delta_lat = (coord2.latitude() - coord1.latitude()).to_radians();
    let delta_lng = (coord2.longitude() - coord1.longitude()).to_radians();
    
    let a = (delta_lat / 2.0).sin().powi(2) +
            lat1.cos() * lat2.cos() * (delta_lng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    
    EARTH_RADIUS * c
}

// Memory-efficient location storage
#[derive(Debug)]
struct CompactLocationStore {
    coordinates: Vec<CompactCoordinate>,
    metadata: Vec<LocationMetadata>,
    spatial_index: Vec<usize>, // Indices sorted by spatial hash
}

#[derive(Debug, Clone)]
struct LocationMetadata {
    id: Uuid,
    name: Option<String>,
    category: String,
    tags: Vec<String>,
}

impl CompactLocationStore {
    fn new() -> Self {
        Self {
            coordinates: Vec::new(),
            metadata: Vec::new(),
            spatial_index: Vec::new(),
        }
    }
    
    fn insert(&mut self, coord: CompactCoordinate, metadata: LocationMetadata) {
        let index = self.coordinates.len();
        self.coordinates.push(coord);
        self.metadata.push(metadata);
        
        // Insert into spatial index maintaining sorted order
        let spatial_hash = self.compute_spatial_hash(&coord);
        let insert_pos = self.spatial_index.binary_search_by_key(&spatial_hash, |&idx| {
            self.compute_spatial_hash(&self.coordinates[idx])
        }).unwrap_or_else(|pos| pos);
        
        self.spatial_index.insert(insert_pos, index);
    }
    
    fn find_nearby(&self, center: &CompactCoordinate, radius_meters: f64) -> Vec<(usize, f64)> {
        let mut results = Vec::new();
        let center_hash = self.compute_spatial_hash(center);
        
        // Search around the spatial hash (simplified implementation)
        for &idx in &self.spatial_index {
            let distance = self.coordinates[idx].distance_to(center);
            if distance <= radius_meters {
                results.push((idx, distance));
            }
            
            // Early termination optimization
            let coord_hash = self.compute_spatial_hash(&self.coordinates[idx]);
            if coord_hash > center_hash + 1000 && results.len() > 10 {
                break;
            }
        }
        
        results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        results
    }
    
    fn compute_spatial_hash(&self, coord: &CompactCoordinate) -> u64 {
        // Simple spatial hash for demonstration
        ((coord.lat_fixed as u64) << 32) | (coord.lng_fixed as u64 & 0xFFFFFFFF)
    }
    
    fn memory_usage(&self) -> usize {
        mem::size_of::<Self>() +
        self.coordinates.capacity() * mem::size_of::<CompactCoordinate>() +
        self.metadata.iter().map(|m| {
            mem::size_of::<LocationMetadata>() +
            m.name.as_ref().map(|n| n.len()).unwrap_or(0) +
            m.category.len() +
            m.tags.iter().map(|t| t.len()).sum::<usize>()
        }).sum::<usize>() +
        self.spatial_index.capacity() * mem::size_of::<usize>()
    }
}
```

## Hierarchical Processing

### Efficient Tree Operations

```rust
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct LocationHierarchy {
    nodes: HashMap<Uuid, HierarchyNode>,
    root_nodes: Vec<Uuid>,
    parent_map: HashMap<Uuid, Uuid>,
    children_map: HashMap<Uuid, Vec<Uuid>>,
}

#[derive(Debug, Clone)]
struct HierarchyNode {
    id: Uuid,
    name: String,
    level: u32,
    metadata: HashMap<String, String>,
}

impl LocationHierarchy {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root_nodes: Vec::new(),
            parent_map: HashMap::new(),
            children_map: HashMap::new(),
        }
    }
    
    fn add_node(&mut self, node: HierarchyNode, parent_id: Option<Uuid>) {
        let node_id = node.id;
        self.nodes.insert(node_id, node);
        
        match parent_id {
            Some(parent) => {
                self.parent_map.insert(node_id, parent);
                self.children_map.entry(parent).or_insert_with(Vec::new).push(node_id);
            },
            None => {
                self.root_nodes.push(node_id);
            },
        }
    }
    
    // Optimized ancestor traversal with path caching
    fn get_ancestors(&self, node_id: &Uuid) -> Vec<Uuid> {
        let mut ancestors = Vec::new();
        let mut current = *node_id;
        
        while let Some(&parent) = self.parent_map.get(&current) {
            ancestors.push(parent);
            current = parent;
        }
        
        ancestors.reverse();
        ancestors
    }
    
    // Breadth-first traversal for level-order processing
    fn get_descendants_bfs(&self, node_id: &Uuid, max_depth: Option<u32>) -> Vec<Uuid> {
        let mut results = Vec::new();
        let mut queue = std::collections::VecDeque::new();
        
        if let Some(children) = self.children_map.get(node_id) {
            for &child in children {
                queue.push_back((child, 1));
            }
        }
        
        while let Some((current, depth)) = queue.pop_front() {
            if let Some(max) = max_depth {
                if depth > max {
                    continue;
                }
            }
            
            results.push(current);
            
            if let Some(children) = self.children_map.get(&current) {
                for &child in children {
                    queue.push_back((child, depth + 1));
                }
            }
        }
        
        results
    }
    
    // Efficient subtree operations
    fn subtree_size(&self, node_id: &Uuid) -> usize {
        let mut size = 1; // Include the node itself
        
        if let Some(children) = self.children_map.get(node_id) {
            for &child in children {
                size += self.subtree_size(&child);
            }
        }
        
        size
    }
    
    // Batch hierarchy operations for performance
    fn batch_move_nodes(&mut self, moves: Vec<(Uuid, Option<Uuid>)>) -> Result<(), String> {
        // Validate all moves first to prevent partial state
        for (node_id, new_parent) in &moves {
            if let Some(new_parent) = new_parent {
                if self.would_create_cycle(*node_id, *new_parent) {
                    return Err(format!("Move would create cycle: {} -> {}", node_id, new_parent));
                }
            }
        }
        
        // Apply all moves atomically
        for (node_id, new_parent) in moves {
            self.move_node(node_id, new_parent);
        }
        
        Ok(())
    }
    
    fn move_node(&mut self, node_id: Uuid, new_parent: Option<Uuid>) {
        // Remove from old parent
        if let Some(old_parent) = self.parent_map.remove(&node_id) {
            if let Some(siblings) = self.children_map.get_mut(&old_parent) {
                siblings.retain(|&x| x != node_id);
            }
        } else {
            self.root_nodes.retain(|&x| x != node_id);
        }
        
        // Add to new parent
        match new_parent {
            Some(parent) => {
                self.parent_map.insert(node_id, parent);
                self.children_map.entry(parent).or_insert_with(Vec::new).push(node_id);
            },
            None => {
                self.root_nodes.push(node_id);
            },
        }
    }
    
    fn would_create_cycle(&self, node_id: Uuid, potential_parent: Uuid) -> bool {
        // Check if potential_parent is a descendant of node_id
        self.get_descendants_bfs(&node_id, None).contains(&potential_parent)
    }
}
```

## Benchmarking

### Comprehensive Performance Test Suite

```rust
use std::time::{Duration, Instant};
use tokio::time::sleep;

#[derive(Debug)]
struct PerformanceMetrics {
    operation: String,
    avg_duration: Duration,
    min_duration: Duration,
    max_duration: Duration,
    p95_duration: Duration,
    p99_duration: Duration,
    throughput_per_second: f64,
    memory_usage_kb: usize,
}

struct LocationDomainBenchmark {
    spatial_store: CompactLocationStore,
    hierarchy: LocationHierarchy,
}

impl LocationDomainBenchmark {
    fn new() -> Self {
        Self {
            spatial_store: CompactLocationStore::new(),
            hierarchy: LocationHierarchy::new(),
        }
    }
    
    async fn setup_test_data(&mut self, num_locations: usize) {
        println!("Setting up {} test locations...", num_locations);
        
        for i in 0..num_locations {
            let lat = 37.7749 + (i as f64 * 0.001) % 1.0 - 0.5;
            let lng = -122.4194 + (i as f64 * 0.001) % 1.0 - 0.5;
            
            let coord = CompactCoordinate::new(lat, lng).unwrap();
            let metadata = LocationMetadata {
                id: Uuid::new_v4(),
                name: Some(format!("Location {}", i)),
                category: "test".to_string(),
                tags: vec!["benchmark".to_string()],
            };
            
            self.spatial_store.insert(coord, metadata);
        }
    }
    
    async fn benchmark_spatial_queries(&self, iterations: usize) -> PerformanceMetrics {
        let mut durations = Vec::with_capacity(iterations);
        let center = CompactCoordinate::new(37.7749, -122.4194).unwrap();
        
        for _ in 0..iterations {
            let start = Instant::now();
            let _results = self.spatial_store.find_nearby(&center, 1000.0);
            durations.push(start.elapsed());
        }
        
        self.calculate_metrics("spatial_queries", durations)
    }
    
    async fn benchmark_hierarchy_operations(&self, iterations: usize) -> PerformanceMetrics {
        let mut durations = Vec::with_capacity(iterations);
        let test_node_id = Uuid::new_v4();
        
        for _ in 0..iterations {
            let start = Instant::now();
            let _ancestors = self.hierarchy.get_ancestors(&test_node_id);
            let _descendants = self.hierarchy.get_descendants_bfs(&test_node_id, Some(5));
            durations.push(start.elapsed());
        }
        
        self.calculate_metrics("hierarchy_operations", durations)
    }
    
    async fn benchmark_geocoding_throughput(&self, iterations: usize) -> PerformanceMetrics {
        let service = MockGeocodingService::new().with_delay(1); // 1ms delay
        let address = Address::new(
            Some("123 Test St".to_string()),
            Some("San Francisco".to_string()),
            Some("CA".to_string()),
            Some("94102".to_string()),
            Some("US".to_string()),
        );
        
        let mut durations = Vec::with_capacity(iterations);
        
        for _ in 0..iterations {
            let start = Instant::now();
            let _result = service.geocode(&address).await;
            durations.push(start.elapsed());
        }
        
        self.calculate_metrics("geocoding_operations", durations)
    }
    
    fn calculate_metrics(&self, operation: String, mut durations: Vec<Duration>) -> PerformanceMetrics {
        durations.sort();
        
        let total_time: Duration = durations.iter().sum();
        let avg_duration = total_time / durations.len() as u32;
        let min_duration = durations[0];
        let max_duration = durations[durations.len() - 1];
        let p95_duration = durations[(durations.len() as f64 * 0.95) as usize];
        let p99_duration = durations[(durations.len() as f64 * 0.99) as usize];
        
        let throughput_per_second = durations.len() as f64 / total_time.as_secs_f64();
        let memory_usage_kb = self.spatial_store.memory_usage() / 1024;
        
        PerformanceMetrics {
            operation,
            avg_duration,
            min_duration,
            max_duration,
            p95_duration,
            p99_duration,
            throughput_per_second,
            memory_usage_kb,
        }
    }
    
    async fn run_full_benchmark_suite(&mut self) -> Vec<PerformanceMetrics> {
        println!("Starting CIM Location Domain Performance Benchmark");
        
        // Setup test data
        self.setup_test_data(10_000).await;
        
        // Run benchmarks
        let mut results = Vec::new();
        
        results.push(self.benchmark_spatial_queries(1000).await);
        results.push(self.benchmark_hierarchy_operations(1000).await);
        results.push(self.benchmark_geocoding_throughput(1000).await);
        
        results
    }
    
    fn print_results(&self, results: &[PerformanceMetrics]) {
        println!("\n=== CIM Location Domain Performance Results ===");
        
        for metric in results {
            println!("\n--- {} ---", metric.operation);
            println!("Average Duration: {:?}", metric.avg_duration);
            println!("Min Duration: {:?}", metric.min_duration);
            println!("Max Duration: {:?}", metric.max_duration);
            println!("P95 Duration: {:?}", metric.p95_duration);
            println!("P99 Duration: {:?}", metric.p99_duration);
            println!("Throughput: {:.2} ops/sec", metric.throughput_per_second);
            println!("Memory Usage: {} KB", metric.memory_usage_kb);
        }
    }
}

#[tokio::main]
async fn main() {
    let mut benchmark = LocationDomainBenchmark::new();
    let results = benchmark.run_full_benchmark_suite().await;
    benchmark.print_results(&results);
}
```

## Production Optimization

### Configuration for High-Performance Deployments

```toml
# Production Cargo.toml optimizations
[profile.release]
opt-level = 3
lto = "fat"
codegen-units = 1
panic = "abort"
overflow-checks = false

# Additional optimizations for geospatial processing
[profile.release.package.geo]
opt-level = 3

[profile.release.package.cim-domain-location]
opt-level = 3
debug = false
```

### Runtime Configuration

```rust
use tokio::runtime::Builder;
use std::sync::Arc;

fn create_optimized_runtime() -> tokio::runtime::Runtime {
    Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .thread_name("location-worker")
        .thread_stack_size(2 * 1024 * 1024) // 2MB stack
        .enable_all()
        .build()
        .expect("Failed to create Tokio runtime")
}

#[derive(Clone)]
pub struct LocationDomainConfig {
    pub spatial_index_capacity: usize,
    pub geocoding_batch_size: usize,
    pub cache_ttl_seconds: u64,
    pub max_hierarchy_depth: u32,
    pub coordinate_precision: u8,
}

impl Default for LocationDomainConfig {
    fn default() -> Self {
        Self {
            spatial_index_capacity: 100_000,
            geocoding_batch_size: 100,
            cache_ttl_seconds: 3600, // 1 hour
            max_hierarchy_depth: 10,
            coordinate_precision: 6, // ~1 meter precision
        }
    }
}

pub struct OptimizedLocationService {
    config: LocationDomainConfig,
    spatial_store: Arc<tokio::sync::RwLock<CompactLocationStore>>,
    hierarchy: Arc<tokio::sync::RwLock<LocationHierarchy>>,
    query_planner: Arc<QueryPlanner>,
}

impl OptimizedLocationService {
    pub fn new(config: LocationDomainConfig) -> Self {
        Self {
            config,
            spatial_store: Arc::new(tokio::sync::RwLock::new(CompactLocationStore::new())),
            hierarchy: Arc::new(tokio::sync::RwLock::new(LocationHierarchy::new())),
            query_planner: Arc::new(QueryPlanner::new()),
        }
    }
    
    pub async fn warmup_caches(&self) {
        // Pre-populate commonly accessed data
        println!("Warming up location service caches...");
        
        // Warm up spatial index
        let store = self.spatial_store.read().await;
        println!("Spatial store loaded with {} locations", store.coordinates.len());
        
        // Warm up hierarchy caches
        let hierarchy = self.hierarchy.read().await;
        println!("Hierarchy loaded with {} nodes", hierarchy.nodes.len());
    }
    
    pub async fn optimize_for_read_heavy_workload(&self) {
        // Configure for read-heavy workloads (typical for location services)
        println!("Optimizing for read-heavy location workloads");
        
        // Additional optimizations could include:
        // - Pre-computing spatial indices
        // - Warming up query caches
        // - Loading frequently accessed hierarchies
    }
}
```

## Monitoring and Observability

### Performance Metrics Collection

```rust
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

pub struct LocationMetricsCollector {
    spatial_query_count: AtomicU64,
    spatial_query_duration: AtomicU64,
    geocoding_requests: AtomicU64,
    geocoding_duration: AtomicU64,
    hierarchy_operations: AtomicU64,
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl LocationMetricsCollector {
    pub fn new() -> Self {
        Self {
            spatial_query_count: AtomicU64::new(0),
            spatial_query_duration: AtomicU64::new(0),
            geocoding_requests: AtomicU64::new(0),
            geocoding_duration: AtomicU64::new(0),
            hierarchy_operations: AtomicU64::new(0),
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        }
    }
    
    pub fn record_spatial_query(&self, duration: Duration) {
        self.spatial_query_count.fetch_add(1, Ordering::Relaxed);
        self.spatial_query_duration.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }
    
    pub fn record_geocoding_request(&self, duration: Duration) {
        self.geocoding_requests.fetch_add(1, Ordering::Relaxed);
        self.geocoding_duration.fetch_add(duration.as_micros() as u64, Ordering::Relaxed);
    }
    
    pub fn record_hierarchy_operation(&self) {
        self.hierarchy_operations.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }
    
    pub fn get_metrics_snapshot(&self) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        
        let spatial_count = self.spatial_query_count.load(Ordering::Relaxed);
        let spatial_duration = self.spatial_query_duration.load(Ordering::Relaxed);
        
        metrics.insert("spatial_queries_total".to_string(), spatial_count as f64);
        if spatial_count > 0 {
            metrics.insert("spatial_query_avg_microseconds".to_string(), 
                          spatial_duration as f64 / spatial_count as f64);
        }
        
        let geocoding_count = self.geocoding_requests.load(Ordering::Relaxed);
        let geocoding_duration = self.geocoding_duration.load(Ordering::Relaxed);
        
        metrics.insert("geocoding_requests_total".to_string(), geocoding_count as f64);
        if geocoding_count > 0 {
            metrics.insert("geocoding_avg_microseconds".to_string(),
                          geocoding_duration as f64 / geocoding_count as f64);
        }
        
        let hierarchy_ops = self.hierarchy_operations.load(Ordering::Relaxed);
        metrics.insert("hierarchy_operations_total".to_string(), hierarchy_ops as f64);
        
        let cache_hits = self.cache_hits.load(Ordering::Relaxed);
        let cache_misses = self.cache_misses.load(Ordering::Relaxed);
        let total_cache_ops = cache_hits + cache_misses;
        
        metrics.insert("cache_hits_total".to_string(), cache_hits as f64);
        metrics.insert("cache_misses_total".to_string(), cache_misses as f64);
        if total_cache_ops > 0 {
            metrics.insert("cache_hit_rate".to_string(), cache_hits as f64 / total_cache_ops as f64);
        }
        
        metrics
    }
}
```

## Troubleshooting Performance Issues

### Common Performance Problems and Solutions

1. **Slow Spatial Queries**
   - **Problem**: High latency for nearby location searches
   - **Solution**: Optimize spatial indexing and add query result caching

2. **Memory Usage Growth**
   - **Problem**: Increasing memory consumption over time
   - **Solution**: Implement coordinate compression and cache eviction

3. **Geocoding Bottlenecks**
   - **Problem**: Sequential geocoding requests causing delays
   - **Solution**: Implement batch processing and connection pooling

4. **Hierarchy Traversal Performance**
   - **Problem**: Slow ancestor/descendant queries
   - **Solution**: Cache path computations and use iterative algorithms

### Performance Monitoring Dashboard

```rust
pub async fn generate_performance_report(collector: &LocationMetricsCollector) -> String {
    let metrics = collector.get_metrics_snapshot();
    
    format!(
        r#"
=== CIM Location Domain Performance Report ===

Spatial Queries:
  Total Queries: {:.0}
  Average Duration: {:.2} μs
  
Geocoding:
  Total Requests: {:.0}
  Average Duration: {:.2} μs
  
Hierarchy Operations:
  Total Operations: {:.0}
  
Caching:
  Cache Hit Rate: {:.1}%
  Total Cache Hits: {:.0}
  Total Cache Misses: {:.0}

Recommendations:
{}
        "#,
        metrics.get("spatial_queries_total").unwrap_or(&0.0),
        metrics.get("spatial_query_avg_microseconds").unwrap_or(&0.0),
        metrics.get("geocoding_requests_total").unwrap_or(&0.0),
        metrics.get("geocoding_avg_microseconds").unwrap_or(&0.0),
        metrics.get("hierarchy_operations_total").unwrap_or(&0.0),
        metrics.get("cache_hit_rate").unwrap_or(&0.0) * 100.0,
        metrics.get("cache_hits_total").unwrap_or(&0.0),
        metrics.get("cache_misses_total").unwrap_or(&0.0),
        generate_recommendations(&metrics)
    )
}

fn generate_recommendations(metrics: &HashMap<String, f64>) -> String {
    let mut recommendations = Vec::new();
    
    if let Some(hit_rate) = metrics.get("cache_hit_rate") {
        if *hit_rate < 0.8 {
            recommendations.push("- Consider increasing cache size or adjusting TTL");
        }
    }
    
    if let Some(avg_duration) = metrics.get("spatial_query_avg_microseconds") {
        if *avg_duration > 1000.0 {
            recommendations.push("- Optimize spatial indexing or add query result caching");
        }
    }
    
    if let Some(geo_duration) = metrics.get("geocoding_avg_microseconds") {
        if *geo_duration > 10000.0 {
            recommendations.push("- Implement geocoding result caching or batch processing");
        }
    }
    
    if recommendations.is_empty() {
        "- Performance looks good!".to_string()
    } else {
        recommendations.join("\n")
    }
}
```

This comprehensive performance guide provides the foundation for building high-performance location-based systems using the CIM Location Domain. It covers spatial indexing, query optimization, memory management, and monitoring strategies essential for production deployments.