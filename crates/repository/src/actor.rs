//! Actor pattern cho Sled: tách thread lưu trữ riêng biệt, giao tiếp qua channel.

use std::thread;

use crate::error::Error;
use crate::sled::Inner;
use tokio::sync::{mpsc, oneshot};
use async_trait::async_trait;

/// Enum đại diện cho các message gửi tới actor lưu trữ
pub enum Message {
    Insert {
        key: Vec<u8>,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<(), Error>>,
    },
    Fetch {
        key: Vec<u8>,
        respond: oneshot::Sender<Result<Option<Vec<u8>>, Error>>,
    },
    Update {
        key: Vec<u8>,
        value: Vec<u8>,
        respond: oneshot::Sender<Result<Vec<u8>, Error>>,
    },
    Delete {
        key: Vec<u8>,
        respond: oneshot::Sender<Result<Vec<u8>, Error>>,
    },
    Query {
        respond: oneshot::Sender<Result<Vec<Vec<u8>>, Error>>,
    },
    Mass {
        entries: Vec<(Vec<u8>, Vec<u8>)>,
        respond: oneshot::Sender<Result<(), Error>>,
    },
    Keys {
        respond: oneshot::Sender<Result<Vec<Vec<u8>>, Error>>,
    },
}

/// Actor lưu trữ: chạy thread riêng, nhận message qua channel
pub struct Actor {
    sender: mpsc::Sender<Message>,
}

impl Actor {
    pub(crate) fn new(inner: Inner) -> Self {
        let (tx, mut rx) = mpsc::channel::<Message>(128);
        let metric = inner.metric.clone();
        thread::spawn(move || {
            while let Some(msg) = rx.blocking_recv() {
                match msg {
                    Message::Insert { key, value, respond } => {
                        let res = inner.db.insert(&key[..], &value[..]).map(|_| ()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "insert" và kết quả của thao tác
                        metric.record("insert", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Fetch { key, respond } => {
                        let res = inner.db.get(&key[..]).map(|opt| opt.map(|v| v.to_vec())).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "fetch"
                        metric.record("fetch", res.is_err());

                        let _ = respond.send(res);
                    }
                    Message::Update { key, value, respond } => {
                        let res = inner.db.insert(&key[..], &value[..]).map(|_| value.clone()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "update"
                        metric.record("update", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Delete { key, respond } => {
                        let res = inner.db.remove(&key[..]).map(|opt| opt.map(|v| v.to_vec()).unwrap_or_default()).map_err(Error::Store);
                        
                        // Ghi lại metric với tên "delete"
                        metric.record("delete", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Query { respond } => {
                        let mut result = Vec::new();
                        let mut iter = inner.db.iter();
                        let mut error = None;
                        for kv in &mut iter {
                            match kv {
                                Ok((_, v)) => result.push(v.to_vec()),
                                Err(e) => { error = Some(e); break; }
                            }
                        }
                        let res = if let Some(e) = error {
                            Err(Error::Store(e))
                        } else {
                            Ok(result)
                        };
                        
                        // Ghi lại metric với tên "query"
                        metric.record("query", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Mass { entries, respond } => {
                        let mut ok = true;
                        for (k, v) in entries.iter() {
                            if inner.db.insert(&k[..], &v[..]).is_err() {
                                ok = false;
                                break;
                            }
                        }
                        let res = if ok { Ok(()) } else { Err(Error::Aborted) };
                        
                        // Ghi lại metric với tên "mass"
                        metric.record("mass", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                    Message::Keys { respond } => {
                        let mut result = Vec::new();
                        let mut iter = inner.db.iter();
                        let mut error = None;
                        for kv in &mut iter {
                            match kv {
                                Ok((k, _)) => result.push(k.to_vec()),
                                Err(e) => { error = Some(e); break; }
                            }
                        }
                        let res = if let Some(e) = error {
                            Err(Error::Store(e))
                        } else {
                            Ok(result)
                        };
                        
                        // Ghi lại metric với tên "keys"
                        metric.record("keys", res.is_err());
                        
                        let _ = respond.send(res);
                    }
                }
            }
        });
        Self { sender: tx }
    }
    pub fn handle(&self) -> Handle {
        Handle { sender: self.sender.clone() }
    }
}

/// Handle gửi request tới actor, cloneable
#[derive(Clone)]
pub struct Handle {
    sender: mpsc::Sender<Message>,
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
        let metric = inner.metric.clone();
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
        let stats = metric.stats().await;
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
        let metric = inner.metric.clone();
        let actor = Actor::new(inner);
        let handle = actor.handle();

        // Test với key không tồn tại để tạo lỗi
        let missing = b"non_existent".to_vec();
        
        // Fetch key không tồn tại
        let result = handle.fetch(missing.clone()).await;
        assert!(result.is_ok()); // Fetch trả về None, không phải lỗi
        
        // Delete key không tồn tại
        let result = handle.delete(missing.clone()).await;
        assert!(result.is_ok()); // Delete trả về empty vec, không phải lỗi

        // Đợi một chút để đảm bảo metrics được ghi
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Kiểm tra metrics
        let stats = metric.stats().await;
        println!("Error handling metrics: {}", stats);
        
        assert!(stats.contains("fetch"));
        assert!(stats.contains("delete"));
    }

    #[tokio::test]
    async fn bulk() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let inner = Inner::new(path).unwrap();
        let metric = inner.metric.clone();
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
        let stats = metric.stats().await;
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
        let metric = inner.metric.clone();
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
        let stats = metric.stats().await;
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