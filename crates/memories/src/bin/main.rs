// main.rs
// Binary crate với CLI để tương tác với thư viện memories.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Id, Error};
use tracing::info;
use memories::{self, Summary}; // Import các thành phần cần thiết

/// Một ứng dụng quản lý bộ nhớ hiệu năng cao.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Thêm một bản ghi bộ nhớ mới
    Add {
        #[arg(long)]
        r#type: String, // 'type' là từ khóa, dùng r#type
        #[arg(long)]
        context: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        subject: String,
        #[arg(long)]
        description: String,
        #[arg(long)]
        decision: String,
        #[arg(long)]
        rationale: String,
    },
    /// Lấy một bản ghi bộ nhớ bằng ID
    Get {
        #[arg(long)]
        id: Id,
    },
    /// Liệt kê các bản ghi bộ nhớ
    List {
        /// Lọc theo loại (ví dụ: 'Decision', 'Analysis')
        #[arg(long)]
        r#type: Option<String>,
        /// Số lượng tối đa hiển thị
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

/// Hàm trợ giúp để in một danh sách các bản tóm tắt
fn print<I>(iter: I) -> Result<(), Error>
where
    I: Iterator<Item = Result<Summary, Error>>,
{
    let mut count = 0;
    for result in iter {
        match result {
            Ok(summary) => {
                println!(
                    "[{}] [{:?}]: {}",
                    summary.id, summary.r#type, summary.subject
                );
                count += 1;
            }
            Err(e) => return Err(e),
        }
    }
    if count == 0 {
        println!("Không tìm thấy bản ghi nào.");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), repository::Error> {
    tracing_subscriber::fmt::init();

    info!("Đang khởi động ứng dụng memories");

    let cli = Cli::parse();
    let store = Sled::new("mem_db")?; // Sử dụng một Sled DB riêng cho bộ nhớ

    match cli.command {
        Some(Commands::Add {
            r#type,
            context,
            module,
            subject,
            description,
            decision,
            rationale,
        }) => {
            info!(
                %r#type, %subject, "Đang xử lý lệnh thêm bản ghi bộ nhớ mới"
            );
            let entry = memories::add(
                &store,
                r#type,
                context,
                module,
                subject,
                description,
                decision,
                rationale,
            ).await?;
            println!("[{}] [{:?}]: {}", entry.id, entry.r#type, entry.subject);
        }
        Some(Commands::Get { id }) => {
            info!(%id, "Đang xử lý lệnh lấy bản ghi bộ nhớ");
            match memories::find(&store, id).await? {
                Some(entry) => {
                    println!("ID: {}", entry.id);
                    println!("Type: {:?}", entry.r#type);
                    println!("Context: {}", entry.context);
                    println!("Module: {}", entry.module);
                    println!("Subject: {}", entry.subject);
                    println!("Description: {}", entry.description);
                    println!("Decision: {}", entry.decision);
                    println!("Rationale: {}", entry.rationale);
                    println!("Created: {}", entry.created);
                }
                None => {
                    println!("Không tìm thấy bản ghi với ID: {}", id);
                }
            }
        }
        Some(Commands::List { r#type, limit }) => {
            let prefix_vec = match r#type {
                Some(kind) => {
                    let kind = memories::Kind::try_from(kind)?;
                    vec![(&kind).into()]
                }
                None => Vec::new(),
            };
            info!(prefix = ?prefix_vec, "Đang xử lý lệnh liệt kê bản ghi bộ nhớ");
            let query = shared::query(prefix_vec, None::<Vec<u8>>, limit);
            let result = memories::query(&store, query).await?;
            print(result)?;
        }
        None => {
            info!("Không có lệnh được chỉ định, hiển thị tin nhắn chào mừng");
            println!("Chào mừng đến với ứng dụng memories. Sử dụng `add`, `get`, `list`.");
        }
    }

    info!("Ứng dụng memories hoàn thành thành công");
    Ok(())
} 