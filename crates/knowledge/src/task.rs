//! Module quản lý các bản ghi công việc (todo) thông qua `todo` crate.

use repository::{Error, Storage, Id};
use task::{self, Patch};
pub use task::{Entry, Status, Priority};

/// Thêm một công việc mới.
/// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
#[allow(clippy::too_many_arguments)]
pub async fn add<S: Storage>(store: &S, context: String, module: String, task: String, priority: String, status: String, assignee: String, due: String, notes: String) -> Result<Entry, Error> {
    let status_enum = Status::try_from(status.to_string())?;
    let priority_enum = Priority::try_from(priority.to_string())?;
    task::add(store, context, module, task, priority_enum, status_enum, assignee, due, notes).await
}

/// Lấy một công việc bằng ID.
/// Mục đích: Cung cấp giao diện `get` cho `knowledge` CLI.
pub async fn get<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    task::find(store, id).await
}

/// Đánh dấu một công việc là đã hoàn thành.
/// Mục đích: Cung cấp giao diện `done` cho `knowledge` CLI.
pub async fn done<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    let patch = Patch { status: Some(Status::Done), ..Default::default() };
    task::change(store, id, patch).await
}

/// Xóa một công việc.
/// Mục đích: Cung cấp giao diện `del` cho `knowledge` CLI.
/// 
pub async fn del<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    task::remove(store, id).await
}

/// Liệt kê các công việc với bộ lọc trạng thái.
/// Mục đích: Cung cấp giao diện `list` cho `knowledge` CLI.
pub async fn list<S: Storage>(
    store: &S,
    done: bool,
    _pending: bool,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<task::Summary, Error>> + Send>, Error> {
    let status = if done {
        "completed"
    } else {
        "pending"
    };
    let prefix = vec![status.as_bytes()[0]];
    let query_obj = shared::query(prefix, None::<Vec<u8>>, limit);
    task::query(store, query_obj).await
}