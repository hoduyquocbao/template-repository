//! Thử nghiệm toàn diện cho framework export storage

use super::*;
use crate::storage::{sled::Sled, Storage, entity::{Entity, Query}};
use crate::Id;
use serde::{Serialize, Deserialize};
use tempfile::tempdir;
use tokio::time::{sleep, Duration};

/// Thực thể test cho export
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct Item {
    id: Id,
    name: String,
    value: u32,
    timestamp: u128,
}

#[derive(Serialize, Deserialize)]
struct Brief {
    id: Id,
    name: String,
}

impl Entity for Item {
    const NAME: &'static str = "test_entities";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = Brief;
    
    fn key(&self) -> Self::Key { 
        self.id 
    }
    
    fn index(&self) -> Self::Index { 
        let mut key = Vec::new();
        key.extend_from_slice(&self.timestamp.to_be_bytes());
        key.extend_from_slice(self.id.as_bytes());
        key
    }
    
    fn summary(&self) -> Self::Summary {
        Brief { 
            id: self.id, 
            name: self.name.clone() 
        }
    }
}

/// Helper function tạo test storage
fn store() -> Sled {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    Sled::new(path).unwrap()
}

/// Helper function tạo test data
fn items(count: usize) -> Vec<Item> {
    (0..count).map(|i| Item {
        id: Id::new_v4(),
        name: format!("test_{}", i),
        value: i as u32,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis(),
    }).collect()
}

#[tokio::test]
async fn builder() {
    let storage = store();
    
    let export = Builder::new()
        .config(Config { batch: 500, timeout: 60, compress: true })
        .format(Format::Json)
        .build(storage);
        
    assert_eq!(export.config.batch, 500);
    assert_eq!(export.config.timeout, 60);
    assert!(export.config.compress);
}

#[tokio::test]
async fn stream() {
    let data = b"test data for streaming".to_vec();
    let mut stream = Stream::new(data);
    
    // Test read chunks
    let chunk1 = stream.read(4);
    assert_eq!(chunk1, Some(b"test".as_ref()));
    
    let chunk2 = stream.read(5);
    assert_eq!(chunk2, Some(b" data".as_ref()));
    
    // Test done status
    assert!(!stream.done());
    
    // Test reset
    stream.reset();
    assert_eq!(stream.pos, 0);
    
    // Test read after reset
    let chunk3 = stream.read(4);
    assert_eq!(chunk3, Some(b"test".as_ref()));
}

#[tokio::test]
async fn formats() {
    let storage = store();
    let export = Export::default(storage);
    
    // Test JSON format
    let json = export.export(Format::Json).await.unwrap();
    assert!(!json.done());
    
    // Test CSV format
    let csv = export.export(Format::Csv).await.unwrap();
    assert!(!csv.done());
    
    // Test Binary format
    let binary = export.export(Format::Binary).await.unwrap();
    assert!(!binary.done());
    
    // Test Custom format
    let config = Config { batch: 100, timeout: 10, compress: false };
    let custom = export.export(Format::Custom(config)).await.unwrap();
    assert!(!custom.done());
}

#[tokio::test]
async fn filter() {
    let storage = store();
    let export = Export::default(storage);
    
    let filter = Filter {
        prefix: b"test_".to_vec(),
        limit: Some(50),
        offset: Some(0),
    };
    
    let stream = export.partial(filter, Format::Json).await.unwrap();
    assert!(!stream.done());
}

#[tokio::test]
async fn data() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert test data
    let items = items(10);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Test export with data
    let stream = export.export(Format::Json).await.unwrap();
    assert!(!stream.done());
    
    // Verify stream contains data
    let mut data = Vec::new();
    while !stream.done() {
        if let Some(chunk) = stream.read(1024) {
            data.extend_from_slice(chunk);
        }
    }
    
    assert!(!data.is_empty());
}

