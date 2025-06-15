//! Định nghĩa các loại lỗi có thể xảy ra trong hệ thống.
//!
//! Module này sử dụng crate `thiserror` để tạo ra một enum lỗi có cấu trúc
//! và có thể mở rộng. Điều này cho phép xử lý lỗi một cách chính xác và 
//! thông báo lỗi rõ ràng cho người dùng.

use thiserror::Error;

/// Các loại lỗi có thể xảy ra trong hệ thống.
///
/// Enum này triển khai `std::error::Error` thông qua derive macro của thiserror,
/// cung cấp các thông báo lỗi có định dạng tốt và khả năng chuyển đổi từ các loại lỗi khác.
#[derive(Error, Debug)]
pub enum Error {
    /// Được trả về khi một mục được yêu cầu không tồn tại.
    #[error("mục không tìm thấy")]
    Missing,
    
    /// Được trả về khi đầu vào không hợp lệ được cung cấp.
    #[error("đầu vào không hợp lệ")]
    Input,
    
    /// Lỗi từ lớp lưu trữ cơ bản.
    #[error("lỗi lưu trữ: {0}")]
    Store(#[from] sled::Error),
    
    /// Lỗi khi tuần tự hóa hoặc giải tuần tự hóa dữ liệu.
    #[error("lỗi định dạng: {0}")]
    Format(#[from] bincode::Error),
    
    /// Được trả về khi một giao dịch bị hủy bỏ.
    #[error("giao dịch bị hủy bỏ")]
    Aborted,
    
    /// Lỗi từ tác vụ bất đồng bộ.
    #[error("lỗi tác vụ bất đồng bộ: {0}")]
    Join(#[from] tokio::task::JoinError),
    
    /// Lỗi khi kết nối bị timeout.
    #[error("kết nối bị timeout")]
    Timeout,
    
    /// Lỗi khi không thể lấy kết nối từ pool.
    #[error("không thể lấy kết nối từ pool")]
    Pool,
    
    /// Lỗi khi cache bị đầy.
    #[error("cache bị đầy")]
    Cache,
    
    /// Lỗi khi metric không hợp lệ.
    #[error("metric không hợp lệ")]
    Metric,
}