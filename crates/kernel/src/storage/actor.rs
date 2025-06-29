//! Actor pattern cho Sled: tách thread lưu trữ riêng biệt, giao tiếp qua channel.

use std::thread;

use crate::error::Error;
use crate::storage::sled::Inner;
use tokio::sync::{mpsc, oneshot};
use async_trait::async_trait;
use crate::storage::actor::message::Message;
use crate::storage::actor::state::{Cell, State};
pub mod message;
pub mod handler;
pub mod state;

/// Actor lưu trữ: chạy thread riêng, nhận message qua channel
pub struct Actor {
    sender: mpsc::Sender<message::Message>,
    metric: crate::metric::Registry,
    state: Cell,
}

impl Actor {
    pub(crate) fn new(inner: Inner) -> Self {
        let (tx, mut rx) = mpsc::channel::<message::Message>(128);
        let metric = inner.metric.clone();
        let shared = metric.clone();
        let state = Cell::new(State::Idle);
        let cell = state.clone();
        thread::spawn(move || {
            cell.set(State::Running);
            while let Some(msg) = rx.blocking_recv() {
                handler::handle(msg, &inner, &shared);
            }
            cell.set(State::Stopped);
        });
        Self { sender: tx, metric, state }
    }
    pub fn handle(&self) -> Handle {
        Handle { sender: self.sender.clone(), metric: self.metric.clone(), state: self.state.clone() }
    }
    pub fn metrics(&self) -> crate::metric::Registry {
        self.metric.clone()
    }
    pub fn state(&self) -> State {
        self.state.get()
    }
}

/// Handle gửi request tới actor, cloneable
#[derive(Clone)]
pub struct Handle {
    sender: mpsc::Sender<message::Message>,
    metric: crate::metric::Registry,
    state: Cell,
}

impl Handle {
    pub fn metrics(&self) -> crate::metric::Registry {
        self.metric.clone()
    }
    pub fn state(&self) -> State {
        self.state.get()
    }
}

