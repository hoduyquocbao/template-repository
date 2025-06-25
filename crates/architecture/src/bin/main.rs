// main.rs
// Binary crate với CLI để tương tác với thư viện kiến trúc.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Error};
use tracing::info;
use architecture::{self, Entry, Summary}; // Import các thành phần cần thiết

/// Một ứng dụng quản lý kiến trúc hiệu năng cao.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Thêm hoặc cập nhật một bản ghi kiến trúc
    Add {
        #[arg(long)]
        context: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        r#type: String, // 'type' là từ khóa, dùng r#type
        #[arg(long)]
        name: String,
        #[arg(long)]
        responsibility: String,
        #[arg(long)]
        dependency: String,
        #[arg(long)]
        performance: String,
        #[arg(long)]
        naming: String,
        #[arg(long)]
        prompt: String,
    },
    /// Lấy một bản ghi kiến trúc bằng key tổng hợp
    Get {
        #[arg(long)]
        context: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        r#type: String,
        #[arg(long)]
        name: String,
    },
    /// Xóa một bản ghi kiến trúc
    Remove {
        #[arg(long)]
        context: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        r#type: String,
        #[arg(long)]
        name: String,
    },
    /// Liệt kê các bản ghi kiến trúc
    List {
        /// Tiền tố để lọc (ví dụ: "System:Director")
        #[arg(long, default_value = "")]
        prefix: String,
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
                    "[{}:{}:{}] {}",
                    summary.context, summary.module, summary.r#type, summary.name
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

    info!("Đang khởi động ứng dụng architecture");

    let cli = Cli::parse();
    let store = Sled::new("arch_db")?; // Sử dụng một Sled DB riêng cho kiến trúc

    match cli.command {
        Some(Commands::Add {
            context,
            module,
            r#type,
            name,
            responsibility,
            dependency,
            performance,
            naming,
            prompt,
        }) => {
            info!(
                %context, %module, %r#type, %name, "Đang xử lý lệnh thêm/cập nhật bản ghi kiến trúc"
            );
            let entry = Entry {
                context,
                module,
                r#type,
                name,
                responsibility,
                dependency,
                performance,
                naming,
                prompt,
                created: repository::now(), // Sử dụng now() từ repository
            };
            architecture::add(&store, entry.clone()).await?;
            println!("Đã thêm/cập nhật: [{}:{}:{}] {}", entry.context, entry.module, entry.r#type, entry.name);
        }
        Some(Commands::Get {
            context,
            module,
            r#type,
            name,
        }) => {
            let key = format!("{}:{}:{}:{}", context, module, r#type, name);
            info!(%key, "Đang xử lý lệnh lấy bản ghi kiến trúc");
            match architecture::find(&store, key.clone()).await? {
                Some(entry) => {
                    println!("Context: {}", entry.context);
                    println!("Module: {}", entry.module);
                    println!("Type: {}", entry.r#type);
                    println!("Name: {}", entry.name);
                    println!("Responsibility: {}", entry.responsibility);
                    println!("Dependency: {}", entry.dependency);
                    println!("Performance: {}", entry.performance);
                    println!("Naming: {}", entry.naming);
                    println!("Prompt: {}", entry.prompt);
                    println!("Created: {}", entry.created);
                }
                None => {
                    println!("Không tìm thấy bản ghi với key: {}", key);
                }
            }
        }
        Some(Commands::Remove {
            context,
            module,
            r#type,
            name,
        }) => {
            let key = format!("{}:{}:{}:{}", context, module, r#type, name);
            info!(%key, "Đang xử lý lệnh xóa bản ghi kiến trúc");
            match architecture::remove(&store, key.clone()).await {
                Ok(entry) => println!(
                    "Đã xóa bản ghi: [{}:{}:{}] {}",
                    entry.context, entry.module, entry.r#type, entry.name
                ),
                Err(Error::Missing) => println!("Không tìm thấy bản ghi để xóa: {}", key),
                Err(e) => return Err(e),
            }
        }
        Some(Commands::List { prefix, limit }) => {
            info!(%prefix, %limit, "Đang xử lý lệnh liệt kê bản ghi kiến trúc");
            let query = shared::query(prefix.into_bytes(), None::<Vec<u8>>, limit);
            let result = architecture::query(&store, query).await?;
            print(result)?;
        }
        None => {
            info!("Không có lệnh được chỉ định, hiển thị tin nhắn chào mừng");
            println!("Chào mừng đến với ứng dụng architecture. Sử dụng `add`, `get`, `list`, `remove`.");
        }
    }

    info!("Ứng dụng architecture hoàn thành thành công");
    Ok(())
} 