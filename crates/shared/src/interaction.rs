use repository::{self, Id};
use std::fmt::Debug;
use std::time::SystemTime;

/// Một trait đánh dấu một struct là một lệnh có thể thực thi.
/// Mọi lệnh phải định nghĩa kiểu dữ liệu Output của nó.
pub trait Command: Debug {
    type Output;
}

/// Đóng gói một Command với các metadata cho việc ghi nhật ký và truy vết.
#[derive(Debug)]
pub struct Interaction<C: Command> {
    /// ID duy nhất cho mỗi lần tương tác.
    pub id: Id,
    /// Thời điểm tương tác được tạo ra.
    pub timestamp: SystemTime,
    /// Lệnh cụ thể được yêu cầu.
    pub command: C,
}

impl<C: Command> Interaction<C> {
    pub fn new(command: C) -> Self {
        Self {
            id: Id::new_v4(),
            timestamp: SystemTime::now(),
            command,
        }
    }
} 