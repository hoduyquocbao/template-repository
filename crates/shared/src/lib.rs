// use serde::{Serialize, Deserialize};
use repository::{Query, Id, Key};

/// Trait để định nghĩa cách một Summary được hiển thị.
/// Mục đích: Cho phép hàm 'show' generic hóa cách in ra màn hình.
pub trait Showable {
    fn show(&self);
}

/// Trait chuẩn hóa cho các entity có thể filter/query theo prefix/after.
pub trait Filterable {
    type Prefix;
    type After;
    fn prefix(&self) -> Self::Prefix;
    fn after(&self) -> Option<Self::After>;
}

/// Hàm tiện ích để tạo truy vấn lọc theo trạng thái (dùng cho search/query nhiều domain).
pub fn filter(done: bool, after: Option<(u128, Id)>, limit: usize) -> Query<Vec<u8>> {
    let prefix = vec![if done { 1 } else { 0 }];
    let after = after.map(|(created, id)| {
        let mut key = Key::reserve(33);
        key.flag(done);
        key.time(created);
        key.id(id);
        key.clone().build()
    });
    Query { prefix, after, limit }
}

/// Hàm tiện ích tạo Query cho mọi domain, nhận vào prefix, after, limit.
pub fn query<P, A>(prefix: P, after: Option<A>, limit: usize) -> repository::Query<Vec<u8>>
where
    P: Into<Vec<u8>>,
    A: Into<Vec<u8>>,
{
    repository::Query {
        prefix: prefix.into(),
        after: after.map(|a| a.into()),
        limit,
    }
}
