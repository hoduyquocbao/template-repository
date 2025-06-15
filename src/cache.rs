//! Quản lý cache cho storage backend.
//!
//! Module này cung cấp một cache thread-safe và hiệu quả
//! cho việc lưu trữ tạm thời các thực thể thường xuyên truy cập.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use std::time::{Duration, Instant};
use std::hash::Hash;

/// Cache entry với thời gian hết hạn
struct Entry<T> {
    /// Dữ liệu được cache
    data: T,
    /// Thời điểm hết hạn
    exp: Instant,
}

#[derive(Clone)]
pub struct Cache<K, V> 
where 
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Map lưu trữ các entry
    map: Arc<RwLock<HashMap<K, Entry<V>>>>,
    /// Thời gian sống mặc định
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where 
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Tạo cache mới với TTL
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
            ttl,
        }
    }
    
    /// Lưu dữ liệu vào cache
    pub async fn set(&self, key: K, data: V) {
        let exp = Instant::now() + self.ttl;
        let entry = Entry { data, exp };
        self.map.write().await.insert(key, entry);
    }
    
    /// Lấy dữ liệu từ cache
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut map = self.map.write().await;
        if let Some(entry) = map.get(key) {
            if entry.exp > Instant::now() {
                return Some(entry.data.clone());
            }
            map.remove(key);
        }
        None
    }
    
    /// Xóa dữ liệu khỏi cache
    pub async fn del(&self, key: &K) {
        self.map.write().await.remove(key);
    }
    
    /// Dọn dẹp các entry đã hết hạn
    pub async fn clean(&self) {
        let now = Instant::now();
        self.map.write().await.retain(|_, entry| entry.exp > now);
    }
} 