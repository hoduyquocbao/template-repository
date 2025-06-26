#![doc = "repository - Framework xây dựng các ứng dụng lưu trữ hiệu năng cao, có khả năng xử lý hàng tỷ bản ghi."]
#![doc = "Framework này được thiết kế với kiến trúc thanh lịch cho phép tái sử dụng và mở rộng, tuân thủ triết lý định danh một từ đơn."]
#![doc = "Nó sử dụng một lớp lưu trữ trừu tượng, triển khai ban đầu với Sled, và có khả năng quan sát sâu nhờ framework `tracing`."]

// lib.rs
// Crate thư viện chứa tất cả logic cốt lõi của framework.
// Mục tiêu: Định nghĩa điểm vào trung tâm, gom nhóm và tái xuất các module quan trọng.
// Thành tựu: Đảm bảo mọi định danh public đều là một từ tiếng Anh, API rõ ràng, dễ dùng.

// --- Định nghĩa các module con ---
// Mỗi module đại diện cho một khía cạnh cốt lõi của hệ thống, được đặt tên một từ duy nhất.
pub mod error;      // Module quản lý lỗi, chuẩn hóa toàn bộ hệ thống lỗi
pub mod extension;  // Module mở rộng, chuyển đổi lỗi từ bên ngoài về hệ thống
pub mod entity;     // Module định nghĩa trait Entity, chuẩn hóa thực thể lưu trữ
pub mod sled;       // Module triển khai lưu trữ với Sled, tối ưu hiệu năng
pub mod storage;    // Module trait Storage, trừu tượng hóa backend lưu trữ
pub mod pool;       // Module quản lý pool kết nối, tối ưu concurrency
pub mod cache;      // Module cache, tăng tốc truy xuất thực thể
pub mod metric;     // Module thu thập metric, quan sát hiệu năng
pub mod time;       // THÊM MỚI: Module tiện ích thời gian
pub mod actor;      // Module actor, mới tạo

// --- Tái xuất các thành phần cốt lõi ---
// Mục đích: Tạo API gọn gàng, giúp người dùng chỉ cần import từ crate gốc
// Thành tựu: Đảm bảo mọi định danh public đều là một từ tiếng Anh, không lộ chi tiết nội bộ
pub use error::Error; // Enum lỗi chuẩn hóa, một từ duy nhất
pub use extension::Extension; // Trait mở rộng lỗi, một từ duy nhất
pub use entity::{Entity, Query, Key}; // Trait thực thể, struct truy vấn, builder khóa
pub use sled::Sled; // Struct lưu trữ chính, một từ duy nhất
pub use storage::Storage; // Trait lưu trữ trừu tượng, một từ duy nhất
pub use uuid::Uuid as Id; // Định danh duy nhất, tái xuất với tên Id (một từ)
pub use pool::Pool; // Struct pool kết nối, một từ duy nhất
pub use cache::Cache; // Struct cache, một từ duy nhất
pub use metric::{Metric, Registry}; // Struct metric và registry, một từ duy nhất
pub use time::now; // THÊM MỚI: Tái xuất hàm now()