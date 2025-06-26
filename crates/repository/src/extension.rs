//! Extension traits cho Error để hỗ trợ chuyển đổi từ các loại lỗi khác.
//!
//! Module này cung cấp các tiện ích để chuyển đổi từ các loại lỗi
//! bên ngoài (như lỗi từ các thư viện) sang loại `Error` của hệ thống.
//! Điều này giúp cho việc xử lý lỗi trở nên nhất quán trong toàn bộ ứng dụng.

// ---
// Import các định nghĩa lỗi nội bộ và loại lỗi từ thư viện ngoài
use crate::Error; // Error: Enum lỗi chuẩn hóa của hệ thống
use tracing_subscriber::filter::ParseError; // ParseError: Lỗi phân tích cú pháp từ tracing_subscriber

/// Trait cung cấp các phương thức chuyển đổi cho Error
/// Mục đích: Chuẩn hóa việc chuyển đổi lỗi từ các thư viện ngoài về hệ thống lỗi nội bộ.
pub trait Extension {
    /// Chuyển đổi lỗi phân tích cú pháp thành loại lỗi của chúng ta
    /// Mục đích: Đảm bảo mọi lỗi parse đều được ánh xạ về Error nội bộ.
    fn parse(err: ParseError) -> Self;

    // Phương thức 'io' đã được loại bỏ vì 'Error::Io' trực tiếp đảm nhận
    // Chuyển đổi lỗi vào/ra (IO) thành loại lỗi của chúng ta
    // Mục đích: Bọc lỗi IO từ std thành Error nội bộ, giúp trace nguồn gốc lỗi IO.
    // fn io(err: std::io::Error) -> Self;
}

// Triển khai Extension cho enum Error của hệ thống
impl Extension for Error {
    fn parse(_err: ParseError) -> Self {
        Error::Validation("Lỗi phân tích cú pháp JSON.".to_string()) // Ánh xạ lỗi phân tích thành lỗi đầu vào
    }

    // Triển khai cho 'io' đã được loại bỏ
    // fn io(err: std::io::Error) -> Self {
    //     Error::Store(sled::Error::Io(err)) // Bọc lỗi IO trong lỗi lưu trữ (Store), giúp trace lỗi IO từ sled
    // }
}

// impl From<serde_json::Error> for Error {
//     fn from(_err: serde_json::Error) -> Self {
//         // Hiện tại, chúng ta không cần chi tiết về lỗi serde,
//         // chỉ cần biết rằng nó đã thất bại.
//         Error::Validation("Lỗi phân tích cú pháp JSON.".to_string()) // Ánh xạ lỗi phân tích thành lỗi đầu vào
//     }
// }
