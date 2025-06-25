//! Thu thập và theo dõi metrics cho storage backend.
//!
//! Module này cung cấp các công cụ để đo lường và theo dõi
//! hiệu suất của các thao tác storage.

// ---
// Import các thư viện cần thiết cho việc đo lường, đồng bộ hóa và lưu trữ trạng thái metric
use std::sync::atomic::{AtomicU64, Ordering}; // AtomicU64: Đếm an toàn đồng thời, Ordering: Kiểm soát thứ tự bộ nhớ
use std::collections::HashMap; // HashMap: Lưu trữ các metric theo tên thao tác
use tokio::sync::RwLock; // RwLock: Cho phép nhiều luồng đọc/ghi metrics đồng thời
use std::time::Instant; // Instant: Đo thời gian thực thi thao tác
use std::sync::Arc; // Arc: Chia sẻ ownership an toàn giữa các thread

/// Metric cho một loại thao tác
/// Mục đích: Lưu trữ số liệu thống kê cho từng loại thao tác (ví dụ: insert, fetch, update)
#[derive(Clone)]
pub struct Metric {
    /// Tổng thời gian thực thi (nano giây)
    /// Thành tựu: Cho phép tính thời gian trung bình mỗi thao tác
    time: Arc<AtomicU64>,
    /// Số lần thực thi thành công
    /// Thành tựu: Đo lường tần suất thành công
    count: Arc<AtomicU64>,
    /// Số lần thực thi thất bại
    /// Thành tựu: Đo lường tần suất lỗi
    fail: Arc<AtomicU64>,
}

/// Registry quản lý tất cả metrics
/// Mục đích: Gom nhóm và quản lý nhiều metric theo tên thao tác
#[derive(Clone)]
pub struct Registry {
    /// Map lưu trữ các metric, key là tên thao tác
    /// Thành tựu: Cho phép truy xuất metric theo tên thao tác một cách hiệu quả
    map: Arc<RwLock<HashMap<String, Metric>>>,
}

impl Metric {
    /// Tạo metric mới với các bộ đếm khởi tạo về 0
    /// Mục đích: Đảm bảo mọi metric bắt đầu từ trạng thái sạch
    pub fn new() -> Self {
        Self {
            time: Arc::new(AtomicU64::new(0)), // Thời gian tích lũy = 0
            count: Arc::new(AtomicU64::new(0)), // Số lần thành công = 0
            fail: Arc::new(AtomicU64::new(0)), // Số lần thất bại = 0
        }
    }
    
    /// Ghi lại thời gian thực thi và trạng thái thành công/thất bại
    /// Mục đích: Cập nhật số liệu cho mỗi lần thao tác được thực hiện
    /// Thuật toán: Tính thời gian đã trôi qua, tăng bộ đếm tương ứng
    pub fn record(&self, start: Instant, failed: bool) {
        let elapsed = start.elapsed().as_nanos() as u64; // Đo thời gian thực thi (nano giây)
        self.time.fetch_add(elapsed, Ordering::Relaxed); // Cộng dồn thời gian
        if failed {
            self.fail.fetch_add(1, Ordering::Relaxed); // Tăng số lần thất bại
        } else {
            self.count.fetch_add(1, Ordering::Relaxed); // Tăng số lần thành công
        }
    }
    
    /// Lấy thống kê dạng chuỗi mô tả
    /// Mục đích: Trả về tổng số lần, số lần thành công/thất bại, thời gian trung bình
    /// Thành tựu: Hỗ trợ quan sát hiệu năng và độ tin cậy
    pub fn stats(&self) -> String {
        let time = self.time.load(Ordering::Relaxed); // Tổng thời gian
        let count = self.count.load(Ordering::Relaxed); // Số lần thành công
        let fail = self.fail.load(Ordering::Relaxed); // Số lần thất bại
        let total = count + fail; // Tổng số lần thực thi
        
        if total == 0 {
            return "Chưa có dữ liệu".to_string(); // Không có dữ liệu để thống kê
        }
        
        let avg = if count > 0 { time / count } else { 0 }; // Thời gian trung bình mỗi lần thành công
        format!(
            "Tổng: {} lần ({} thành công, {} thất bại), Thời gian trung bình: {}ns",
            total, count, fail, avg
        )
    }
    
    /// Lấy tỷ lệ lỗi (fail/success)
    /// Mục đích: Đánh giá độ tin cậy của thao tác
    /// Thành tựu: Cho phép phát hiện thao tác có tỷ lệ lỗi cao
    pub fn rate(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed); // Số lần thành công
        if count == 0 {
            return 0.0; // Tránh chia cho 0
        }
        self.fail.load(Ordering::Relaxed) as f64 / count as f64 // Tỷ lệ lỗi
    }
}

impl Registry { 
    /// Tạo registry mới, khởi tạo map rỗng
    /// Mục đích: Quản lý tập hợp các metric cho toàn hệ thống
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())), // Map rỗng, thread-safe
        }
    }
    
    /// Lấy metric cho một thao tác, tạo mới nếu chưa có
    /// Mục đích: Đảm bảo mọi thao tác đều có metric riêng biệt
    /// Thuật toán: Sử dụng entry API để lấy hoặc chèn metric mới
    pub async fn get(&self, name: &str) -> Metric {
        let mut map = self.map.write().await; // Ghi lock để có thể thêm mới
        map.entry(name.to_string())
            .or_insert_with(Metric::new)
            .clone() // Trả về bản sao để dùng ngoài lock
    }
    
    /// Lấy thống kê cho tất cả metrics dưới dạng chuỗi
    /// Mục đích: Tổng hợp toàn bộ số liệu cho các thao tác
    /// Thành tựu: Hỗ trợ giám sát tổng thể hệ thống
    pub async fn stats(&self) -> String {
        let map = self.map.read().await; // Đọc lock để duyệt map
        let mut stats = Vec::new(); // Gom các chuỗi thống kê
        for (name, metric) in map.iter() {
            stats.push(format!("{}: {}", name, metric.stats())); // Thêm thống kê từng metric
        }
        stats.join("\n") // Ghép thành một chuỗi duy nhất
    }
} 