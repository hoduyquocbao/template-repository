//! Định nghĩa cốt lõi cho bất kỳ thực thể nào có thể lưu trữ được.
//!
//! Trait `Entity` là trái tim của framework, định nghĩa các yêu cầu 
//! mà bất kỳ kiểu dữ liệu nào cũng phải thỏa mãn để có thể được lưu trữ
//! và truy vấn hiệu quả.

use serde::{de::DeserializeOwned, Serialize};
use crate::Id;
use std::fmt::Debug;

/// Một "hợp đồng" cho bất kỳ loại dữ liệu nào có thể được lưu trữ và lập chỉ mục.
///
/// Trait này là trái tim của framework Bedrock. Nó buộc mọi loại dữ liệu
/// phải định nghĩa rõ ràng cách nó được nhận dạng, lưu trữ, và quan trọng nhất,
/// cách nó được lập chỉ mục để truy vấn hiệu suất cao.
pub trait Entity: Serialize + DeserializeOwned + Clone + Send + Sync + 'static {
    /// Tên định danh cho loại thực thể này, dùng để tạo các "cây" lưu trữ riêng biệt.
    const NAME: &'static str;

    /// Loại dữ liệu dùng làm khóa chính trong cây dữ liệu. Thường là `Id`.
    type Key: AsRef<[u8]> + Clone + Send + Debug;

    /// Loại dữ liệu dùng làm khóa cho cây chỉ mục. Phải có thể sắp xếp.
    type Index: AsRef<[u8]> + Clone + Send + Debug;

    /// Một phiên bản tóm tắt của thực thể, được lưu trong chỉ mục bao phủ.
    type Summary: Serialize + DeserializeOwned + Send;

    /// Trả về khóa chính của thực thể.
    fn key(&self) -> Self::Key;

    /// Trả về khóa chỉ mục của thực thể.
    /// Logic tạo khóa (ví dụ: đảo ngược timestamp) được gói gọn tại đây.
    fn index(&self) -> Self::Index;
    
    /// Trả về một bản tóm tắt của thực thể để lưu vào chỉ mục.
    fn summary(&self) -> Self::Summary;
}

/// Cấu trúc tham số truy vấn cho các thao tác truy vấn.
///
/// Cấu trúc này tổng quát hóa các tiêu chí truy vấn phổ biến như phân trang
/// và giới hạn kết quả, mà không ràng buộc vào bất kỳ kiểu thực thể cụ thể nào.
#[derive(Debug, Clone)]
pub struct Query<I: AsRef<[u8]> + Clone> {
    /// Tiền tố chỉ mục để lọc kết quả
    pub prefix: Vec<u8>,
    
    /// Khóa chỉ mục để bắt đầu sau đó (dùng cho phân trang)
    pub after: Option<I>,
    
    /// Số lượng kết quả tối đa
    pub limit: usize,
}

impl<I: AsRef<[u8]> + Clone> Default for Query<I> {
    fn default() -> Self {
        Self {
            prefix: Vec::new(),
            after: None,
            limit: 10, // Giá trị mặc định hợp lý
        }
    }
}

/// Một bộ công cụ tiện ích cho việc xây dựng các khóa chỉ mục phức tạp.
///
/// Struct này giúp tạo ra các khóa chỉ mục đa thành phần một cách nhất quán,
/// đảm bảo tính thống nhất giữa các thực thể khác nhau.
#[derive(Clone)]
pub struct Key(Vec<u8>);

impl Key {
    /// Thay đổi: `with_capacity` thành `reserve` để tuân thủ quy tắc một từ.
    /// Tạo một builder mới với dung lượng đã cấp phát sẵn.
    pub fn reserve(capacity: usize) -> Self {
        Self(Vec::with_capacity(capacity))
    }
    
    /// Thay đổi: `add_bool` thành `flag` để mô tả vai trò (cờ trạng thái).
    /// Thêm một phần tử boolean vào khóa.
    pub fn flag(&mut self, value: bool) -> &mut Self {
        self.0.push(if value { 1 } else { 0 });
        self
    }
    
    /// Thay đổi: `add_rev_time` thành `time` để đặt tên theo mục đích.
    /// Thêm một timestamp đảo ngược (để sắp xếp mới nhất trước).
    pub fn time(&mut self, value: u128) -> &mut Self {
        self.0.extend_from_slice(&(u128::MAX - value).to_be_bytes());
        self
    }
    
    /// Thay đổi: `add_id` thành `id` để đặt tên theo dữ liệu.
    /// Thêm một ID vào khóa.
    pub fn id(&mut self, value: Id) -> &mut Self {
        self.0.extend_from_slice(value.as_bytes());
        self
    }
    
    /// Thay đổi: `add_u8` thành `byte` để đặt tên theo dữ liệu.
    /// Thêm một số nguyên vào khóa (với thứ tự byte big-endian).
    pub fn byte(&mut self, value: u8) -> &mut Self {
        self.0.push(value);
        self
    }
    
    /// Hoàn thành và lấy khóa dưới dạng Vec<u8>.
    pub fn build(self) -> Vec<u8> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Thay đổi: `key_builder_works` thành `build` cho ngắn gọn.
    #[test]
    fn build() {
        let id = Id::new_v4();
        let time = 12345u128;
        
        // Cập nhật các lời gọi phương thức để phản ánh thay đổi
        let key = Key::reserve(33)
            .flag(true)
            .time(time)
            .id(id).clone()
            .build();
            
        assert_eq!(key[0], 1); // true -> 1
        assert_eq!(key.len(), 1 + 16 + 16); // bool + u128 + uuid
    }
}
