//! Example: Bulk Storage Operations và Advanced Querying
//!
//! Minh họa các tính năng nâng cao của storage:
//! - Bulk insert operations
//! - Advanced querying với prefix và pagination
//! - Performance monitoring
//! - Error handling

use kernel::{
    storage::{Storage, entity::{Entity, Query, Key}, time::now},
    Sled, Id
};
use serde::{Serialize, Deserialize};
use std::time::Instant;

/// Entity mẫu: User
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    id: Id,
    name: String,
    email: String,
    active: bool,
    created: u128,
    last_login: u128,
}

/// Summary cho User
#[derive(Serialize, Deserialize, Debug, Clone)]
struct UserSummary {
    id: Id,
    name: String,
    email: String,
    active: bool,
}

impl Entity for User {
    const NAME: &'static str = "users";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = UserSummary;
    
    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn index(&self) -> Self::Index {
        // Index: active_flag + created_timestamp_reversed + id
        Key::reserve(33)
            .flag(self.active)
            .time(self.created)
            .id(self.id).clone()
            .build()
    }
    
    fn summary(&self) -> Self::Summary {
        UserSummary {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
            active: self.active,
        }
    }
}

/// Tạo dữ liệu mẫu
fn generate_sample_users(count: usize) -> Vec<User> {
    let mut users = Vec::with_capacity(count);
    let current_time = now();
    
    for i in 0..count {
        let user = User {
            id: Id::new_v4(),
            name: format!("User {}", i + 1),
            email: format!("user{}@example.com", i + 1),
            active: i % 3 != 0, // 2/3 users active
            created: current_time - (i as u128 * 1000), // Staggered creation times
            last_login: current_time - (i as u128 * 500),
        };
        users.push(user);
    }
    users
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Khởi tạo storage
    let storage = Sled::new("./bulk_storage_example")?;
    
    println!("=== Bulk Storage Operations Example ===");
    
    // 1. Bulk Insert
    println!("\n1. Bulk Insert Operations...");
    let start_time = Instant::now();
    
    let users = generate_sample_users(100);
    println!("✓ Generated {} sample users", users.len());
    
    // Bulk insert
    let bulk_start = Instant::now();
    storage.mass(Box::new(users.clone().into_iter())).await?;
    let bulk_duration = bulk_start.elapsed();
    println!("✓ Bulk insert completed in {:?}", bulk_duration);
    
    // 2. Query Active Users
    println!("\n2. Querying Active Users...");
    let mut active_key = Key::reserve(1);
    active_key.flag(true);
    let active_query = Query {
        prefix: active_key.clone().build(), // Active users only
        after: None,
        limit: 10,
    };
    
    let active_users: Vec<_> = storage.query::<User>(active_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    println!("✓ Found {} active users (first 10)", active_users.len());
    
    for user in &active_users {
        println!("  - {} ({})", user.name, user.email);
    }
    
    // 3. Query Inactive Users
    println!("\n3. Querying Inactive Users...");
    let mut inactive_key = Key::reserve(1);
    inactive_key.flag(false);
    let inactive_query = Query {
        prefix: inactive_key.clone().build(), // Inactive users only
        after: None,
        limit: 5,
    };
    
    let inactive_users: Vec<_> = storage.query::<User>(inactive_query).await?
        .collect::<Result<Vec<_>, _>>()?;
    println!("✓ Found {} inactive users (first 5)", inactive_users.len());
    
    for user in &inactive_users {
        println!("  - {} ({})", user.name, user.email);
    }
    
    // 4. Pagination Example
    println!("\n4. Pagination Example...");
    let mut page = 0;
    let page_size = 5;
    let mut last_key: Option<Vec<u8>> = None;
    
    loop {
        let paginated_query = Query {
            prefix: Vec::new(), // All users
            after: last_key.clone(),
            limit: page_size,
        };
        
        let summaries: Vec<_> = storage.query::<User>(paginated_query).await?
            .collect::<Result<Vec<_>, _>>()?;
        let mut page_users = Vec::new();
        for summary in &summaries {
            if let Some(user) = storage.fetch::<User>(summary.id).await? {
                page_users.push(user);
            }
        }
        
        if page_users.is_empty() {
            break;
        }
        
        println!("  Page {}: {} users", page + 1, page_users.len());
        for user in &page_users {
            println!("    - {} ({})", user.name, user.email);
        }
        
        // Set last key for next page - dùng index của User
        if let Some(last_user) = page_users.last() {
            last_key = Some(last_user.index());
        } else {
            break;
        }
        
        page += 1;
        
        // Limit to 3 pages for demo
        if page >= 3 {
            break;
        }
    }
    
    // 5. Performance Monitoring
    println!("\n5. Performance Monitoring...");
    let total_duration = start_time.elapsed();
    println!("✓ Total operation time: {:?}", total_duration);
    println!("✓ Average time per user: {:?}", total_duration / users.len() as u32);
    
    // 6. Error Handling Example
    println!("\n6. Error Handling Example...");
    
    // Try to fetch non-existent user
    let non_existent_id = Id::new_v4();
    match storage.fetch::<User>(non_existent_id).await {
        Ok(None) => println!("✓ Correctly handled non-existent user"),
        Ok(Some(_)) => println!("✗ Unexpected: found non-existent user"),
        Err(e) => println!("✗ Error fetching non-existent user: {:?}", e),
    }
    
    // 7. Update Multiple Users
    println!("\n7. Update Multiple Users...");
    let update_count = 5;
    let mut updated_count = 0;
    
    for user in users.iter().take(update_count) {
        match storage.update::<User, _>(user.id, |mut u| {
            u.name = format!("Updated {}", u.name);
            u.last_login = now();
            u
        }).await {
            Ok(_) => {
                updated_count += 1;
                println!("  ✓ Updated user: {}", user.name);
            }
            Err(e) => println!("  ✗ Failed to update user {}: {:?}", user.name, e),
        }
    }
    
    println!("✓ Successfully updated {}/{} users", updated_count, update_count);
    
    // 8. Cleanup
    println!("\n8. Cleanup...");
    let mut deleted_count = 0;
    
    for user in users.iter().take(10) { // Delete first 10 for demo
        match storage.delete::<User>(user.id).await {
            Ok(_) => {
                deleted_count += 1;
                println!("  ✓ Deleted user: {}", user.name);
            }
            Err(e) => println!("  ✗ Failed to delete user {}: {:?}", user.name, e),
        }
    }
    
    println!("✓ Successfully deleted {}/10 users", deleted_count);
    
    println!("\n=== Bulk Storage Example completed successfully! ===");
    println!("✓ Total time: {:?}", start_time.elapsed());
    
    Ok(())
} 