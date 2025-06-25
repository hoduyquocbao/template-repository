//! Module quản lý các bản ghi kiến trúc thông qua `architecture` crate.

use repository::{Error, Storage};
use architecture::{self, Entry}; // Chỉ import Arch, không import Summary hay đổi tên

#[derive(Debug, Clone)]
pub struct Add {
    pub context: String,
    pub module: String,
    pub r#type: String,
    pub name: String,
    pub responsibility: String,
    pub dependency: String,
    pub performance: String,
    pub naming: String,
    pub prompt: String,
    pub created: u128,
}

/// Thêm hoặc cập nhật một bản ghi kiến trúc.
/// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
pub async fn add<S: Storage>(store: &S, args: Add) -> Result<Entry, Error> {
    let entry = Entry {
        context: args.context,
        module: args.module,
        r#type: args.r#type,
        name: args.name,
        responsibility: args.responsibility,
        dependency: args.dependency,
        performance: args.performance,
        naming: args.naming,
        prompt: args.prompt,
        created: args.created,
    };
    let result = architecture::add(store, entry).await?;
    Ok(result)
}

/// Lấy một bản ghi kiến trúc bằng key tổng hợp.
/// Mục đích: Cung cấp giao diện `get` cho `knowledge` CLI.
pub async fn get<S: Storage>(
    store: &S,
    context: String,
    module: String,
    r#type: String,
    name: String,
) -> Result<Option<Entry>, Error> {
    let key = format!("{}:{}:{}:{}", context, module, r#type, name);
    architecture::find(store, key).await
}

/// Xóa một bản ghi kiến trúc.
/// Mục đích: Cung cấp giao diện `del` cho `knowledge` CLI.
pub async fn del<S: Storage>(
    store: &S,
    context: String,
    module: String,
    r#type: String,
    name: String,
) -> Result<Entry, Error> {
    let key = format!("{}:{}:{}:{}", context, module, r#type, name);
    architecture::remove(store, key).await
}

/// Liệt kê các bản ghi kiến trúc.
/// Mục đích: Cung cấp giao diện `list` cho `knowledge` CLI.
pub async fn list<S: Storage>(
    store: &S,
    prefix: String,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<architecture::Summary, Error>> + Send>, Error> { // SỬ DỤNG architecture::Summary ĐẦY ĐỦ
    let query_prefix_bytes = prefix.as_bytes().to_vec();
    architecture::query(store, query_prefix_bytes, None, limit).await
}