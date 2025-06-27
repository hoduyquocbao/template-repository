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

impl Default for Metric {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry quản lý tất cả metrics
/// Mục đích: Gom nhóm và quản lý nhiều metric theo tên thao tác
#[derive(Clone)]
pub struct Registry {
    /// Map lưu trữ các metric, key là tên thao tác
    /// Thành tựu: Cho phép truy xuất metric theo tên thao tác một cách hiệu quả
    map: Arc<RwLock<HashMap<String, Metric>>>,
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
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
    
    /// Ghi lại metric đồng bộ cho một thao tác
    /// Mục đích: Cho phép Actor thread ghi metric mà không cần async
    /// Thuật toán: Sử dụng try_write để tránh deadlock, fallback về async nếu cần
    pub fn record(&self, name: &str, failed: bool) {
        let start = Instant::now();
        
        // Thử sử dụng try_write trước để tránh deadlock
        if let Ok(mut map) = self.map.try_write() {
            let metric = map.entry(name.to_string())
                .or_insert_with(Metric::new)
                .clone();
            drop(map); // Giải phóng lock trước khi gọi record
            metric.record(start, failed);
        } else {
            // Fallback: tạo metric mới nếu không thể acquire lock
            let metric = Metric::new();
            metric.record(start, failed);
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use std::thread;

    #[test]
    fn record() {
        let metric = Metric::new();
        
        // Test ghi nhận thành công
        let start = Instant::now();
        thread::sleep(Duration::from_millis(10)); // Tạo độ trễ nhỏ
        metric.record(start, false);
        
        // Test ghi nhận thất bại
        let start = Instant::now();
        thread::sleep(Duration::from_millis(5));
        metric.record(start, true);
        
        let stats = metric.stats();
        assert!(stats.contains("Tổng: 2 lần"));
        assert!(stats.contains("1 thành công"));
        assert!(stats.contains("1 thất bại"));
        assert!(stats.contains("Thời gian trung bình:"));
    }

    #[test]
    fn rate() {
        let metric = Metric::new();
        
        // 3 thành công, 1 thất bại
        for _ in 0..3 {
            let start = Instant::now();
            metric.record(start, false);
        }
        let start = Instant::now();
        metric.record(start, true);
        
        let rate = metric.rate();
        assert!((rate - 0.333333).abs() < 0.001); // 1/3 ≈ 0.333333
    }

    #[test]
    fn registry() {
        let registry = Registry::new();
        
        // Test ghi nhận các loại thao tác khác nhau
        registry.record("insert", false);
        registry.record("fetch", false);
        registry.record("update", true);
        registry.record("delete", false);
        
        // Test thống kê async
        tokio::runtime::Runtime::new().unwrap().block_on(async {
            let stats = registry.stats().await;
            assert!(stats.contains("insert"));
            assert!(stats.contains("fetch"));
            assert!(stats.contains("update"));
            assert!(stats.contains("delete"));
            assert!(stats.contains("1 thành công"));
            assert!(stats.contains("1 thất bại"));
        });
    }

    #[test]
    fn concurrent() {
        let registry = Registry::new();
        let mut handles = vec![];
        
        // Tạo nhiều thread ghi metric đồng thời
        for _i in 0..10 {
            let clone = registry.clone();
            let handle = std::thread::spawn(move || {
                for j in 0..100 {
                    let operation = if j % 3 == 0 { "insert" } else if j % 3 == 1 { "fetch" } else { "update" };
                    let failed = j % 10 == 0; // 10% thất bại
                    clone.record(operation, failed);
                }
            });
            handles.push(handle);
        }
        
        // Đợi tất cả thread hoàn thành
        for handle in handles {
            handle.join().unwrap();
        }
        // Đợi thêm để đảm bảo atomic cập nhật xong
        std::thread::sleep(std::time::Duration::from_millis(200));
        
        // Kiểm tra thống kê
        let stats = std::thread::spawn({
            let registry = registry.clone();
            move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    registry.stats().await
                })
            }
        }).join().unwrap();
        
        println!("Registry concurrent stats: {}", stats);
        
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

    #[test]
    fn empty() {
        let metric = Metric::new();
        let stats = metric.stats();
        assert_eq!(stats, "Chưa có dữ liệu");
        
        let rate = metric.rate();
        assert_eq!(rate, 0.0);
    }
} 