//! Module quản lý các bản ghi kiến trúc thông qua `architecture` crate.

use repository::error::Fault;
use repository::{Error, Storage};
use architecture::{self, Entry}; // Chỉ import Arch, không import Summary hay đổi tên
use shared;
use shared::interaction::{Command, Interaction};
use tracing::info;

pub use architecture::Kind;

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

impl Command for Add {
    type Output = architecture::Entry;
}

impl Add {
    pub fn validate(&self) -> Result<(), Vec<Fault>> {
        let mut errors = Vec::new();
        if self.name.trim().is_empty() {
            errors.push(Fault {
                field: "name".to_string(),
                message: "Tên không được để trống.".to_string(),
            });
        }
        if self.name.len() > 128 {
            errors.push(Fault {
                field: "name".to_string(),
                message: "Tên không được vượt quá 128 ký tự.".to_string(),
            });
        }
        if self.context.len() > 64 || self.module.len() > 64 {
            errors.push(Fault {
                field: "context/module".to_string(),
                message: "Ngữ cảnh và module không được vượt quá 64 ký tự.".to_string(),
            });
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

pub async fn add<S: Storage>(
    store: &S,
    interaction: Interaction<Add>,
) -> Result<architecture::Entry, Error> {
    info!(interaction_id = %interaction.id, command = ?interaction.command, "Đang xử lý lệnh AddArchitecture");

    interaction.command.validate().map_err(Error::Validation)?;

    let kind = architecture::Kind::try_from(interaction.command.r#type)?;
    let entry = architecture::Entry {
        id: repository::Id::new_v4(),
        context: interaction.command.context,
        module: interaction.command.module,
        r#type: kind,
        name: interaction.command.name,
        responsibility: interaction.command.responsibility,
        dependency: interaction.command.dependency,
        performance: interaction.command.performance,
        naming: interaction.command.naming,
        prompt: interaction.command.prompt,
        created: interaction.command.created,
    };

    let result = architecture::add(store, entry).await;

    match &result {
        Ok(entry) => info!(interaction_id = %interaction.id, architecture_id = %entry.id, "Hoàn thành xử lý Add Architecture"),
        Err(e) => tracing::error!(interaction_id = %interaction.id, error = ?e, "Xử lý AddArchitecture thất bại"),
    }

    result
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
    r#type: Option<String>,
    context: Option<String>,
    module: Option<String>,
    limit: usize,
) -> Result<Box<dyn Iterator<Item = Result<architecture::Summary, repository::Error>> + Send>, repository::Error> {
    info!(r#type = ?r#type, context = ?context, module = ?module, limit = limit, "Đang thực hiện architecture list query");
    
    let mut prefix = Vec::new();
    if let Some(type_str) = r#type {
        let kind = architecture::Kind::try_from(type_str)?;
        prefix.push((&kind).into());
        if let Some(ctx_str) = context {
            prefix.extend_from_slice(ctx_str.as_bytes());
            prefix.push(0); // Dấu phân cách
            if let Some(mod_str) = module {
                prefix.extend_from_slice(mod_str.as_bytes());
            }
        }
    }
    
    info!(prefix_len = prefix.len(), "Query prefix: {:?}", prefix);
    
    let query = shared::query(prefix, None::<Vec<u8>>, limit);
    let result = architecture::query(store, query).await;
    
    match &result {
        Ok(_) => info!("Architecture list query thành công"),
        Err(e) => tracing::error!(error = ?e, "Architecture list query thất bại"),
    }
    
    result
}

