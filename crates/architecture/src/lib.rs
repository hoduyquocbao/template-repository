//! Crate này quản lý cấu trúc kiến trúc của hệ thống.
//! Dữ liệu được lưu trữ thông qua `repository::Storage` để tăng hiệu suất.

use serde::{Deserialize, Serialize};
use repository::{Entity, Storage, Error, Key, now};
use shared::{Showable, Filterable};

/// Đại diện cho một bản ghi kiến trúc.
/// Đây là một `Entity` có thể được lưu trữ và truy vấn thông qua `repository`.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Entry {
    pub context: String,      // Ngữ cảnh (Bounded Context)
    pub module: String,       // Module hoặc crate
    pub r#type: String,       // Loại thành phần (Agent, Module, Trait, etc.)
    pub name: String,         // Tên định danh
    pub responsibility: String, // Trách nhiệm chính
    pub dependency: String,   // Phụ thuộc
    pub performance: String,  // Ghi chú hiệu suất
    pub naming: String,       // Lý do đặt tên
    pub prompt: String,       // Tham chiếu đến prompt system (ví dụ: "DirectorPrompt.md")
    pub created: u128,        // Timestamp tạo
}

impl Entity for Entry {
    const NAME: &'static str = "architecture"; // Tên tree trong Sled
    type Key = String; // Key duy nhất là sự kết hợp của các trường
    type Index = Vec<u8>; // Index để sắp xếp/truy vấn
    type Summary = Summary;

    fn key(&self) -> Self::Key {
        // Tạo key tổng hợp duy nhất
        format!("{}:{}:{}:{}", self.context, self.module, self.r#type, self.name)
    }

    fn index(&self) -> Self::Index {
        // Sử dụng Key builder để tạo index cho truy vấn hiệu quả
        let mut key = Key::reserve(Entity::key(self).len() + 16);
        key.byte(1); // Flag cho bản ghi sống (không bị xóa logic)
        key.time(self.created); // Sắp xếp theo thời gian tạo (mới nhất trước)
        key.byte(self.r#type.as_bytes()[0]); // Ví dụ: index theo loại
        key.build()
    }

    fn summary(&self) -> Self::Summary {
        Summary {
            context: self.context.clone(),
            module: self.module.clone(),
            name: self.name.clone(),
            r#type: self.r#type.clone(),
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
        key.byte(self.r#type.as_bytes()[0]);
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
    pub r#type: String,
}

// Triển khai Showable cho Summary của architecture
impl Showable for Summary {
    fn show(&self) {
        println!(
            "[{}:{}:{}] {}",
            self.context, self.module, self.r#type, self.name
        );
    }
}

/// Thêm một bản ghi kiến trúc mới. Nếu key đã tồn tại, nó sẽ cập nhật (upsert).
pub async fn add<S: Storage>(store: &S, new_entry: Entry) -> Result<Entry, Error> {
    let key = Entity::key(&new_entry);
    // Sửa lỗi move: Clone `new_entry` để `update_data` được move vào closure,
    // trong khi `new_entry` gốc vẫn có thể được sử dụng sau đó.
    let update_data = new_entry.clone();

    // Thử cập nhật, nếu không tồn tại thì insert.
    // `key` được move vào hàm `update`.
    let result = store.update::<Entry, _>(key, move |mut entry| {
        // Đây là logic khi update: giữ lại created, cập nhật các trường khác
        entry.responsibility = update_data.responsibility;
        entry.dependency = update_data.dependency;
        entry.performance = update_data.performance;
        entry.naming = update_data.naming;
        entry.prompt = update_data.prompt;
        entry // Trả về entry đã update
    }).await;

    match result {
        Ok(entry) => Ok(entry), // Đã cập nhật thành công
        Err(Error::Missing) => {
            // Không tìm thấy, nên insert mới. `new_entry` vẫn hợp lệ ở đây.
            let final_entry = Entry { created: now(), ..new_entry };
            // Sửa lỗi logic: `insert` không trả về gì, nên ta clone `final_entry` để insert
            // và trả về `final_entry` gốc.
            store.insert(final_entry.clone()).await?;
            Ok(final_entry)
        },
        Err(e) => Err(e), // Lỗi khác
    }
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

/// Truy vấn các bản ghi kiến trúc.
pub async fn query<S: Storage>(store: &S, prefix: Vec<u8>, after: Option<Vec<u8>>, limit: usize)
    -> Result<Box<dyn Iterator<Item = Result<Summary, Error>> + Send>, Error>
{
    let query_obj = shared::query(prefix, after, limit);
    store.query::<Entry>(query_obj).await
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
    fn add_and_update() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let entry1 = Entry {
                context: "Sys".to_string(), module: "Dir".to_string(), r#type: "Agent".to_string(), name: "Dir".to_string(),
                responsibility: "Coord".to_string(), dependency: "".to_string(), performance: "".to_string(), naming: "".to_string(),
                prompt: "".to_string(), created: 0, // Sẽ được ghi đè bởi add
            };

            let added = add(&store, entry1.clone()).await.unwrap();
            assert_eq!(added.context, "Sys");
            assert!(added.created != 0); // Đảm bảo created được gán

            let found = find(&store, added.key()).await.unwrap().unwrap();
            assert_eq!(found.responsibility, "Coord");

            let entry_updated = Entry {
                responsibility: "NewCoord".to_string(), // Thay đổi responsibility
                ..entry1 // Giữ nguyên các trường khác
            };

            let updated = add(&store, entry_updated).await.unwrap(); // Sử dụng add để upsert
            assert_eq!(updated.responsibility, "NewCoord");
            assert_eq!(updated.key(), added.key()); // Key không đổi

            let loaded = find(&store, added.key()).await.unwrap().unwrap();
            assert_eq!(loaded.responsibility, "NewCoord"); // Xác nhận đã update
        });
    }

    #[test]
    fn remove_test() { // Đổi tên hàm để tránh trùng lặp với hàm `remove`
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let entry = Entry {
                context: "Sys".to_string(), module: "Mod".to_string(), r#type: "Type".to_string(), name: "Name".to_string(),
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
    fn query_summaries() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            for i in 0..5 {
                let entry = Entry {
                    context: "Test".to_string(), module: format!("Mod{}", i), r#type: "Comp".to_string(), name: format!("Item{}", i),
                    responsibility: "".to_string(), dependency: "".to_string(), performance: "".to_string(), naming: "".to_string(),
                    prompt: "".to_string(), created: now() + i as u128,
                };
                add(&store, entry).await.unwrap();
            }

            let results = query(&store, Vec::new(), None, 10).await.unwrap();
            let summaries: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(summaries.len(), 5);
            // Kiểm tra thứ tự sắp xếp (mới nhất trước theo created timestamp)
            assert_eq!(summaries[0].module, "Mod4");
            assert_eq!(summaries[4].module, "Mod0");
        });
    }
}