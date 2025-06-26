//! Module quản lý các bản ghi bộ nhớ thông qua `memories` crate.

use repository::error::ValidationError;
use repository::{Error, Id, Storage};
pub use memories::{Entry, Kind, Summary};
use shared;
use shared::interaction::Command;
use shared::interaction::Interaction;
use tracing::info;

#[derive(Debug, Clone)]
pub struct Add {
    pub r#type: String, // Giữ nguyên String cho dữ liệu truyền từ CLI
    pub context: String,
    pub module: String,
    pub subject: String,
    pub description: String,
    pub decision: String,
    pub rationale: String,
    pub created: u128,
}

impl Command for Add {
    type Output = memories::Entry;
}

impl Add {
    pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();
        if self.subject.trim().is_empty() {
            errors.push(ValidationError {
                field: "subject".to_string(),
                message: "Chủ đề không được để trống.".to_string(),
            });
        }
        if self.subject.len() > 256 {
            errors.push(ValidationError {
                field: "subject".to_string(),
                message: "Chủ đề không được vượt quá 256 ký tự.".to_string(),
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
        if self.description.len() > 4096
            || self.decision.len() > 4096
            || self.rationale.len() > 4096
        {
            errors.push(ValidationError {
                field: "description".to_string(),
                message: "Mô tả, Quyết định, và Lý do không được vượt quá 4096 ký tự.".to_string(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub async fn add<S: Storage>(store: &S, interaction: Interaction<Add>) -> Result<memories::Entry, Error> {
    info!(interaction_id = %interaction.id, command = ?interaction.command, "Đang xử lý lệnh AddMemory");

    interaction.command.validate().map_err(Error::Validation)?;

    let result = memories::add(
        store,
        interaction.command.r#type,
        interaction.command.context,
        interaction.command.module,
        interaction.command.subject,
        interaction.command.description,
        interaction.command.decision,
        interaction.command.rationale,
    ).await;

    match &result {
        Ok(entry) => info!(interaction_id = %interaction.id, memory_id = %entry.id, "Hoàn thành xử lý AddMemory"),
        Err(e) => tracing::error!(interaction_id = %interaction.id, error = ?e, "Xử lý AddMemory thất bại"),
    }
    
    result
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
    kind: Option<String>,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<memories::Summary, repository::Error>> + Send>, repository::Error> {
    let prefix_vec = match kind {
        Some(s) => {
            let kind = Kind::try_from(s)?;
            vec![(&kind).into()]
        }
        None => Vec::new(),
    };
    let query = shared::query(prefix_vec, None::<Vec<u8>>, limit);
    memories::query(store, query).await
}

