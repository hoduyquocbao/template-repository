//! Triển khai Entity cho mô hình Task, sử dụng enum để tăng cường an toàn và hiệu suất.

use serde::{Deserialize, Serialize};
use repository::{error::Fault, Entity, Error, Id, Key, now, Query, Storage};
use shared::Showable;
use tracing::{info, instrument, warn};
use std::convert::TryFrom;

// --- Định nghĩa Enum cho Status và Priority ---

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Open,
    Pending,
    Done,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

// --- Implementations cho việc chuyển đổi ---

impl From<&Status> for u8 {
    fn from(status: &Status) -> u8 {
        match status {
            Status::Open => 0,
            Status::Pending => 1,
            Status::Done => 2,
        }
    }
}

impl TryFrom<String> for Status {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "open" => Ok(Status::Open),
            "inprogress" => Ok(Status::Pending),
            "done" => Ok(Status::Done),
            "wontfix" => Ok(Status::Pending),
            _ => Err(Error::Validation(vec![Fault {
                field: "status".to_string(),
                message: format!("Trạng thái '{}' không hợp lệ.", s),
            }])),
        }
    }
}

impl From<&Priority> for u8 {
    fn from(priority: &Priority) -> u8 {
        match priority {
            Priority::High => 0,
            Priority::Medium => 1,
            Priority::Low => 2,
        }
    }
}

impl TryFrom<String> for Priority {
    type Error = Error;
    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "low" => Ok(Priority::Low),
            "medium" => Ok(Priority::Medium),
            "high" => Ok(Priority::High),
            "urgent" => Ok(Priority::High),
            _ => Err(Error::Validation(vec![Fault {
                field: "priority".to_string(),
                message: format!("Ưu tiên '{}' không hợp lệ.", s),
            }])),
        }
    }
}

/// Đại diện cho một công việc với các thuộc tính chi tiết, sử dụng enum.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub id: Id,
    pub context: String,
    pub module: String,
    pub task: String,
    pub priority: Priority, // Sử dụng enum
    pub status: Status,     // Sử dụng enum
    pub assignee: String,
    pub due: String,
    pub notes: String,
    pub created: u128,
}

impl Entity for Entry {
    const NAME: &'static str = "tasks";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = Summary;
    
    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn index(&self) -> Self::Index {
        let mut key = Key::reserve(34); // status + priority + time + id
        key.byte((&self.status).into());      // Chuyển đổi hiệu suất cao
        key.byte((&self.priority).into());    // Chuyển đổi hiệu suất cao
        key.time(self.created);
        key.id(self.id);
        key.build()
    }
    
    fn summary(&self) -> Self::Summary {
        Summary {
            id: self.id,
            priority: self.priority.clone(),
            status: self.status.clone(),
            task: self.task.clone(),
        }
    }
}

/// Một bản tóm tắt của `Entry` để hiển thị trong danh sách.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: Id,
    pub priority: Priority,
    pub status: Status,
    pub task: String,
}

impl Showable for Summary {
    fn show(&self) {
        println!("[{}] P:{:?} S:{:?} - {}", self.id, self.priority, self.status, self.task);
    }
}

/// Đại diện cho một bản vá (thay đổi một phần) cho một Entry.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Patch {
    pub context: Option<String>,
    pub module: Option<String>,
    pub task: Option<String>,
    pub priority: Option<Priority>,
    pub status: Option<Status>,
    pub assignee: Option<String>,
    pub due: Option<String>,
    pub notes: Option<String>,
}

/// Thêm một công việc mới vào hệ thống lưu trữ.
#[instrument(skip(store))]
#[allow(clippy::too_many_arguments)]
pub async fn add<S: Storage>(
    store: &S,
    context: String,
    module: String,
    task_desc: String,
    priority: Priority,
    status: Status,
    assignee: String,
    due: String,
    notes: String,
) -> Result<Entry, Error> {
    info!(task = %task_desc, "Đang thêm công việc mới");
    if task_desc.is_empty() {
        warn!("Cố gắng thêm công việc với nội dung rỗng");
        return Err(Error::Validation(vec![Fault {
            field: "task".to_string(),
            message: "Mô tả công việc không được để trống.".to_string(),
        }]));
    }
    
    let task = Entry {
        id: Id::new_v4(),
        context,
        module,
        task: task_desc,
        priority,
        status,
        assignee,
        due,
        notes,
        created: now(),
    };
    
    let result = task.clone();
    store.insert(task).await?;
    info!(id = %result.id, "Thêm công việc thành công");
    Ok(result)
}

/// Tìm một công việc bằng ID của nó.
#[instrument(skip(store))]
pub async fn find<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    info!(%id, "Đang tìm công việc theo ID");
    store.fetch::<Entry>(id).await?.ok_or(Error::Missing)
}

