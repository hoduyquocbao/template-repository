//! Module quản lý các bản ghi bộ nhớ thông qua `memories` crate.

use repository::{Error, Storage, Id};
use memories::{self, Entry}; // Chỉ import Mem, không import Summary

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
pub async fn add<S: Storage>(store: &S, args: Add) -> Result<Entry, Error> {
    let result = memories::add(
        store,
        args.r#type,
        args.context,
        args.module,
        args.subject,
        args.description,
        args.decision,
        args.rationale,
        args.created,
    ).await?;
    Ok(result)
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
    r#type: Option<char>,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<memories::Summary, Error>> + Send>, Error> { // Sử dụng memories::Summary đầy đủ
    memories::query(store, r#type, None, limit).await
}