//! Quản lý cache cho storage backend.
//!
//! Module này cung cấp một cache thread-safe và hiệu quả
//! cho việc lưu trữ tạm thời các thực thể thường xuyên truy cập.

// ---
// Import các thư viện cần thiết cho cache: lưu trữ, đồng bộ hóa, thời gian, và hash
use std::collections::HashMap; // HashMap: Lưu trữ các entry cache theo key
use std::sync::Arc; // Arc: Chia sẻ ownership map giữa các thread
use tokio::sync::RwLock; // RwLock: Đảm bảo thread-safe cho map
use std::time::{Duration, Instant}; // Duration, Instant: Quản lý TTL và thời điểm hết hạn
use std::hash::Hash; // Hash: Đảm bảo key có thể dùng cho HashMap

/// Cache entry với thời gian hết hạn
/// Mục đích: Lưu trữ dữ liệu và thời điểm hết hạn cho từng entry
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
    /// Map lưu trữ các entry (key -> Entry)
    /// Thành tựu: Cho phép truy xuất, cập nhật, xóa entry hiệu quả và thread-safe
    map: Arc<RwLock<HashMap<K, Entry<V>>>>,
    /// Thời gian sống mặc định (Time-To-Live)
    /// Mục đích: Xác định thời gian dữ liệu tồn tại trong cache
    ttl: Duration,
}

impl<K, V> Cache<K, V>
where 
    K: Hash + Eq + Clone + Send + Sync + 'static,
    V: Clone + Send + Sync + 'static,
{
    /// Tạo cache mới với TTL
    /// Mục đích: Khởi tạo cache rỗng với thời gian sống mặc định cho mỗi entry
    pub fn new(ttl: Duration) -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())), // Map rỗng, thread-safe
            ttl,
        }
    }
    
    /// Lưu dữ liệu vào cache với key và TTL mặc định
    /// Thuật toán: Ghi đè entry cũ nếu key đã tồn tại, cập nhật thời điểm hết hạn mới
    pub async fn set(&self, key: K, data: V) {
        let exp = Instant::now() + self.ttl; // Tính thời điểm hết hạn
        let entry = Entry { data, exp };
        self.map.write().await.insert(key, entry); // Ghi entry vào map
    }
    
    /// Lấy dữ liệu từ cache nếu chưa hết hạn
    /// Thuật toán: Nếu entry hết hạn thì xóa khỏi cache, trả về None
    pub async fn get(&self, key: &K) -> Option<V> {
        let mut map = self.map.write().await; // Ghi lock để có thể xóa nếu hết hạn
        if let Some(entry) = map.get(key) {
            if entry.exp > Instant::now() {
                return Some(entry.data.clone()); // Trả về bản sao dữ liệu nếu còn hạn
            }
            map.remove(key); // Xóa entry hết hạn
        }
        None
    }
    
    /// Xóa dữ liệu khỏi cache theo key
    /// Mục đích: Cho phép chủ động loại bỏ entry khỏi cache
    pub async fn del(&self, key: &K) {
        self.map.write().await.remove(key);
    }
    
    /// Dọn dẹp các entry đã hết hạn khỏi cache
    /// Thuật toán: Duyệt toàn bộ map, chỉ giữ lại các entry còn hạn
    pub async fn clean(&self) {
        let now = Instant::now();
        self.map.write().await.retain(|_, entry| entry.exp > now);
    }
} 