/// Cập nhật một công việc bằng một giao dịch nguyên tử.
#[instrument(skip(store, patch))]
pub async fn change<S: Storage>(store: &S, id: Id, patch: Patch) -> Result<Entry, Error> {
    info!(%id, ?patch, "Đang cập nhật công việc");
    
    // Kiểm tra lỗi đầu vào
    if let Some(ref task) = patch.task {
        if task.trim().is_empty() {
            warn!(%id, "Cố gắng cập nhật công việc với nội dung rỗng");
            return Err(Error::Validation(vec![Fault {
                field: "task".to_string(),
                message: "Mô tả công việc không được để trống.".to_string(),
            }]));
        }
    }
    
    store.update::<Entry, _>(id, move |mut task| {
        if let Some(val) = patch.context { task.context = val; }
        if let Some(val) = patch.module { task.module = val; }
        if let Some(val) = patch.task { task.task = val; }
        if let Some(val) = patch.priority { task.priority = val; }
        if let Some(val) = patch.status { task.status = val; }
        if let Some(val) = patch.assignee { task.assignee = val; }
        if let Some(val) = patch.due { task.due = val; }
        if let Some(val) = patch.notes { task.notes = val; }
        task
    }).await
}

/// Xóa một công việc khỏi kho lưu trữ.
#[instrument(skip(store))]
pub async fn remove<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    info!(%id, "Đang xóa công việc");
    store.delete::<Entry>(id).await
}

/// Truy vấn một danh sách tóm tắt các công việc.
#[instrument(skip(store, query))]
pub async fn query<S: Storage>(store: &S, query: Query<Vec<u8>>)
    -> Result<Box<dyn Iterator<Item = Result<Summary, Error>> + Send>, Error> 
{
    info!(?query, "Đang truy vấn danh sách công việc");
    store.query::<Entry>(query).await
}

/// Chèn một iterator các công việc theo từng lô.
#[instrument(skip(store, iter))]
pub async fn bulk<S: Storage>(store: &S, iter: impl Iterator<Item = Entry> + Send + 'static) -> Result<(), Error> {
    info!("Đang chèn hàng loạt công việc");
    store.mass::<Entry>(Box::new(iter)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use repository::sled::Sled;
    use tokio::runtime::Runtime;
    use tempfile::tempdir;

    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[test]
    // Kiểm tra tổng hợp các chức năng chính: thêm, tìm, ... (gốc: add_and_find_task)
    fn features() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(
                &store, "ctx".into(), "mdl".into(), "Test task".into(), 
                Priority::High, Status::Open, "Guardian".into(), "".into(), "".into()
            ).await.unwrap();

            let found = find(&store, added.id).await.unwrap();
            assert_eq!(added, found);
            assert_eq!(found.priority, Priority::High);
        });
    }
    
    #[test]
    // Kiểm tra cập nhật trạng thái task (gốc: change_task_status)
    fn update() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(
                &store, "ctx".into(), "mdl".into(), "Test task".into(), 
                Priority::High, Status::Open, "Guardian".into(), "".into(), "".into()
            ).await.unwrap();

            let patch = Patch { status: Some(Status::Done), ..Default::default() };
            let updated = change(&store, added.id, patch).await.unwrap();
            assert_eq!(updated.status, Status::Done);
        });
    }
    
    #[test]
    // Kiểm tra truy vấn/lọc theo trạng thái và độ ưu tiên (gốc: query_by_status_and_priority)
    fn filter() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            // Add tasks with different statuses and priorities
            add(&store, "".into(), "".into(), "High Open".into(), Priority::High, Status::Open, "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "Med Open".into(), Priority::Medium, Status::Open, "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "High Done".into(), Priority::High, Status::Done, "".into(), "".into(), "".into()).await.unwrap();

            // Query for Open tasks
            let open = vec![(&Status::Open).into()];
            let val = shared::query(open, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, val).await.unwrap().collect::<Result<_,_>>().unwrap();
            let mut result: Vec<_> = results.into_iter().filter(|t| t.status == Status::Open).collect();
            result.sort_by_key(|t| match t.priority { Priority::High => 0, Priority::Medium => 1, Priority::Low => 2 });
            println!("DEBUG: Query Open tasks, got {} results:", result.len());
            for t in &result {
                println!("  - task: {} | status: {:?} | priority: {:?}", t.task, t.status, t.priority);
            }
            assert_eq!(result.len(), 2);
            // Check sorting: High priority should come first
            assert_eq!(result[0].task, "High Open");
            assert_eq!(result[1].task, "Med Open");

            // Query for Open, High-Priority tasks
            let high = vec![(&Status::Open).into(), (&Priority::High).into()];
            let val = shared::query(high, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, val).await.unwrap().collect::<Result<_,_>>().unwrap();
            let output: Vec<_> = results.into_iter().filter(|t| t.status == Status::Open && t.priority == Priority::High).collect();
            assert_eq!(output.len(), 1);
            assert_eq!(output[0].task, "High Open");
        });
    }
}