// main.rs
// Binary crate với CLI để tương tác với thư viện.

use clap::{Parser, Subcommand};
use bedrock::{self, Sled, Id, Patch, Error, todo::{self, Summary}};
use tracing::info;

/// Một ứng dụng todo hiệu năng cao, giới hạn bởi quy tắc đơn từ.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Thêm một công việc mới
    Add { text: String },
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
        done: bool,

        /// Chỉ hiển thị các công việc đang chờ
        #[arg(long, conflicts_with = "done")]
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
                println!("[{}] {}", summary.id, summary.text);
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
async fn main() -> Result<(), bedrock::Error> {
    // Khởi tạo tracing subscriber ở đầu chương trình
    tracing_subscriber::fmt::init();
    
    info!("Đang khởi động ứng dụng bedrock");
    
    let cli = Cli::parse();
    let store = Sled::new("db")?;

    match cli.command {
        Some(Commands::Add { text }) => {
            info!(text = %text, "Đang xử lý lệnh thêm mới");
            let task = todo::add(&store, text).await?;
            println!("Đã thêm: [{}], {}", task.id, task.text);
        }
        Some(Commands::Get { id }) => {
            info!(%id, "Đang xử lý lệnh lấy");
            let task = todo::find(&store, id).await?;
            let status = if task.done { "hoàn thành" } else { "đang chờ" };
            println!("[{}] {} ({})", task.id, task.text, status);
        }
        Some(Commands::Done { id }) => {
            info!(%id, "Đang xử lý lệnh hoàn thành");
            let patch = Patch {
                text: None,
                done: Some(true),
            };
            let task = todo::change(&store, id, patch).await?;
            println!("Đã hoàn thành: [{}], {}", task.id, task.text);
        }
        Some(Commands::Remove { id }) => {
            info!(%id, "Đang xử lý lệnh xóa");
            let task = todo::remove(&store, id).await?;
            println!("Đã xóa: [{}], {}", task.id, task.text);
        }
        Some(Commands::List { done, pending, limit }) => {
            info!(done = %done, pending = %pending, limit = %limit, "Đang xử lý lệnh liệt kê");
            
            // Xác định trạng thái cần truy vấn. Mặc định là 'pending' nếu không có cờ nào.
            let status = if done {
                true
            } else if pending || (!done && !pending) {
                // mặc định là 'pending' nếu không có cờ nào được đặt
                false
            } else {
                return Err(Error::Input); // Không bao giờ xảy ra nhờ conflicts_with
            };

            let title = if status { "Đã hoàn thành" } else { "Đang chờ" };
            println!("--- Các công việc {} (Tóm tắt) ---", title);
            
            // Sử dụng hàm query mới với status, after, limit
            let result = todo::query(&store, status, None, limit).await?;
            
            print(result)?;
            println!("----------------------------");
        }
        None => {
            info!("Không có lệnh được chỉ định, hiển thị tin nhắn chào mừng");
            println!("Chào mừng đến với bedrock. Sử dụng `list --pending` hoặc `list --done` để bắt đầu.");
        }
    }

    info!("Ứng dụng bedrock hoàn thành thành công");
    Ok(())
}