//! Example: Time Utilities và Timestamp Operations
//!
//! Minh họa cách sử dụng time utilities:
//! - Timestamp generation và manipulation
//! - Time-based indexing
//! - Performance timing
//! - Time-based queries

use kernel::{
    storage::{Storage, time::now, entity::{Entity, Query, Key}},
    Sled, Id
};
use serde::{Serialize, Deserialize};
use std::time::{Duration, Instant};

/// Entity với time-based indexing
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Event {
    id: Id,
    name: String,
    timestamp: u128,
    priority: u8,
    data: String,
}

/// Summary cho Event
#[derive(Serialize, Deserialize, Debug, Clone)]
struct EventSummary {
    id: Id,
    name: String,
    timestamp: u128,
    priority: u8,
}

impl Entity for Event {
    const NAME: &'static str = "events";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = EventSummary;
    
    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn index(&self) -> Self::Index {
        // Index: priority + timestamp_reversed + id
        Key::reserve(25)
            .byte(self.priority)
            .time(self.timestamp)
            .id(self.id).clone()
            .build()
    }
    
    fn summary(&self) -> Self::Summary {
        EventSummary {
            id: self.id,
            name: self.name.clone(),
            timestamp: self.timestamp,
            priority: self.priority,
        }
    }
}

/// Tạo events với timestamp khác nhau
fn generate_events(count: usize) -> Vec<Event> {
    let mut events = Vec::with_capacity(count);
    let base_time = now();
    
    for i in 0..count {
        let event = Event {
            id: Id::new_v4(),
            name: format!("Event {}", i + 1),
            timestamp: base_time - (i as u128 * 1000), // Staggered timestamps
            priority: (i % 3) as u8, // 0, 1, 2 priorities
            data: format!("Data for event {}", i + 1),
        };
        events.push(event);
    }
    events
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Time Utilities Example ===");
    
    // 1. Basic Time Operations
    println!("\n1. Basic Time Operations...");
    
    let start_time = now();
    println!("  ✓ Current timestamp: {}", start_time);
    
    // Wait a bit
    tokio::time::sleep(Duration::from_millis(100)).await;
    
    let end_time = now();
    println!("  ✓ After 100ms: {}", end_time);
    println!("  ✓ Time difference: {} nanoseconds", end_time - start_time);
    
    // 2. Time-based Entity Creation
    println!("\n2. Time-based Entity Creation...");
    
    let storage = Sled::new("./time_example")?;
    
    let events = generate_events(20);
    println!("  ✓ Generated {} events with staggered timestamps", events.len());
    
    // Insert events
    for event in &events {
        storage.insert(event.clone()).await?;
    }
    println!("  ✓ Inserted all events");
    
    // 3. Time-based Queries
    println!("\n3. Time-based Queries...");
    
    // Query high priority events (priority = 2)
    let mut key_hp = Key::reserve(1);
    key_hp.byte(2);
    let high_priority_query = Query {
        prefix: key_hp.clone().build(),
        after: None,
        limit: 5,
    };
    
    let high_priority_events: Vec<_> = storage.query::<Event>(high_priority_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    
    println!("  ✓ High priority events (priority 2):");
    for event in &high_priority_events {
        println!("    - {} (timestamp: {})", event.name, event.timestamp);
    }
    
    // Query medium priority events (priority = 1)
    let mut key_mp = Key::reserve(1);
    key_mp.byte(1);
    let medium_priority_query = Query {
        prefix: key_mp.clone().build(),
        after: None,
        limit: 5,
    };
    
    let medium_priority_events: Vec<_> = storage.query::<Event>(medium_priority_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    
    println!("  ✓ Medium priority events (priority 1):");
    for event in &medium_priority_events {
        println!("    - {} (timestamp: {})", event.name, event.timestamp);
    }
    
    // 4. Recent Events Query
    println!("\n4. Recent Events Query...");
    
    // Get most recent events (all priorities)
    let recent_query = Query {
        prefix: Vec::new(), // All priorities
        after: None,
        limit: 10,
    };
    
    let recent_events: Vec<_> = storage.query::<Event>(recent_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    
    println!("  ✓ Most recent events:");
    for event in &recent_events {
        println!("    - {} (priority: {}, timestamp: {})", 
                event.name, event.priority, event.timestamp);
    }
    
    // 5. Time Range Queries
    println!("\n5. Time Range Queries...");
    
    let current_time = now();
    let one_minute_ago = current_time - 60_000_000_000; // 60 seconds in nanoseconds
    
    // Find events from the last minute
    println!("  ✓ Events from last minute (timestamp > {}):", one_minute_ago);
    
    let time_range_query = Query {
        prefix: Vec::new(),
        after: None,
        limit: 20,
    };
    
    let all_events: Vec<_> = storage.query::<Event>(time_range_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    
    let recent_events: Vec<_> = all_events.iter()
        .filter(|event| event.timestamp > one_minute_ago)
        .collect();
    
    for event in recent_events {
        println!("    - {} (timestamp: {})", event.name, event.timestamp);
    }
    
    // 6. Performance Timing
    println!("\n6. Performance Timing...");
    
    let timing_start = Instant::now();
    
    // Perform multiple operations and measure time
    for _i in 0..100 {
        let query = Query {
            prefix: Vec::new(),
            after: None,
            limit: 5,
        };
        
        let _results: Vec<_> = storage.query::<Event>(query).await?
            .collect::<Result<Vec<_>, _>>()?;
    }
    
    let timing_duration = timing_start.elapsed();
    println!("  ✓ 100 queries completed in {:?}", timing_duration);
    println!("  ✓ Average time per query: {:?}", timing_duration / 100);
    
    // 7. Time-based Indexing Analysis
    println!("\n7. Time-based Indexing Analysis...");
    
    // Analyze index structure
    println!("  ✓ Index structure analysis:");
    for event in events.iter().take(5) {
        let index = event.index();
        println!("    Event: {} -> Index length: {} bytes", event.name, index.len());
        
        // Decode index components
        if index.len() >= 25 {
            let priority = index[0];
            let timestamp_bytes = &index[1..17];
            let id_bytes = &index[17..33];
            
            println!("      Priority: {}", priority);
            println!("      Timestamp bytes: {:?}", timestamp_bytes);
            println!("      ID bytes: {:?}", id_bytes);
        }
    }
    
    // 8. Time Utilities for Application Logic
    println!("\n8. Time Utilities for Application Logic...");
    
    // Simulate application that needs time-based decisions
    let app_start_time = now();
    
    // Simulate processing events
    for event in &events[..5] {
        let processing_start = now();
        
        // Simulate processing time
        tokio::time::sleep(Duration::from_millis(10)).await;
        
        let processing_end = now();
        let processing_duration = processing_end - processing_start;
        
        println!("  ✓ Processed {} in {} nanoseconds", event.name, processing_duration);
    }
    
    let app_total_time = now() - app_start_time;
    println!("  ✓ Total application time: {} nanoseconds", app_total_time);
    
    // 9. Cleanup
    println!("\n9. Cleanup...");
    
    let mut deleted_count = 0;
    for event in &events[..10] { // Delete first 10 for demo
        match storage.delete::<Event>(event.id).await {
            Ok(_) => {
                deleted_count += 1;
                println!("  ✓ Deleted event: {}", event.name);
            }
            Err(e) => println!("  ✗ Failed to delete event {}: {:?}", event.name, e),
        }
    }
    
    println!("  ✓ Successfully deleted {}/10 events", deleted_count);
    
    println!("\n=== Time Utilities Example completed successfully! ===");
    Ok(())
} 