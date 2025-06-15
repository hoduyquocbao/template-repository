//! Extension traits cho Error để hỗ trợ chuyển đổi từ các loại lỗi khác.
//!
//! Module này cung cấp các tiện ích để chuyển đổi từ các loại lỗi
//! bên ngoài (như lỗi từ các thư viện) sang loại `Error` của hệ thống.
//! Điều này giúp cho việc xử lý lỗi trở nên nhất quán trong toàn bộ ứng dụng.

use crate::Error;
use tracing_subscriber::filter::ParseError;

/// Trait cung cấp các phương thức chuyển đổi cho Error
pub trait Extension {
    /// Chuyển đổi lỗi phân tích cú pháp thành loại lỗi của chúng ta
    fn parse(err: ParseError) -> Self;
    
    /// Chuyển đổi lỗi vào/ra thành loại lỗi của chúng ta
    fn io(err: std::io::Error) -> Self;
}

impl Extension for Error {
    fn parse(_err: ParseError) -> Self {
        Error::Input // Ánh xạ lỗi phân tích thành lỗi đầu vào
    }
    
    fn io(err: std::io::Error) -> Self {
        Error::Store(sled::Error::Io(err)) // Bọc lỗi IO trong lỗi lưu trữ
    }
}
