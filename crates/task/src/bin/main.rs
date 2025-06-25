// main.rs
// Binary crate với CLI để tương tác với thư viện.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Id, Error, };
use tracing::info;
use task::{Patch, Status, Priority, Summary};

/// Một ứng dụng task hiệu năng cao, giới hạn bởi quy tắc đơn từ.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Thêm một công việc mới
    Add { task: String },
    /// Lấy một công việc bằng ID
    Get { id: Id },
    /// Đánh dấu một công việc là đã hoàn thành
    Done { id: Id },
    /// Xóa một công việc
    Remove { id: Id },
    /// Liệt kê các công việc với bộ lọc trạng thái
    List {
        /// Chỉ hiển thị các công việc đã hoàn thành
        #[arg(long)]
        status: bool,

        /// Chỉ hiển thị các công việc đang chờ
        #[arg(long, conflicts_with = "status")]
        pending: bool,

        /// Số lượng tối đa hiển thị
        #[arg(short, long, default_value = "10")]
        limit: usize,
    }
}

/// Hàm trợ giúp để in một danh sách các công việc từ một iterator
fn print<I>(iter: I) -> Result<(), Error> 
where
    I: Iterator<Item = Result<Summary, Error>>
{
    let mut count = 0;
    for result in iter {
        match result {
            Ok(summary) => {
                println!("[{}] {}", summary.id, summary.task);
                count += 1;
            }
            Err(e) => return Err(e),
        }
    }
    if count == 0 {
        println!("No matching tasks found.");
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), repository::Error> {
    // Khởi tạo tracing subscriber ở đầu chương trình
    tracing_subscriber::fmt::init();
    
    info!("Đang khởi động ứng dụng repository");
    
    let cli = Cli::parse();
    let store = Sled::new("db")?;

    match cli.command {
        Some(Commands::Add { task }) => {
            info!(task = %task, "Đang xử lý lệnh thêm mới");
            let status_enum = Status::try_from("Pending".to_string())?;
            let priority_enum = Priority::try_from("Medium".to_string())?;
            let task = task::add(&store, "".to_string(), "".to_string(), task, priority_enum, status_enum, "".to_string(), "".to_string(), "".to_string()).await?;
            println!("Đã thêm: [{}], {}", task.id, task.task);
        }
        Some(Commands::Get { id }) => {
            info!(%id, "Đang xử lý lệnh lấy");
            let task = task::find(&store, id).await?;
            let status = match task.status {
                Status::Done => "hoàn thành",
                Status::Pending => "đang chờ",
                Status::Open => "mở",
            };
            println!("[{}] {} ({})", task.id, task.task, status);
        }
        Some(Commands::Done { id }) => {
            info!(%id, "Đang xử lý lệnh hoàn thành");
            let patch = Patch { status: Some(Status::Done), ..Default::default() };
            let task = task::change(&store, id, patch).await?;
            println!("Đã hoàn thành: [{}], {}", task.id, task.task);
        }
        Some(Commands::Remove { id }) => {
            info!(%id, "Đang xử lý lệnh xóa");
            let task = task::remove(&store, id).await?;
            println!("Đã xóa: [{}], {}", task.id, task.task);
        }
        Some(Commands::List { status, pending, limit }) => {
            info!(status = %status, pending = %pending, limit = %limit, "Đang xử lý lệnh liệt kê");
            let status_enum = if status {
                Status::Done
            } else if pending || !status {
                Status::Pending
            } else {
                return Err(Error::Input);
            };
            let title = if status { "Đã hoàn thành" } else { "Đang chờ" };
            println!("--- Các công việc {} (Tóm tắt) ---", title);
            let prefix = vec![(&status_enum).into()];
            let query_obj = shared::query(prefix, None::<Vec<u8>>, limit);
            let result = task::query(&store, query_obj).await?;
            print(result)?;
            println!("----------------------------");
        }
        None => {
            info!("Không có lệnh được chỉ định, hiển thị tin nhắn chào mừng");
            println!("Chào mừng đến với repository. Sử dụng `list --pending` hoặc `list --status` để bắt đầu.");
        }
    }

    info!("Ứng dụng repository hoàn thành thành công");
    Ok(())
}