#[tokio::test]
async fn error() {
    let storage = store();
    let export = Export::default(storage);
    
    // Test with invalid filter
    let wrong = Filter {
        prefix: Vec::new(),
        limit: Some(0), // Invalid limit
        offset: Some(0),
    };
    
    // Should handle gracefully
    let result = export.partial(wrong, Format::Json).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn memory() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert large amount of data
    let items = items(10000);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Test memory usage during export
    let begin = std::alloc::System::allocated();
    
    let stream = export.export(Format::Json).await.unwrap();
    
    // Process stream in chunks to avoid loading everything into memory
    let mut size = 0;
    while !stream.done() {
        if let Some(chunk) = stream.read(1024) {
            size += chunk.len();
        }
    }
    
    let end = std::alloc::System::allocated();
    let diff = end - begin;
    
    // Memory usage should be reasonable (less than 10MB)
    assert!(diff < 10 * 1024 * 1024);
    assert!(size > 0);
}

#[tokio::test]
async fn config() {
    let storage = store();
    
    // Test with various config combinations
    let configs = vec![
        Config { batch: 1, timeout: 1, compress: false },
        Config { batch: 1000, timeout: 60, compress: true },
        Config { batch: 10000, timeout: 300, compress: false },
    ];
    
    for config in configs {
        let export = Export::new(storage.clone(), config);
        let stream = export.export(Format::Json).await.unwrap();
        assert!(!stream.done());
    }
}

#[tokio::test]
async fn format() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert test data
    let items = items(10);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Test that different formats produce different outputs
    let json = export.export(Format::Json).await.unwrap();
    let csv = export.export(Format::Csv).await.unwrap();
    let binary = export.export(Format::Binary).await.unwrap();
    
    // Collect data from each stream
    let mut j = Vec::new();
    let mut c = Vec::new();
    let mut b = Vec::new();
    
    while !json.done() {
        if let Some(chunk) = json.read(1024) {
            j.extend_from_slice(chunk);
        }
    }
    
    while !csv.done() {
        if let Some(chunk) = csv.read(1024) {
            c.extend_from_slice(chunk);
        }
    }
    
    while !binary.done() {
        if let Some(chunk) = binary.read(1024) {
            b.extend_from_slice(chunk);
        }
    }
    
    // Verify different formats produce different outputs
    assert_ne!(j, c);
    assert_ne!(j, b);
    assert_ne!(c, b);
}

#[tokio::test]
async fn again() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert test data
    let items = items(100);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Test export again after interruption
    let stream1 = export.export(Format::Json).await.unwrap();
    
    // Simulate interruption
    drop(stream1);
    
    // Should be able to export again
    let stream2 = export.export(Format::Json).await.unwrap();
    assert!(!stream2.done());
}

#[tokio::test]
async fn full() {
    let storage = store();
    let export = Export::default(storage);
    
    // Full integration test
    let items = items(500);
    
    // Insert data
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Verify data exists
    for item in &items {
        let fetched = storage.fetch::<Item>(item.id).await.unwrap();
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap(), *item);
    }
    
    // Export data
    let stream = export.export(Format::Json).await.unwrap();
    assert!(!stream.done());
    
    // Process export
    let mut output = Vec::new();
    while !stream.done() {
        if let Some(chunk) = stream.read(1024) {
            output.extend_from_slice(chunk);
        }
    }
    
    // Verify export contains data
    assert!(!output.is_empty());
    
    // Verify export is valid JSON
    let text = String::from_utf8(output).unwrap();
    assert!(text.contains("test_"));
}

#[tokio::test]
async fn speed() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert large amount of test data
    let items = items(1000);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Measure export speed
    let start = std::time::Instant::now();
    let stream = export.export(Format::Json).await.unwrap();
    let time = start.elapsed();
    
    // Verify speed is reasonable (should complete within 1 second)
    assert!(time < Duration::from_secs(1));
    assert!(!stream.done());
}

#[tokio::test]
async fn group() {
    let storage = store();
    let export = Export::default(storage);
    
    // Insert test data
    let items = items(100);
    for item in &items {
        storage.insert(item.clone()).await.unwrap();
    }
    
    // Test concurrent exports
    let mut handles = Vec::new();
    
    for _ in 0..5 {
        let clone = Export::default(storage.clone());
        let handle = tokio::spawn(async move {
            let stream = clone.export(Format::Json).await.unwrap();
            assert!(!stream.done());
        });
        handles.push(handle);
    }
    
    // Wait for all exports to complete
    for handle in handles {
        handle.await.unwrap();
    }
} 