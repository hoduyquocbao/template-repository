//! Module quản lý các bản ghi bộ nhớ thông qua `memories` crate.

use repository::{Error, Id, Storage};
pub use memories::{Entry, Kind, Summary};
use shared;

#[derive(Debug, Clone)]
pub struct Add {
    pub r#type: String, // Giữ nguyên String cho dữ liệu truyền từ CLI
    pub context: String,
    pub module: String,
    pub subject: String,
    pub description: String,
    pub decision: String,
    pub rationale: String,
    pub created: u128,
}

/// Thêm một bản ghi bộ nhớ mới.
/// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
#[allow(clippy::too_many_arguments)]
pub async fn add<S: Storage>(
    store: &S,
    args: Add,
) -> Result<memories::Entry, repository::Error> {
    memories::add(
        store,
        args.r#type,
        args.context,
        args.module,
        args.subject,
        args.description,
        args.decision,
        args.rationale,
    )
    .await
}

/// Lấy một bản ghi bộ nhớ bằng ID.
/// Mục đích: Cung cấp giao diện `get` cho `knowledge` CLI.
pub async fn get<S: Storage>(store: &S, id: Id) -> Result<Option<Entry>, Error> {
    memories::find(store, id).await
}

/// Liệt kê các bản ghi bộ nhớ.
/// Mục đích: Cung cấp giao diện `list` cho `knowledge` CLI.
pub async fn list<S: Storage>(
    store: &S,
    kind: Option<String>,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<memories::Summary, repository::Error>> + Send>, repository::Error> {
    let prefix_vec = match kind {
        Some(s) => {
            let kind = Kind::try_from(s)?;
            vec![(&kind).into()]
        }
        None => Vec::new(),
    };
    let query = shared::query(prefix_vec, None::<Vec<u8>>, limit);
    memories::query(store, query).await
}