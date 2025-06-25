//! Triển khai Entity cho mô hình Task, sử dụng enum để tăng cường an toàn và hiệu suất.

use serde::{Deserialize, Serialize};
use repository::{Storage, Id, Error, Entity, Key, now, Query};
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
        match s.as_str() {
            "Open" => Ok(Status::Open),
            "Pending" => Ok(Status::Pending),
            "Done" => Ok(Status::Done),
            _ => Err(Error::Input),
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
        match s.as_str() {
            "High" => Ok(Priority::High),
            "Medium" => Ok(Priority::Medium),
            "Low" => Ok(Priority::Low),
            _ => Err(Error::Input),
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
        return Err(Error::Input);
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
    if let Some(text) = &patch.task {
        if text.is_empty() {
            warn!(%id, "Cố gắng cập nhật công việc với nội dung rỗng");
            return Err(Error::Input);
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
    use shared;

    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[test]
    fn add_and_find_task() {
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
    fn change_task_status() {
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
    fn query_by_status_and_priority() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            // Add tasks with different statuses and priorities
            add(&store, "".into(), "".into(), "High Open".into(), Priority::High, Status::Open, "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "Med Open".into(), Priority::Medium, Status::Open, "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "High Done".into(), Priority::High, Status::Done, "".into(), "".into(), "".into()).await.unwrap();

            // Query for Open tasks
            let open_prefix = vec![(&Status::Open).into()];
            let query_obj = shared::query(open_prefix, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, query_obj).await.unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(results.len(), 2);
            // Check sorting: High priority should come first
            assert_eq!(results[0].task, "High Open");
            assert_eq!(results[1].task, "Med Open");

            // Query for Open, High-Priority tasks
            let high_open_prefix = vec![(&Status::Open).into(), (&Priority::High).into()];
            let query_obj = shared::query(high_open_prefix, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, query_obj).await.unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].task, "High Open");
        });
    }
}