//! Thu thập và theo dõi metrics cho storage backend.
//!
//! Module này cung cấp các công cụ để đo lường và theo dõi
//! hiệu suất của các thao tác storage.

use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use tokio::sync::RwLock;
use std::time::Instant;
use std::sync::Arc;

/// Metric cho một loại thao tác
#[derive(Clone)]
pub struct Metric {
    /// Tổng thời gian thực thi
    time: Arc<AtomicU64>,
    /// Số lần thực thi thành công
    count: Arc<AtomicU64>,
    /// Số lần thực thi thất bại
    fail: Arc<AtomicU64>,
}

/// Registry quản lý tất cả metrics
#[derive(Clone)]
pub struct Registry {
    /// Map lưu trữ các metric
    map: Arc<RwLock<HashMap<String, Metric>>>,
}

impl Metric {
    /// Tạo metric mới
    pub fn new() -> Self {
        Self {
            time: Arc::new(AtomicU64::new(0)),
            count: Arc::new(AtomicU64::new(0)),
            fail: Arc::new(AtomicU64::new(0)),
        }
    }
    
    /// Ghi lại thời gian thực thi
    pub fn record(&self, start: Instant, failed: bool) {
        let elapsed = start.elapsed().as_nanos() as u64;
        self.time.fetch_add(elapsed, Ordering::Relaxed);
        if failed {
            self.fail.fetch_add(1, Ordering::Relaxed);
        } else {
            self.count.fetch_add(1, Ordering::Relaxed);
        }
    }
    
    /// Lấy thống kê
    pub fn stats(&self) -> String {
        let time = self.time.load(Ordering::Relaxed);
        let count = self.count.load(Ordering::Relaxed);
        let fail = self.fail.load(Ordering::Relaxed);
        let total = count + fail;
        
        if total == 0 {
            return "Chưa có dữ liệu".to_string();
        }
        
        let avg = if count > 0 { time / count } else { 0 };
        format!(
            "Tổng: {} lần ({} thành công, {} thất bại), Thời gian trung bình: {}ns",
            total, count, fail, avg
        )
    }
    
    /// Lấy tỷ lệ lỗi
    pub fn rate(&self) -> f64 {
        let count = self.count.load(Ordering::Relaxed);
        if count == 0 {
            return 0.0;
        }
        self.fail.load(Ordering::Relaxed) as f64 / count as f64
    }
}

impl Registry { 
    /// Tạo registry mới
    pub fn new() -> Self {
        Self {
            map: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Lấy metric cho một thao tác
    pub async fn get(&self, name: &str) -> Metric {
        let mut map = self.map.write().await;
        map.entry(name.to_string())
            .or_insert_with(Metric::new)
            .clone()
    }
    
    /// Lấy thống kê cho tất cả metrics
    pub async fn stats(&self) -> String {
        let map = self.map.read().await;
        let mut stats = Vec::new();
        for (name, metric) in map.iter() {
            stats.push(format!("{}: {}", name, metric.stats()));
        }
        stats.join("\n")
    }
} 