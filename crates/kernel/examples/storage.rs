//! Example: Sử dụng Storage với Entity, Pool, Cache, và Time
//!
//! Minh họa cách sử dụng các thành phần storage đã được tổ chức lại:
//! - Entity trait và Query
//! - Pool kết nối
//! - Cache với TTL
//! - Time utilities
//! - Storage backend (Sled)

use kernel::{
    storage::{Storage, entity::{Entity, Query, Key}, pool::Pool, cache::Cache, time::now},
    Sled, Id
};
use serde::{Serialize, Deserialize};
use std::time::Duration;

/// Entity mẫu: Task
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Task {
    id: Id,
    title: String,
    done: bool,
    created: u128,
}

/// Summary cho Task
#[derive(Serialize, Deserialize, Debug, Clone)]
struct TaskSummary {
    id: Id,
    title: String,
    done: bool,
}

impl Entity for Task {
    const NAME: &'static str = "tasks";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = TaskSummary;
    
    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn index(&self) -> Self::Index {
        // Tạo index với done flag và timestamp đảo ngược
        Key::reserve(33)
            .flag(self.done)
            .time(self.created)
            .id(self.id).clone()
            .build()
    }
    
    fn summary(&self) -> Self::Summary {
        TaskSummary {
            id: self.id,
            title: self.title.clone(),
            done: self.done,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo storage
    let storage = Sled::new("./storage_example")?;
    
    // Tạo task mẫu
    let task = Task {
        id: Id::new_v4(),
        title: "Hoàn thành storage example".to_string(),
        done: false,
        created: now(),
    };
    
    println!("=== Storage Example ===");
    println!("Task: {:?}", task);
    
    // 1. Insert task
    println!("\n1. Inserting task...");
    storage.insert(task.clone()).await?;
    println!("✓ Task inserted successfully");
    
    // 2. Fetch task
    println!("\n2. Fetching task...");
    let fetched = storage.fetch::<Task>(task.id).await?;
    match fetched {
        Some(t) => println!("✓ Task fetched: {:?}", t),
        None => println!("✗ Task not found"),
    }
    
    // 3. Update task
    println!("\n3. Updating task...");
    let updated = storage.update::<Task, _>(task.id, |mut t| {
        t.done = true;
        t.title = "Storage example completed!".to_string();
        t
    }).await?;
    println!("✓ Task updated: {:?}", updated);
    
    // 4. Query tasks
    println!("\n4. Querying tasks...");
    let query = Query::default();
    let summaries: Vec<_> = storage.query::<Task>(query).await?
        .collect::<Result<Vec<_>, _>>()?;
    println!("✓ Found {} tasks", summaries.len());
    for summary in summaries {
        println!("  - {}: {}", summary.id, summary.title);
    }
    
    // 5. Pool example
    println!("\n5. Pool example...");
    let pool = Pool::new(5, || Ok("connection".to_string()))?;
    let conn = pool.get().await?;
    println!("✓ Got connection from pool: {}", conn);
    println!("✓ Available connections: {}", pool.free());
    
    // 6. Cache example
    println!("\n6. Cache example...");
    let cache = Cache::new(Duration::from_secs(10));
    
    // Set cache
    cache.set("key1".to_string(), "value1".to_string()).await;
    println!("✓ Set cache key1");
    
    // Get cache
    if let Some(value) = cache.get(&"key1".to_string()).await {
        println!("✓ Got from cache: {}", value);
    } else {
        println!("✗ Cache miss for key1");
    }
    
    // 7. Time utilities
    println!("\n7. Time utilities...");
    let current_time = now();
    println!("✓ Current timestamp: {}", current_time);
    
    // 8. Delete task
    println!("\n8. Deleting task...");
    let deleted = storage.delete::<Task>(task.id).await?;
    println!("✓ Task deleted: {:?}", deleted);
    
    println!("\n=== Example completed successfully! ===");
    Ok(())
} 