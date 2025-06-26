//! Crate này quản lý cấu trúc kiến trúc của hệ thống.
//! Dữ liệu được lưu trữ thông qua `repository::Storage` để tăng hiệu suất.

use serde::{Deserialize, Serialize};
use repository::{now, Entity, Error, Key, Query, Storage};
use shared::{Showable, Filterable};
use std::convert::TryFrom;
use repository::Id;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Kind {
    System,
    Context,
    Module,
    Agent,
    Trait,
    Entity,
    Aggregate,
    Value,
    Event,
    Command,
    Other,
}

impl From<&Kind> for u8 {
    fn from(kind: &Kind) -> u8 {
        match kind {
            Kind::System => 0,
            Kind::Context => 1,
            Kind::Module => 2,
            Kind::Agent => 3,
            Kind::Trait => 4,
            Kind::Entity => 5,
            Kind::Aggregate => 6,
            Kind::Value => 7,
            Kind::Event => 8,
            Kind::Command => 9,
            Kind::Other => 255,
        }
    }
}

impl TryFrom<String> for Kind {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "system" => Ok(Kind::System),
            "context" => Ok(Kind::Context),
            "module" => Ok(Kind::Module),
            "agent" => Ok(Kind::Agent),
            "trait" => Ok(Kind::Trait),
            "entity" => Ok(Kind::Entity),
            "aggregate" => Ok(Kind::Aggregate),
            "value" => Ok(Kind::Value),
            "event" => Ok(Kind::Event),
            "command" => Ok(Kind::Command),
            "other" => Ok(Kind::Other),
            _ => Err(Error::Validation(vec![repository::error::Fault {
                field: "kind".to_string(),
                message: format!("Loại '{}' không hợp lệ.", s),
            }])),
        }
    }
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Đại diện cho một bản ghi kiến trúc.
/// Đây là một `Entity` có thể được lưu trữ và truy vấn thông qua `repository`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: Id,
    pub context: String,      // Ngữ cảnh (Bounded Context)
    pub module: String,       // Module hoặc crate
    pub r#type: Kind,         // Loại thành phần (Agent, Module, Trait, etc.)
    pub name: String,         // Tên định danh
    pub responsibility: String, // Trách nhiệm chính
    pub dependency: String,   // Phụ thuộc
    pub performance: String,  // Ghi chú hiệu suất
    pub naming: String,       // Lý do đặt tên
    pub prompt: String,       // Tham chiếu đến prompt system (ví dụ: "DirectorPrompt.md")
    pub created: u128,        // Timestamp tạo
}

impl Entity for Entry {
    const NAME: &'static str = "architecture";
    type Key = String;
    type Index = Vec<u8>;
    type Summary = Summary;