#[async_trait]
pub trait Actorable: Send + Sync + Clone + 'static {
    async fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error>;
    async fn fetch(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error>;
    async fn update(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>, Error>;
    async fn delete(&self, key: Vec<u8>) -> Result<Vec<u8>, Error>;
    async fn query(&self) -> Result<Vec<Vec<u8>>, Error>;
    async fn mass(&self, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error>;
    async fn keys(&self) -> Result<Vec<Vec<u8>>, Error>;
}

#[async_trait]
impl Actorable for Handle {
    async fn insert(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Insert { key, value, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn fetch(&self, key: Vec<u8>) -> Result<Option<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Fetch { key, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn update(&self, key: Vec<u8>, value: Vec<u8>) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Update { key, value, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn delete(&self, key: Vec<u8>) -> Result<Vec<u8>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Delete { key, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn query(&self) -> Result<Vec<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Query { respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn mass(&self, entries: Vec<(Vec<u8>, Vec<u8>)>) -> Result<(), Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Mass { entries, respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
    async fn keys(&self) -> Result<Vec<Vec<u8>>, Error> {
        let (tx, rx) = oneshot::channel();
        let msg = Message::Keys { respond: tx };
        self.sender.send(msg).await.map_err(|_| Error::Aborted)?;
        rx.await.map_err(|_| Error::Aborted)?
    }
}

// TODO: Triển khai các hàm gửi message bất đồng bộ cho Handle 

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    // Test Actor với metrics
    #[tokio::test]
    async fn metrics() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let inner = Inner::new(path).unwrap();
        let _metric = inner.metric.clone();
        let actor = Actor::new(inner);
        let handle = actor.handle();

        // Thực hiện các thao tác để trigger metrics
        let key = b"test_key".to_vec();
        let value = b"test_value".to_vec();

        // Test insert
        let result = handle.insert(key.clone(), value.clone()).await;
        assert!(result.is_ok());

        // Test fetch
        let result = handle.fetch(key.clone()).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());

        // Test update
        let updated = b"updated_value".to_vec();
        let result = handle.update(key.clone(), updated.clone()).await;
        assert!(result.is_ok());

        // Test delete
        let result = handle.delete(key.clone()).await;
        assert!(result.is_ok());

        // Đợi một chút để đảm bảo metrics được ghi
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Kiểm tra metrics
        let stats = handle.metrics().stats().await;
        println!("Metrics stats: {}", stats);
        
        assert!(stats.contains("insert"));
        assert!(stats.contains("fetch"));
        assert!(stats.contains("update"));
        assert!(stats.contains("delete"));
        assert!(stats.contains("1 thành công")); // Mỗi operation thành công
    }

    #[tokio::test]
    async fn error() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let inner = Inner::new(path).unwrap();
        let _metric = inner.metric.clone();
        let actor = Actor::new(inner);
        let handle = actor.handle();

        // Test với key không tồn tại để tạo lỗi
        let missing = b"non_existent".to_vec();
        
        // Fetch key không tồn tại
        let result = handle.insert(missing.clone(), b"value".to_vec()).await;
        assert!(result.is_ok()); // Insert trả về Ok, không phải lỗi
        
        // Delete key không tồn tại
        let result = handle.delete(missing.clone()).await;
        assert!(result.is_ok()); // Delete trả về empty vec, không phải lỗi

        // Đợi một chút để đảm bảo metrics được ghi
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Kiểm tra metrics
        let stats = handle.metrics().stats().await;
        println!("Error handling metrics: {}", stats);
        
        assert!(stats.contains("insert"));
        assert!(stats.contains("delete"));
    }

    #[tokio::test]
    async fn bulk() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let inner = Inner::new(path).unwrap();
        let _metric = inner.metric.clone();
        let actor = Actor::new(inner);
        let handle = actor.handle();

        // Test mass insert
        let entries = vec![
            (b"key1".to_vec(), b"value1".to_vec()),
            (b"key2".to_vec(), b"value2".to_vec()),
            (b"key3".to_vec(), b"value3".to_vec()),
        ];
        
        let result = handle.mass(entries).await;
        assert!(result.is_ok());

        // Test query
        let result = handle.query().await;
        assert!(result.is_ok());

        // Test keys
        let result = handle.keys().await;
        assert!(result.is_ok());

        // Đợi một chút để đảm bảo metrics được ghi
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Kiểm tra metrics
        let stats = handle.metrics().stats().await;
        println!("Bulk operations metrics: {}", stats);
        
        assert!(stats.contains("mass"));
        assert!(stats.contains("query"));
        assert!(stats.contains("keys"));
    }

    #[tokio::test]
    async fn concurrent() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let inner = Inner::new(path).unwrap();
        let _metric = inner.metric.clone();
        let actor = Actor::new(inner);
        let handle = actor.handle();

        let mut handles = vec![];

        // Tạo nhiều task đồng thời
        for i in 0..10 {
            let clone = handle.clone();
            let task = tokio::spawn(async move {
                for j in 0..10 {
                    let key = format!("key_{}_{}", i, j).into_bytes();
                    let value = format!("value_{}_{}", i, j).into_bytes();
                    // Insert
                    let _ = clone.insert(key.clone(), value.clone()).await;
                    // Fetch
                    let _ = clone.fetch(key.clone()).await;
                    // Update
                    let updated = format!("updated_{}_{}", i, j).into_bytes();
                    let _ = clone.update(key.clone(), updated).await;
                }
            });
            handles.push(task);
        }

        // Đợi tất cả task hoàn thành
        for handle in handles {
            handle.await.unwrap();
        }
        // Đợi thêm để đảm bảo atomic cập nhật xong
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // Kiểm tra metrics
        let stats = handle.metrics().stats().await;
        println!("Concurrent metrics: {}", stats);
        
        // Kiểm tra từng loại metric có tổng > 0
        assert!(stats.contains("insert: Tổng:"));
        assert!(stats.contains("fetch: Tổng:"));
        assert!(stats.contains("update: Tổng:"));
        
        // Kiểm tra có ít nhất một loại có tổng > 0
        let found = stats.lines().any(|line| {
            if let Some(idx) = line.find(": Tổng: ") {
                let rest = &line[idx+8..]; // Bỏ qua ": Tổng: "
                // Tìm số đầu tiên trong phần còn lại
                for word in rest.split_whitespace() {
                    if let Ok(count) = word.parse::<usize>() {
                        return count > 0;
                    }
                }
            }
            false
        });
        
        assert!(found, "Phải có ít nhất một loại metric có tổng > 0");
    }
} 