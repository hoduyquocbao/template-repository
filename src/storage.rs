//! Định nghĩa giao diện trừu tượng (`Storage` trait) cho việc lưu trữ bất kỳ thực thể nào.
//!
//! Module này tách rời logic nghiệp vụ khỏi các triển khai lưu trữ cụ thể,
//! cho phép hệ thống dễ dàng thay đổi hoặc thêm các backend lưu trữ mới.
//! Thiết kế này tuân theo nguyên tắc Dependency Inversion, cho phép
//! các lớp nghiệp vụ cấp cao phụ thuộc vào các trừu tượng, không phải các triển khai cụ thể.

use async_trait::async_trait;
use std::fmt::Debug;
use crate::{Error, entity::{Entity, Query}};

/// Hợp đồng cho bất kỳ cơ chế lưu trữ nào muốn làm việc với framework.
/// 
/// Trait này định nghĩa một tập các thao tác mà mọi backend lưu trữ phải triển khai.
/// Nó sử dụng `async_trait` để hỗ trợ các phương thức bất đồng bộ, cho phép
/// các triển khai tối ưu hiệu năng thông qua xử lý đồng thời.
#[async_trait]
pub trait Storage: Send + Sync {
    /// Chèn một thực thể mới.
    ///
    /// # Đối số
    ///
    /// * `entity`: Thực thể cần lưu trữ.
    async fn insert<E: Entity>(&self, entity: E) -> Result<(), Error>
    where E::Key: Debug, E::Index: Debug;

    /// Lấy một thực thể bằng khóa chính.
    ///
    /// # Đối số
    ///
    /// * `key`: Khóa chính của thực thể cần tìm.
    ///
    /// # Trả về
    ///
    /// * `Ok(Some(E))`: Nếu tìm thấy.
    /// * `Ok(None)`: Nếu không tìm thấy.
    /// * `Err(Error)`: Nếu có lỗi xảy ra.
    async fn fetch<E: Entity>(&self, key: E::Key) -> Result<Option<E>, Error>
    where E::Key: Debug;

    /// Cập nhật một thực thể dựa trên hàm biến đổi.
    ///
    /// # Đối số
    ///
    /// * `key`: Khóa chính của thực thể cần cập nhật.
    /// * `transform`: Hàm nhận thực thể hiện tại và trả về phiên bản đã sửa đổi.
    ///
    /// # Trả về
    ///
    /// * `Ok(E)`: Thực thể đã được cập nhật thành công.
    /// * `Err(Error::NotFound)`: Nếu không tìm thấy thực thể với khóa đã cho.
    /// * `Err(Error)`: Nếu có lỗi khác xảy ra.
    async fn update<E: Entity, F>(&self, key: E::Key, transform: F) -> Result<E, Error>
    where
        F: FnOnce(E) -> E + Send + 'static,
        E::Key: Debug;

    /// Xóa một thực thể.
    ///
    /// # Đối số
    ///
    /// * `key`: Khóa chính của thực thể cần xóa.
    ///
    /// # Trả về
    ///
    /// * `Ok(E)`: Thực thể đã bị xóa.
    /// * `Err(Error::NotFound)`: Nếu không tìm thấy thực thể với khóa đã cho.
    /// * `Err(Error)`: Nếu có lỗi khác xảy ra.
    async fn delete<E: Entity>(&self, key: E::Key) -> Result<E, Error>
    where E::Key: Debug;

    /// Truy vấn một danh sách các bản tóm tắt dưới dạng một stream.
    ///
    /// # Đối số
    ///
    /// * `query`: Các tham số truy vấn bao gồm tiền tố, vị trí bắt đầu và giới hạn.
    ///
    /// # Trả về
    ///
    /// Một iterator các `Summary` đáp ứng điều kiện truy vấn.
    async fn query<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<E::Summary, Error>> + Send>, Error>
    where E::Index: Debug;

    /// Chèn hàng loạt các thực thể.
    ///
    /// # Đối số
    ///
    /// * `iter`: Iterator cung cấp các thực thể cần chèn.
    ///
    /// # Lưu ý
    ///
    /// Các triển khai nên xử lý việc chèn theo lô để tối ưu hiệu năng và sử dụng bộ nhớ.
    async fn mass<E: Entity>(&self, iter: Box<dyn Iterator<Item = E> + Send>) -> Result<(), Error>
    where E::Key: Debug, E::Index: Debug;
    
    /// Hàm trợ giúp cho benchmark - lấy các khóa chỉ mục
    ///
    /// Phương thức này chỉ khả dụng khi biên dịch với tính năng 'testing'.
    /// Nó được sử dụng để kiểm tra hiệu năng và xác minh hoạt động của chỉ mục.
    #[cfg(any(test, feature = "testing"))]
    async fn keys<E: Entity>(&self, query: Query<E::Index>) 
        -> Result<Box<dyn Iterator<Item = Result<Vec<u8>, Error>> + Send>, Error>
    where E::Index: Debug;
}