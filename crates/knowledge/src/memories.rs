//! Module quản lý các bản ghi bộ nhớ thông qua `memories` crate.

use repository::{Error, Storage, Id};
use memories::{self, Entry}; // Chỉ import Mem, không import Summary
use shared;

#[derive(Debug, Clone)]
pub struct Add {
    pub r#type: String,
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
    r#type: String,
    context: String,
    module: String,
    subject: String,
    description: String,
    decision: String,
    rationale: String,
) -> Result<memories::Entry, repository::Error> {
    memories::add(
        store,
        r#type,
        context,
        module,
        subject,
        description,
        decision,
        rationale,
    ).await
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
    prefix: Option<char>,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<memories::Summary, repository::Error>> + Send>, repository::Error> {
    let prefix_vec = prefix.map_or(Vec::new(), |c| vec![c as u8]);
    let query = shared::query(prefix_vec, None::<Vec<u8>>, limit);
    memories::query(store, query).await
}