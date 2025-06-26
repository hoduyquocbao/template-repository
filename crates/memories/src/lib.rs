//! Crate này quản lý bộ nhớ và các quyết định của hệ thống.
//! Dữ liệu được lưu trữ thông qua `repository::Storage` để tăng hiệu suất.

use serde::{Deserialize, Serialize};
use repository::{error::ValidationError, now, Entity, Error, Id, Key, Query, Storage};
use shared::{Showable, Filterable};
use tracing::{info, warn};

use std::convert::TryFrom;

/// Enum đại diện cho loại bản ghi bộ nhớ.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Kind {
    Decision,
    Analysis,
    Lesson,
    Refactor,
    Other,
}

impl From<&Kind> for u8 {
    fn from(kind: &Kind) -> u8 {
        match kind {
            Kind::Decision => 0,
            Kind::Analysis => 1,
            Kind::Lesson => 2,
            Kind::Refactor => 3,
            Kind::Other => 255,
        }
    }
}

impl TryFrom<String> for Kind {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "decision" => Ok(Kind::Decision),
            "analysis" => Ok(Kind::Analysis),
            "lesson" => Ok(Kind::Lesson),
            "refactor" => Ok(Kind::Refactor),
            "other" => Ok(Kind::Other),
            _ => Err(Error::Validation(vec![ValidationError {
                field: "kind".to_string(),
                message: format!("Loại '{}' không hợp lệ.", s),
            }])),
        }
    }
}

/// Đại diện cho một bản ghi bộ nhớ.
/// Đây là một `Entity` có thể được lưu trữ và truy vấn thông qua `repository`.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Entry {
    pub id: Id,               // ID duy nhất cho bản ghi
    pub r#type: Kind,         // Loại bản ghi (Decision, Analysis, Lesson, etc.)
    pub context: String,      // Ngữ cảnh liên quan
    pub module: String,       // Module hoặc crate liên quan
    pub subject: String,      // Chủ đề chính
    pub description: String,  // Mô tả chi tiết
    pub decision: String,     // Quyết định được đưa ra
    pub rationale: String,    // Lý do đằng sau quyết định
    pub created: u128,        // Timestamp tạo (thay vì String để dễ index)
}


impl Entity for Entry {
    const NAME: &'static str = "memories"; // Tên tree trong Sled
    type Key = Id; // Key là ID duy nhất
    type Index = Vec<u8>; // Index để sắp xếp/truy vấn
    type Summary = Summary;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn index(&self) -> Self::Index {
        let mut key = Key::reserve(1 + 16 + 16); // type_byte + time + id
        key.byte((&self.r#type).into()); // Sử dụng phương thức chuyển đổi mới
        key.time(self.created);
        key.id(self.id);
        key.build()
    }

    fn summary(&self) -> Self::Summary {
        Summary {
            id: self.id,
            r#type: self.r#type.clone(),
            subject: self.subject.clone(),
            created: self.created,
        }
    }
}

impl Filterable for Entry {
    type Prefix = Vec<u8>;
    type After = Vec<u8>;
    fn prefix(&self) -> Self::Prefix {
        let mut key = Key::reserve(1 + 16 + 16);
        key.byte((&self.r#type).into());
        key.time(self.created);
        key.id(self.id);
        key.build()
    }
    fn after(&self) -> Option<Self::After> {
        None
    }
}

/// Một bản tóm tắt của `Entry` để hiển thị trong danh sách.
#[derive(Debug, Serialize, Deserialize)]
pub struct Summary {
    pub id: Id,
    pub r#type: Kind,
    pub subject: String,
    pub created: u128,
}

// Triển khai Showable cho Summary của memories
impl Showable for Summary {
    fn show(&self) {
        println!(
            "[{}] [{:?}]: {}",
            self.id, self.r#type, self.subject
        );
    }
}

/// Tạo và thêm một bản ghi bộ nhớ mới. created được tự động sinh.
#[allow(clippy::too_many_arguments)]
pub async fn add<S: Storage>(
    store: &S,
    kind_str: String,
    context: String,
    module: String,
    subject: String,
    description: String,
    decision: String,
    rationale: String,
) -> Result<Entry, Error> {
    info!(kind = %kind_str, context = %context, "Đang thêm bộ nhớ mới");
    let kind = Kind::try_from(kind_str)?; // Chuyển đổi và xác thực loại
    
    // Kiểm tra các trường bắt buộc
    if context.trim().is_empty() || module.trim().is_empty() || subject.trim().is_empty() {
        warn!("Thiếu thông tin bắt buộc khi thêm bộ nhớ");
        return Err(Error::Validation(vec![ValidationError {
            field: "context/module/subject".to_string(),
            message: "Context, Module, và Subject không được để trống.".to_string(),
        }]));
    }
    
    let entry = Entry {
        id: Id::new_v4(),
        r#type: kind, // Sử dụng enum đã được xác thực
        context,
        module,
        subject,
        description,
        decision,
        rationale,
        created: now(),
    };
    let result = entry.clone();
    store.insert(entry).await?;
    Ok(result)
}

/// Tìm một bản ghi bộ nhớ bằng ID.
pub async fn find<S: Storage>(store: &S, id: Id) -> Result<Option<Entry>, Error> {
    store.fetch::<Entry>(id).await
}

/// Cập nhật một bản ghi bộ nhớ bằng hàm biến đổi.
/// (Thường ít dùng cho memory logs, nhưng có sẵn để hoàn thiện)
pub async fn change<S: Storage, F>(store: &S, id: Id, transform: F) -> Result<Entry, Error>
where
    F: FnOnce(Entry) -> Entry + Send + 'static,
{
    store.update::<Entry, F>(id, transform).await
}

/// Xóa một bản ghi bộ nhớ.
/// (Thường ít dùng cho memory logs, nhưng có sẵn để hoàn thiện)
pub async fn remove<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    store.delete::<Entry>(id).await
}

/// Truy vấn các bản ghi bộ nhớ. Nhận repository::Query<Vec<u8>>
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
    // use repository::now; // Import `now` chỉ trong scope test

    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[test]
    fn add_and_find() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(
                &store,
                "Decision".to_string(),
                "System".to_string(),
                "Director".to_string(),
                "Naming".to_string(),
                "Use single words".to_string(),
                "Standard".to_string(),
                "Clarity".to_string(),
            ).await.unwrap();

            let found = find(&store, added.id).await.unwrap().unwrap();
            assert_eq!(found.subject, "Naming");
            assert_eq!(found.id, added.id);
            assert_eq!(found.r#type, Kind::Decision);
        });
    }

    #[test]
    fn query_summaries() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            for i in 0..5 {
                let r#type = if i % 2 == 0 { "Decision" } else { "Analysis" }.to_string();
                add(
                    &store,
                    r#type,
                    "Context".to_string(),
                    "Module".to_string(),
                    format!("Subject{}", i),
                    "Desc".to_string(),
                    "Dec".to_string(),
                    "Rat".to_string(),
                ).await.unwrap();
            }

            let all_results = query(&store, Query { prefix: Vec::new(), after: None, limit: 10 }).await.unwrap();
            let mut summaries: Vec<_> = all_results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(summaries.len(), 5);
            summaries.sort_by(|a, b| b.created.cmp(&a.created));
            assert_eq!(summaries[0].subject, "Subject4");
            assert_eq!(summaries[4].subject, "Subject0");
            // Kiểm tra đúng loại
            assert_eq!(summaries[0].r#type, Kind::Decision);
            assert_eq!(summaries[1].r#type, Kind::Analysis);
        });
    }
}