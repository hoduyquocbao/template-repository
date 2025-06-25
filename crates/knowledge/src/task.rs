//! Module quản lý các bản ghi công việc (todo) thông qua `todo` crate.

use repository::{Error, Storage, Id};
use todo::{self, Todo}; // Import Todo, không import Summary
use shared::Patch;

/// Thêm một công việc mới.
/// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
pub async fn add<S: Storage>(store: &S, text: String) -> Result<Todo, Error> {
    todo::add(store, text).await
}

/// Lấy một công việc bằng ID.
/// Mục đích: Cung cấp giao diện `get` cho `knowledge` CLI.
pub async fn get<S: Storage>(store: &S, id: Id) -> Result<Todo, Error> {
    todo::find(store, id).await
}

/// Đánh dấu một công việc là đã hoàn thành.
/// Mục đích: Cung cấp giao diện `done` cho `knowledge` CLI.
pub async fn done<S: Storage>(store: &S, id: Id) -> Result<Todo, Error> {
    let patch = Patch {
        text: None,
        done: Some(true),
    };
    todo::change(store, id, patch).await
}

/// Xóa một công việc.
/// Mục đích: Cung cấp giao diện `del` cho `knowledge` CLI.
pub async fn del<S: Storage>(store: &S, id: Id) -> Result<Todo, Error> {
    todo::remove(store, id).await
}

/// Liệt kê các công việc với bộ lọc trạng thái.
/// Mục đích: Cung cấp giao diện `list` cho `knowledge` CLI.
pub async fn list<S: Storage>(
    store: &S,
    done: bool,
    pending: bool,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<todo::Summary, Error>> + Send>, Error> { // Sử dụng todo::Summary đầy đủ
    // Xác định trạng thái cần truy vấn. Mặc định là 'pending' nếu không có cờ nào.
    let status = if done {
        true
    } else if pending || !done {
        false
    } else {
        return Err(Error::Input); // Không bao giờ xảy ra nhờ conflicts_with trong CLI
    };
    todo::query(store, status, None, limit).await
}