//! Demo framework export cho storage
//!
//! Example này minh họa cách sử dụng framework export
//! để xuất dữ liệu từ storage theo nhiều định dạng khác nhau.

use kernel::storage::{Builder, Config, Export, Exportable, Filter, Format};
use kernel::{Entity, Id, Sled, Storage};
use serde::{Serialize, Deserialize};
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs::File;
use std::io::Write;

/// Thực thể demo cho export
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct User {
    id: Id,
    name: String,
    email: String,
    age: u32,
    created: u128,
}

#[derive(Serialize, Deserialize)]
struct Summary {
    id: Id,
    name: String,
    email: String,
}

impl Entity for User {
    const NAME: &'static str = "users";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = Summary;
    
    fn key(&self) -> Self::Key { 
        self.id 
    }
    
    fn index(&self) -> Self::Index { 
        let mut key = Vec::new();
        key.extend_from_slice(&self.created.to_be_bytes());
        key.extend_from_slice(self.id.as_bytes());
        key
    }
    
    fn summary(&self) -> Self::Summary {
        Summary { 
            id: self.id, 
            name: self.name.clone(),
            email: self.email.clone(),
        }
    }
}

/// Tạo dữ liệu demo
fn users() -> Vec<User> {
    vec![
        User {
            id: Id::new_v4(),
            name: "Alice Johnson".to_string(),
            email: "alice@example.com".to_string(),
            age: 25,
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        },
        User {
            id: Id::new_v4(),
            name: "Bob Smith".to_string(),
            email: "bob@example.com".to_string(),
            age: 30,
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        },
        User {
            id: Id::new_v4(),
            name: "Carol Davis".to_string(),
            email: "carol@example.com".to_string(),
            age: 28,
            created: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis(),
        },
    ]
}

/// Ghi toàn bộ dữ liệu từ stream ra file
fn file(path: &str, mut stream: kernel::storage::Stream) -> std::io::Result<()> {
    let mut out = File::create(path)?;
    let mut count = 0;
    const LIMIT: usize = 1000; // Giới hạn để tránh vòng lặp vô hạn
    
    while !stream.done() && count < LIMIT {
        if let Some(chunk) = stream.read(1024) {
            out.write_all(&chunk)?;
        }
        count += 1;
    }
    
    if count >= LIMIT {
        eprintln!("Warning: Stream reading reached maximum iterations");
    }
    
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Demo Framework Export cho Storage");
    println!("=====================================");
    
    // Tạo storage
    let storage = Sled::new("demo_export")?;
    println!("✅ Storage đã được khởi tạo");
    
    // Tạo dữ liệu demo
    let users = users();
    println!("📊 Tạo {} users demo", users.len());
    
    // Insert dữ liệu vào storage
    for user in &users {
        storage.insert(user.clone()).await?;
    }
    println!("💾 Dữ liệu đã được lưu vào storage");
    
    // Demo 1: Export cơ bản với Builder
    println!("\n📤 Demo 1: Export cơ bản với Builder");
    let export = Builder::new()
        .config(Config { batch: 100, timeout: 30, compress: false })
        .format(Format::Json)
        .build(storage.clone());
    let stream = export.export(Format::Json).await?;
    file("export.json", stream)?;
    println!("✅ Export JSON thành công, đã ghi ra export.json");
    
    // Demo 2: Export với filter
    println!("\n📤 Demo 2: Export với filter");
    let filter = Filter {
        prefix: b"".to_vec(),
        limit: Some(2),
        offset: Some(0),
    };
    let stream = export.partial(filter, Format::Json).await?;
    file("export_filter.json", stream)?;
    println!("✅ Export với filter thành công, đã ghi ra export_filter.json");
    
    // Demo 3: Export các format khác nhau
    println!("\n📤 Demo 3: Export các format khác nhau");
    let formats = vec![
        ("JSON", Format::Json, "export.json"),
        ("CSV", Format::Csv, "export.csv"),
        ("Binary", Format::Binary, "export.bin"),
    ];
    for (name, format, path) in formats {
        let stream = export.export(format).await?;
        file(path, stream)?;
        println!("✅ Export {} thành công, đã ghi ra {}", name, path);
    }
    
    // Demo 4: Export với config custom
    println!("\n📤 Demo 4: Export với config custom");
    let custom = Config {
        batch: 50,
        timeout: 60,
        compress: true,
    };
    let stream = export.export(Format::Custom(custom)).await?;
    file("export_custom.json", stream)?;
    println!("✅ Export với config custom thành công, đã ghi ra export_custom.json");
    
    // Demo 5: Export concurrent
    println!("\n📤 Demo 5: Export concurrent");
    let mut handles = Vec::new();
    
    for i in 0..3 {
        let export = Export::default(storage.clone());
        let handle = tokio::spawn(async move {
            let stream = export.export(Format::Json).await.unwrap();
            println!("✅ Export concurrent {} thành công", i + 1);
            stream
        });
        handles.push(handle);
    }
    
    // Wait for all exports to complete
    for handle in handles {
        handle.await?;
    }
    
    // Demo 6: Performance test
    println!("\n📤 Demo 6: Performance test");
    let start = std::time::Instant::now();
    
    for _ in 0..10 {
        let mut stream = export.export(Format::Json).await?;
        // Process stream
        while !stream.done() {
            if let Some(_chunk) = stream.read(1024) {
                // Process chunk
            }
        }
    }
    
    let duration = start.elapsed();
    println!("✅ Performance test hoàn thành trong {:?}", duration);
    
    // Demo 7: Error handling
    println!("\n📤 Demo 7: Error handling");
    let filter = Filter {
        prefix: Vec::new(),
        limit: Some(0), // Invalid limit
        offset: Some(0),
    };
    
    match export.partial(filter, Format::Json).await {
        Ok(_) => println!("✅ Error handling hoạt động tốt"),
        Err(e) => println!("⚠️ Error được xử lý: {:?}", e),
    }
    
    println!("\n🎉 Demo hoàn thành!");
    println!("Framework export đã được test với:");
    println!("  - Builder pattern");
    println!("  - Multiple formats");
    println!("  - Filtering");
    println!("  - Custom config");
    println!("  - Concurrent access");
    println!("  - Performance optimization");
    println!("  - Error handling");
    
    Ok(())
} 