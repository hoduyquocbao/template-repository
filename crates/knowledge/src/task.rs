//! Module quản lý các bản ghi công việc (todo) thông qua `todo` crate.

use repository::error::ValidationError;
use repository::{Error, Id, Query, Storage};
pub use task::{Entry, Patch, Priority, Status, Summary};
use task;
use shared::interaction::Command;
use shared::interaction::Interaction;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Add {
    pub context: String,
    pub module: String,
    pub task: String,
    pub priority: Priority,
    pub status: Status,
    pub assignee: String,
    pub due: String,
    pub notes: String,
}

impl Command for Add {
    type Output = Entry;
}

impl Add {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if self.task.trim().is_empty() {
            errors.push(ValidationError {
                field: "task".to_string(),
                message: "Mô tả công việc không được để trống.".to_string(),
            });
        }
        if self.task.len() > 256 {
            errors.push(ValidationError {
                field: "task".to_string(),
                message: "Mô tả công việc không được vượt quá 256 ký tự.".to_string(),
            });
        }
        if self.context.len() > 64 {
            errors.push(ValidationError {
                field: "context".to_string(),
                message: "Ngữ cảnh không được vượt quá 64 ký tự.".to_string(),
            });
        }
        if self.module.len() > 64 {
            errors.push(ValidationError {
                field: "module".to_string(),
                message: "Module không được vượt quá 64 ký tự.".to_string(),
            });
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub async fn add<S: Storage>(store: &S, interaction: Interaction<Add>) -> Result<Entry, Error> {
    info!(interaction_id = %interaction.id, command = ?interaction.command, "Đang xử lý lệnh AddTask");
    
    // 1. Xác thực
    interaction.command.validate().map_err(Error::Validation)?;
    
    // 2. Gọi logic nghiệp vụ cốt lõi
    let result = task::add(
        store,
        interaction.command.context,
        interaction.command.module,
        interaction.command.task,
        interaction.command.priority,
        interaction.command.status,
        interaction.command.assignee,
        interaction.command.due,
        interaction.command.notes,
    ).await;

    // 3. Ghi nhật ký kết quả
    match &result {
        Ok(entry) => info!(interaction_id = %interaction.id, task_id = %entry.id, "Hoàn thành xử lý AddTask"),
        Err(e) => tracing::error!(interaction_id = %interaction.id, error = ?e, "Xử lý AddTask thất bại"),
    }
    
    result
}

/// Lấy một công việc bằng ID.
/// Mục đích: Cung cấp giao diện `get` cho `knowledge` CLI.
pub async fn get<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    task::find(store, id).await
}

/// Đánh dấu một công việc là đã hoàn thành.
/// Mục đích: Cung cấp giao diện `done` cho `knowledge` CLI.
pub async fn done<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    let patch = Patch { status: Some(Status::Done), ..Default::default() };
    task::change(store, id, patch).await
}

/// Xóa một công việc.
/// Mục đích: Cung cấp giao diện `del` cho `knowledge` CLI.
/// 
pub async fn del<S: Storage>(store: &S, id: Id) -> Result<Entry, Error> {
    task::remove(store, id).await
}

/// Liệt kê các công việc với bộ lọc trạng thái.
/// Mục đích: Cung cấp giao diện `list` cho `knowledge` CLI.
pub async fn list<S: Storage>(
    store: &S,
    query: Query<Vec<u8>>,
) -> Result<Vec<Summary>, Error> {
    let results = task::query(store, query).await?;
    results.collect()
}

/// Thay đổi một công việc.
pub async fn change<S: Storage>(store: &S, id: Id, patch: Patch) -> Result<Entry, Error> {
    task::change(store, id, patch).await
}