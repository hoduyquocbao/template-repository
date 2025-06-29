//! Example: Cache và Pool Operations
//!
//! Minh họa cách sử dụng Cache và Pool:
//! - Cache với TTL và cleanup
//! - Pool connection management
//! - Performance optimization
//! - Concurrent access patterns

use kernel::{
    storage::{pool::Pool, cache::Cache, time::now}
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Simulate database connection
#[derive(Clone, Debug)]
struct DbConnection {
    id: String,
    created: u128,
}

impl DbConnection {
    fn new(id: String) -> Self {
        Self {
            id,
            created: now(),
        }
    }
    
    fn execute_query(&self, query: &str) -> String {
        format!("Result from connection {}: {}", self.id, query)
    }
}

/// Simulate expensive computation result
#[derive(Clone, Debug)]
struct ComputationResult {
    input: String,
    result: String,
    computed_at: u128,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Cache và Pool Operations Example ===");
    
    // 1. Pool Example
    println!("\n1. Connection Pool Example...");
    
    let pool = Pool::new(3, || {
        let conn_id = format!("conn_{}", now() % 1000);
        Ok(DbConnection::new(conn_id))
    })?;
    
    println!("✓ Created pool with 3 connections");
    println!("✓ Available connections: {}", pool.free());
    
    // Simulate concurrent database operations
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            let conn = pool_clone.get().await.unwrap();
            let result = conn.execute_query(&format!("SELECT * FROM table_{}", i));
            println!("  Thread {}: {}", i, result);
            sleep(Duration::from_millis(100)).await; // Simulate work
            println!("  Thread {}: Released connection", i);
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await?;
    }
    
    println!("✓ All pool operations completed");
    println!("✓ Available connections: {}", pool.free());
    
    // 2. Cache Example
    println!("\n2. Cache Operations Example...");
    
    let cache = Cache::new(Duration::from_secs(5)); // 5 second TTL
    println!("✓ Created cache with 5 second TTL");
    
    // Set multiple cache entries
    for i in 0..5 {
        let key = format!("user_{}", i);
        let value = ComputationResult {
            input: format!("input_{}", i),
            result: format!("expensive_computation_result_{}", i),
            computed_at: now(),
        };
        cache.set(key, value).await;
        println!("  ✓ Cached: user_{}", i);
    }
    
    // Get cache entries
    for i in 0..5 {
        let key = format!("user_{}", i);
        if let Some(result) = cache.get(&key).await {
            println!("  ✓ Cache hit: {} -> {:?}", key, result);
        } else {
            println!("  ✗ Cache miss: {}", key);
        }
    }
    
    // 3. Cache TTL Test
    println!("\n3. Cache TTL Test...");
    
    let short_ttl_cache = Cache::new(Duration::from_millis(500)); // 500ms TTL
    short_ttl_cache.set("temp_key".to_string(), "temp_value".to_string()).await;
    println!("  ✓ Set temp_key with 500ms TTL");
    
    // Immediate access
    if let Some(value) = short_ttl_cache.get(&"temp_key".to_string()).await {
        println!("  ✓ Immediate access: {}", value);
    }
    
    // Wait for TTL to expire
    println!("  Waiting for TTL to expire...");
    sleep(Duration::from_millis(600)).await;
    
    if let Some(value) = short_ttl_cache.get(&"temp_key".to_string()).await {
        println!("  ✓ Still available after TTL: {}", value);
    } else {
        println!("  ✗ Expired as expected");
    }
    
    // 4. Cache Cleanup
    println!("\n4. Cache Cleanup Example...");
    
    let cleanup_cache = Cache::new(Duration::from_millis(100)); // Very short TTL
    
    // Add entries
    for i in 0..10 {
        let key = format!("cleanup_test_{}", i);
        cleanup_cache.set(key, format!("value_{}", i)).await;
    }
    println!("  ✓ Added 10 entries to cleanup cache");
    
    // Wait for some to expire
    sleep(Duration::from_millis(150)).await;
    
    // Manual cleanup
    cleanup_cache.clean().await;
    println!("  ✓ Performed manual cleanup");
    
    // Check remaining entries
    let mut remaining = 0;
    for i in 0..10 {
        let key = format!("cleanup_test_{}", i);
        if cleanup_cache.get(&key).await.is_some() {
            remaining += 1;
        }
    }
    println!("  ✓ Remaining entries after cleanup: {}", remaining);
    
    // 5. Concurrent Cache Access
    println!("\n5. Concurrent Cache Access...");
    
    let concurrent_cache = Cache::new(Duration::from_secs(10));
    let mut handles = Vec::new();
    
    // Spawn multiple tasks that read/write cache
    for i in 0..3 {
        let cache_clone = concurrent_cache.clone();
        let handle = tokio::spawn(async move {
            for j in 0..5 {
                let key = format!("concurrent_{}_{}", i, j);
                let value = format!("value_{}_{}", i, j);
                
                // Write
                cache_clone.set(key.clone(), value.clone()).await;
                
                // Read
                if let Some(cached_value) = cache_clone.get(&key).await {
                    println!("    Thread {}: {} -> {}", i, key, cached_value);
                }
                
                sleep(Duration::from_millis(50)).await;
            }
        });
        handles.push(handle);
    }
    
    // Wait for all concurrent operations
    for handle in handles {
        handle.await?;
    }
    
    println!("  ✓ All concurrent cache operations completed");
    
    // 6. Performance Comparison
    println!("\n6. Performance Comparison...");
    
    let test_iterations = 1000;
    
    // Test without cache
    let start = Instant::now();
    for i in 0..test_iterations {
        let _result = expensive_operation(i);
    }
    let without_cache = start.elapsed();
    println!("  ✓ Without cache: {:?} for {} operations", without_cache, test_iterations);
    
    // Test with cache
    let perf_cache = Cache::new(Duration::from_secs(30));
    let start = Instant::now();
    
    for i in 0..test_iterations {
        let key = format!("perf_test_{}", i % 10); // Only 10 unique keys
        
        if let Some(_result) = perf_cache.get(&key).await {
            // Cache hit
        } else {
            // Cache miss - compute and store
            let result = expensive_operation(i);
            perf_cache.set(key, result).await;
        }
    }
    
    let with_cache = start.elapsed();
    println!("  ✓ With cache: {:?} for {} operations", with_cache, test_iterations);
    
    let speedup = without_cache.as_nanos() as f64 / with_cache.as_nanos() as f64;
    println!("  ✓ Speedup: {:.2}x", speedup);
    
    println!("\n=== Cache và Pool Example completed successfully! ===");
    Ok(())
}

/// Simulate expensive operation
fn expensive_operation(input: usize) -> String {
    // Simulate CPU-intensive work
    let mut result = 0;
    for i in 0..1000 {
        result += input * i;
    }
    format!("expensive_result_{}", result)
} 