    fn key(&self) -> Self::Key {
        format!("{}:{}:{}:{}", self.context, self.module, self.r#type, self.name)
    }

    fn index(&self) -> Self::Index {
        let mut index = Vec::new();
        index.push((&self.r#type).into());
        index.extend_from_slice(self.context.as_bytes());
        index.push(0);
        index.extend_from_slice(self.module.as_bytes());
        index.push(0);
        index.extend_from_slice(self.name.as_bytes());
        index
    }

    fn summary(&self) -> Self::Summary {
        Summary {
            context: self.context.clone(),
            module: self.module.clone(),
            name: self.name.clone(),
            r#type: self.r#type.clone(),
            created: self.created,
        }
    }
}

impl Filterable for Entry {
    type Prefix = Vec<u8>;
    type After = Vec<u8>;
    fn prefix(&self) -> Self::Prefix {
        let mut key = Key::reserve(Entity::key(self).len() + 16);
        key.byte(1);
        key.time(self.created);
        key.byte((&self.r#type).into());
        key.build()
    }
    fn after(&self) -> Option<Self::After> {
        None
    }
}

/// Một bản tóm tắt của `Entry` để hiển thị trong danh sách.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Summary {
    pub context: String,
    pub module: String,
    pub name: String,
    pub r#type: Kind,
    pub created: u128,
}

// Triển khai Showable cho Summary của architecture
impl Showable for Summary {
    fn show(&self) {
        println!(
            "[{}:{}:{}] {} (created: {})",
            self.context, self.module, self.r#type, self.name, self.created
        );
    }
}

/// Thêm một bản ghi kiến trúc mới. Chỉ insert, không upsert.
pub async fn add<S: Storage>(store: &S, mut new_entry: Entry) -> Result<Entry, Error> {
    new_entry.created = now();
    store.insert(new_entry.clone()).await?;
    Ok(new_entry)
}

/// Tìm một bản ghi kiến trúc bằng key.
pub async fn find<S: Storage>(store: &S, key: String) -> Result<Option<Entry>, Error> {
    store.fetch::<Entry>(key).await
}

/// Cập nhật một bản ghi kiến trúc bằng hàm biến đổi.
pub async fn change<S: Storage, F>(store: &S, key: String, transform: F) -> Result<Entry, Error>
where
    F: FnOnce(Entry) -> Entry + Send + 'static,
{
    store.update::<Entry, F>(key, transform).await
}

/// Xóa một bản ghi kiến trúc.
pub async fn remove<S: Storage>(store: &S, key: String) -> Result<Entry, Error> {
    store.delete::<Entry>(key).await
}

/// Truy vấn các bản ghi kiến trúc. Nhận repository::Query<Vec<u8>>
pub async fn query<S: Storage>(store: &S, query: Query<Vec<u8>>)
    -> Result<Box<dyn Iterator<Item = Result<Summary, Error>> + Send>, Error>
{
    store.query::<Entry>(query).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use repository::sled::Sled; // Sử dụng Sled làm backend test
    use tempfile::tempdir;
    use tokio::runtime::Runtime;

    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[test]
    // Kiểm tra tổng hợp các chức năng thêm và cập nhật (gốc: add_and_update)
    fn features() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let entry1 = Entry {
                id: Id::new_v4(),
                context: "Sys".to_string(), module: "Dir".to_string(), r#type: Kind::Agent, name: "Dir".to_string(),
                responsibility: "Coord".to_string(), dependency: "".to_string(), performance: "".to_string(), naming: "".to_string(),
                prompt: "".to_string(), created: 0, // Sẽ được ghi đè bởi add
            };

            let added = add(&store, entry1.clone()).await.unwrap();
            assert_eq!(added.context, "Sys");
            assert!(added.created != 0); // Đảm bảo created được gán

            let found = find(&store, added.key()).await.unwrap().unwrap();
            assert_eq!(found.responsibility, "Coord");

            let item = Entry {
                id: Id::new_v4(),
                responsibility: "NewCoord".to_string(), // Thay đổi responsibility
                ..entry1 // Giữ nguyên các trường khác
            };

            let updated = add(&store, item).await.unwrap();
            assert_eq!(updated.responsibility, "NewCoord");
            assert_eq!(updated.key(), added.key());

            let loaded = find(&store, added.key()).await.unwrap().unwrap();
            assert_eq!(loaded.responsibility, "NewCoord");
        });
    }

    #[test]
    fn clear() { // Đổi tên hàm để tránh trùng lặp với hàm `remove`
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let entry = Entry {
                id: Id::new_v4(),
                context: "Sys".to_string(), module: "Mod".to_string(), r#type: Kind::Agent, name: "Name".to_string(),
                responsibility: "Resp".to_string(), dependency: "".to_string(), performance: "".to_string(), naming: "".to_string(),
                prompt: "".to_string(), created: now(),
            };
            let added = add(&store, entry.clone()).await.unwrap();
            assert!(find(&store, added.key()).await.unwrap().is_some());

            let removed = remove(&store, added.key()).await.unwrap();
            assert_eq!(removed.name, entry.name);
            assert!(find(&store, added.key()).await.unwrap().is_none());
        });
    }

    #[test]
    fn list() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            for i in 0..5 {
                let entry = Entry {
                    id: Id::new_v4(),
                    context: "Test".to_string(), module: format!("Mod{}", i), r#type: Kind::Agent, name: format!("Item{}", i),
                    responsibility: "".to_string(), dependency: "".to_string(), performance: "".to_string(), naming: "".to_string(),
                    prompt: "".to_string(), created: now() + i as u128,
                };
                add(&store, entry).await.unwrap();
            }

            let results = query(&store, Query { prefix: Vec::new(), after: None, limit: 10 }).await.unwrap();
            let mut summaries: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(summaries.len(), 5);
            // Sắp xếp lại theo created giảm dần
            summaries.sort_by(|a, b| b.created.cmp(&a.created));
            // Kiểm tra thứ tự sắp xếp (mới nhất trước theo created timestamp)
            assert_eq!(summaries[0].module, "Mod4");
            assert_eq!(summaries[4].module, "Mod0");
        });
    }
}