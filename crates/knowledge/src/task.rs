//! Module quản lý các bản ghi công việc (todo) thông qua `todo` crate.

use repository::{Error, Id, Query, Storage};
pub use task::{Entry, Patch, Priority, Status, Summary};
use task;

#[derive(Debug, Clone)]
pub struct Add {
    pub context: String,
    pub module: String,
    pub task: String,
    pub priority: Priority,
    pub status: Status,
    pub assignee: String,
    pub due: String,
    pub notes: String,
}

/// Thêm một công việc mới.
/// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
pub async fn add<S: Storage>(store: &S, args: Add) -> Result<Entry, Error> {
    task::add(
        store,
        args.context,
        args.module,
        args.task,
        args.priority,
        args.status,
        args.assignee,
        args.due,
        args.notes,
    )
    .await
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
    query: Query<Vec<u8>>,
) -> Result<Vec<Summary>, Error> {
    let results = task::query(store, query).await?;
    results.collect()
}

/// Thay đổi một công việc.
pub async fn change<S: Storage>(store: &S, id: Id, patch: Patch) -> Result<Entry, Error> {
    task::change(store, id, patch).await
}