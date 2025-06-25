//! Quản lý pool kết nối cho storage backend.
//!
//! Module này cung cấp một pool kết nối thread-safe và hiệu quả
//! cho việc tái sử dụng các kết nối database.

// ---
// Import các thư viện cần thiết cho pool: chia sẻ ownership, đồng bộ hóa, và lỗi hệ thống
use std::sync::Arc; // Arc: Chia sẻ ownership danh sách kết nối giữa các thread
use tokio::sync::Semaphore; // Semaphore: Kiểm soát số lượng kết nối đồng thời
use crate::Error; // Error: Enum lỗi chuẩn hóa của hệ thống

/// Pool quản lý các kết nối database
/// Mục đích: Tối ưu hóa việc tái sử dụng kết nối, giảm chi phí khởi tạo mới
#[derive(Clone)]
pub struct Pool<T> {
    /// Danh sách các kết nối có sẵn (immutable, chia sẻ qua Arc)
    conn: Arc<Vec<T>>,
    /// Semaphore để kiểm soát số lượng kết nối đồng thời
    sem: Arc<Semaphore>,
}

impl<T: Clone + Send + Sync + 'static> Pool<T> {
    /// Tạo pool mới với số lượng kết nối cố định
    /// Mục đích: Khởi tạo trước các kết nối, đảm bảo luôn sẵn sàng phục vụ
    /// Thuật toán: Gọi hàm init nhiều lần, lưu vào Vec, bọc trong Arc
    pub fn new(size: usize, init: impl Fn() -> Result<T, Error>) -> Result<Self, Error> {
        let mut conn = Vec::with_capacity(size); // Dự phòng bộ nhớ cho Vec
        for _ in 0..size {
            conn.push(init()?); // Khởi tạo từng kết nối, có thể trả về lỗi
        }
        
        Ok(Self {
            conn: Arc::new(conn), // Chia sẻ danh sách kết nối qua Arc
            sem: Arc::new(Semaphore::new(size)), // Semaphore với số lượng permit = size
        })
    }
    
    /// Lấy một kết nối từ pool (bất đồng bộ)
    /// Mục đích: Đảm bảo không vượt quá số lượng kết nối tối đa
    /// Thuật toán: acquire semaphore, trả về bản sao kết nối đầu tiên (demo, có thể mở rộng round-robin)
    pub async fn get(&self) -> Result<T, Error> {
        let _permit = self.sem.acquire().await.map_err(|_| Error::Input)?; // Chặn nếu hết permit
        Ok(self.conn[0].clone()) // Trả về bản sao kết nối (có thể cải tiến chọn kết nối khác)
    }
    
    /// Trả về số lượng kết nối có sẵn (permit chưa dùng)
    /// Mục đích: Hỗ trợ giám sát, kiểm tra trạng thái pool
    pub fn free(&self) -> usize {
        self.sem.available_permits()
    }
} 