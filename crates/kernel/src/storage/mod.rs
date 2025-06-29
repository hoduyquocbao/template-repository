//! Định nghĩa giao diện trừu tượng (`Storage` trait) cho việc lưu trữ bất kỳ thực thể nào.
//!
//! Module này tách rời logic nghiệp vụ khỏi các triển khai lưu trữ cụ thể,
//! cho phép hệ thống dễ dàng thay đổi hoặc thêm các backend lưu trữ mới.
//! Thiết kế này tuân theo nguyên tắc Dependency Inversion, cho phép
//! các lớp nghiệp vụ cấp cao phụ thuộc vào các trừu tượng, không phải các triển khai cụ thể.

// ---
// Import các thư viện cần thiết cho trait bất đồng bộ, kiểm tra kiểu, và các định nghĩa cốt lõi
use async_trait::async_trait; // Cho phép định nghĩa trait với hàm async
use std::fmt::Debug; // Đảm bảo các khóa/chỉ mục có thể debug dễ dàng
use crate::{Error, storage::entity::{Entity, Query}}; // Import các định nghĩa lỗi, trait Entity và struct Query
use serde; // Import serde module

/// Hợp đồng cho bất kỳ cơ chế lưu trữ nào muốn làm việc với framework.
/// 
/// Trait này định nghĩa một tập các thao tác mà mọi backend lưu trữ phải triển khai.
/// Nó sử dụng `async_trait` để hỗ trợ các phương thức bất đồng bộ, cho phép
/// các triển khai tối ưu hiệu năng thông qua xử lý đồng thời.
#[async_trait]
pub trait Storage: Send + Sync { // Trait phải thread-safe để dùng trong môi trường bất đồng bộ
    /// Chèn một thực thể mới vào backend lưu trữ.
    /// Mục đích: Đảm bảo mọi backend đều hỗ trợ thêm mới thực thể.
    /// Thuật toán: Có thể dùng transaction, cache, hoặc ghi trực tiếp tuỳ backend.
    /// Thành tựu: Đảm bảo tính mở rộng và nhất quán khi thêm dữ liệu.
    async fn insert<E: Entity>(&self, entity: E) -> Result<(), Error>
    where E::Key: Debug + serde::Serialize, E::Index: Debug;

    /// Lấy một thực thể bằng khóa chính.
    /// Mục đích: Cho phép truy xuất nhanh một thực thể duy nhất.
    /// Thuật toán: Có thể dùng cache, index, hoặc truy vấn trực tiếp backend.
    /// Thành tựu: Đảm bảo khả năng truy xuất hiệu quả và chính xác.
    async fn fetch<E: Entity>(&self, key: E::Key) -> Result<Option<E>, Error>
    where E::Key: Debug + serde::Serialize;

    /// Cập nhật một thực thể dựa trên hàm biến đổi (transform).
    /// Mục đích: Cho phép cập nhật nguyên tử một thực thể với logic tuỳ biến.
    /// Thuật toán: Đọc thực thể, áp dụng transform, ghi lại (có thể dùng transaction).
    /// Thành tựu: Đảm bảo tính toàn vẹn dữ liệu khi cập nhật đồng thời.
    async fn update<E: Entity, F>(&self, key: E::Key, transform: F) -> Result<E, Error>
    where
        F: FnOnce(E) -> E + Send + 'static, // Hàm biến đổi phải thread-safe
        E::Key: Debug + serde::Serialize;

    /// Xóa một thực thể khỏi backend lưu trữ.
    /// Mục đích: Đảm bảo mọi backend đều hỗ trợ xóa dữ liệu.
    /// Thuật toán: Xóa theo khoá chính, có thể kết hợp xóa index/phụ trợ.
    /// Thành tựu: Đảm bảo dữ liệu không còn tồn tại và không rò rỉ index.
    async fn delete<E: Entity>(&self, key: E::Key) -> Result<E, Error>
    where E::Key: Debug + serde::Serialize;

    /// Truy vấn một danh sách các bản tóm tắt dưới dạng một stream iterator.
    /// Mục đích: Hỗ trợ truy vấn hiệu quả với phân trang, tiền tố, và giới hạn.
    /// Thuật toán: Có thể dùng covering index, range scan, hoặc filter tuỳ backend.
    /// Thành tựu: Đảm bảo khả năng liệt kê dữ liệu lớn mà không tốn bộ nhớ.
    async fn query<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<E::Summary, Error>> + Send>, Error>
    where E::Index: Debug;

    /// Chèn hàng loạt các thực thể (bulk insert).
    /// Mục đích: Tối ưu hiệu năng khi thêm nhiều thực thể cùng lúc.
    /// Thuật toán: Chia lô, dùng transaction, hoặc ghi tuần tự tuỳ backend.
    /// Thành tựu: Đảm bảo hiệu năng cao và an toàn bộ nhớ khi thao tác dữ liệu lớn.
    async fn mass<E: Entity>(&self, iter: Box<dyn Iterator<Item = E> + Send>) -> Result<(), Error>
    where E::Key: Debug + serde::Serialize, E::Index: Debug;
    
    /// Hàm trợ giúp cho benchmark - lấy các khóa chỉ mục (chỉ bật khi test/benchmark).
    /// Mục đích: Hỗ trợ kiểm thử hiệu năng và xác minh hoạt động index.
    /// Thuật toán: Truy vấn index, trả về iterator các khoá.
    /// Thành tựu: Đảm bảo khả năng kiểm thử và benchmark toàn diện.
    #[cfg(any(test, feature = "testing"))]
    async fn keys<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>> + Send>, Error>
    where E::Index: Debug;
}

// --- Các module con của storage ---
pub mod actor;
pub mod sled;
pub mod pool;    // Module quản lý pool kết nối
pub mod cache;   // Module cache
pub mod entity;  // Module định nghĩa trait Entity
pub mod time;    // Module tiện ích thời gian
pub mod export;  // Module export dữ liệu

// --- Re-export các thành phần từ module export ---
pub use export::{
    Exportable,
    Transformable,
    Validatable,
    Streamable,
    Config,
    Filter,
    Format,
    Stream,
    Export,
    Builder,
    Ext,
};