//! Định nghĩa các loại lỗi có thể xảy ra trong hệ thống.
//!
//! Module này sử dụng crate `thiserror` để tạo ra một enum lỗi có cấu trúc
//! và có thể mở rộng. Điều này cho phép xử lý lỗi một cách chính xác và
//! thông báo lỗi rõ ràng cho người dùng.

// ---
// Import macro derive cho error, giúp tự động sinh code cho enum lỗi
use thiserror::Error; // thiserror: Chuẩn hóa và đơn giản hóa việc định nghĩa lỗi

/// Các loại lỗi có thể xảy ra trong hệ thống.
///
/// Enum này triển khai `std::error::Error` thông qua derive macro của thiserror,
/// cung cấp các thông báo lỗi có định dạng tốt và khả năng chuyển đổi từ các loại lỗi khác.
/// Mục đích: Chuẩn hóa toàn bộ hệ thống lỗi, giúp debug và xử lý nhất quán.
#[derive(Error, Debug)]
pub enum Error {
    /// Được trả về khi một mục được yêu cầu không tồn tại.
    /// Mục đích: Phân biệt lỗi không tìm thấy với các lỗi khác.
    #[error("mục không tìm thấy")]
    Missing,

    /// Được trả về khi đầu vào không hợp lệ được cung cấp.
    /// Mục đích: Bắt các trường hợp dữ liệu đầu vào sai, thiếu, hoặc không hợp lệ.
    #[error("đầu vào không hợp lệ")]
    Input,

    /// Lỗi từ lớp lưu trữ cơ bản (sled).
    /// Mục đích: Bọc lỗi từ backend lưu trữ, giúp trace nguồn gốc lỗi.
    /// Thuật toán: Sử dụng #[from] để tự động chuyển đổi từ sled::Error sang Error::Store.
    #[error("lỗi lưu trữ: {0}")]
    Store(#[from] sled::Error),

    /// Lỗi khi tuần tự hóa hoặc giải tuần tự hóa dữ liệu (bincode).
    /// Mục đích: Bọc lỗi serialization/deserialization, giúp phát hiện lỗi định dạng dữ liệu.
    #[error("lỗi định dạng: {0}")]
    Format(#[from] bincode::Error),

    /// Được trả về khi một giao dịch bị hủy bỏ (abort transaction).
    /// Mục đích: Phân biệt lỗi logic trong transaction với lỗi hệ thống.
    #[error("giao dịch bị hủy bỏ")]
    Aborted,

    /// Lỗi từ tác vụ bất đồng bộ (tokio join error).
    /// Mục đích: Bọc lỗi khi join các task bất đồng bộ thất bại.
    #[error("lỗi tác vụ bất đồng bộ: {0}")]
    Join(#[from] tokio::task::JoinError),

    /// Lỗi khi kết nối bị timeout.
    /// Mục đích: Phát hiện các thao tác bị treo hoặc backend không phản hồi kịp.
    #[error("kết nối bị timeout")]
    Timeout,

    /// Lỗi khi không thể lấy kết nối từ pool.
    /// Mục đích: Phát hiện pool cạn kiệt hoặc deadlock.
    #[error("không thể lấy kết nối từ pool")]
    Pool,

    /// Lỗi khi cache bị đầy.
    /// Mục đích: Phát hiện và xử lý tình trạng cache overflow.
    #[error("cache bị đầy")]
    Cache,

    /// Lỗi khi metric không hợp lệ.
    /// Mục đích: Đảm bảo các thao tác với metric luôn hợp lệ, phát hiện lỗi cấu hình hoặc logic.
    #[error("metric không hợp lệ")]
    Metric,

    /// Lỗi vào/ra từ hệ điều hành (file, network, v.v.).
    /// Mục đích: Xử lý các lỗi IO chung.
    #[error("lỗi io: {0}")]
    Io(#[from] std::io::Error), // THÊM MỚI

    /// Lỗi khi xử lý dữ liệu CSV.
    /// Mục đích: Phân biệt lỗi liên quan đến định dạng CSV hoặc đọc/ghi CSV.
    #[error("lỗi csv: {0}")]
    Csv(#[from] csv::Error), // THÊM MỚI
}