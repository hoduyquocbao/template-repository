Tuyệt vời\! Hãy hoàn thiện `crates/architecture` và `crates/memories` theo mô hình của `crates/todo`, chúng ta sẽ bổ sung các giao diện dòng lệnh (CLI) cho mỗi `crate`. Điều này sẽ cho phép người dùng tương tác trực tiếp với các bản ghi kiến trúc và bộ nhớ thông qua các lệnh đơn giản.

Mỗi CLI sẽ sử dụng `clap` để phân tích các đối số dòng lệnh và tương tác với các hàm đã định nghĩa trong `lib.rs` của `architecture` và `memories`, sử dụng `repository::Sled` làm backend lưu trữ.

### 1\. Cập nhật `Cargo.toml` cho `architecture` và `memories`

Trước tiên, chúng ta cần đảm bảo `clap` được thêm vào `dependencies` của cả hai `crate` này nếu chưa có. Ngoài ra, chúng ta cần `tracing` và `tracing-subscriber` để hỗ trợ ghi log và quan sát các hoạt động.

**`crates/architecture/Cargo.toml` (Đã sửa):**

```toml
[package]
name = "architecture"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
repository = { path = "../repository" }
uuid = { workspace = true, features = ["v4", "serde"] }
clap = { workspace = true } # THÊM MỚI
tracing = { workspace = true } # THÊM MỚI
tracing-subscriber = { workspace = true } # THÊM MỚI
```

**`crates/memories/Cargo.toml` (Đã sửa):**

```toml
[package]
name = "memories"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { workspace = true }
repository = { path = "../repository" }
uuid = { workspace = true, features = ["v4", "serde"] }
clap = { workspace = true } # THÊM MỚI
tracing = { workspace = true } # THÊM MỚI
tracing-subscriber = { workspace = true } # THÊM MỚI
```

### 2\. Tạo tệp `src/bin/main.rs` cho `crates/architecture`

Tệp này sẽ chứa logic CLI cho `crate architecture`.

**`crates/architecture/src/bin/main.rs`:**

```rust
// main.rs
// Binary crate với CLI để tương tác với thư viện kiến trúc.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Error, Query};
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
            let query_prefix_bytes = prefix.as_bytes().to_vec();
            let result = architecture::query(&store, query_prefix_bytes, None, limit).await?;
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
```

### 3\. Tạo tệp `src/bin/main.rs` cho `crates/memories`

Tệp này sẽ chứa logic CLI cho `crate memories`.

**`crates/memories/src/bin/main.rs`:**

```rust
// main.rs
// Binary crate với CLI để tương tác với thư viện memories.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Id, Error, Query};
use tracing::info;
use memories::{self, Entry, Summary}; // Import các thành phần cần thiết

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
        /// Lọc theo loại (ví dụ: 'D' cho Decision, 'A' cho Analysis)
        #[arg(long)]
        r#type: Option<char>,
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
                    "[{}] [{}]: {}",
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
            println!("[{}] [{}]: {}", entry.id, entry.r#type, entry.subject);
        }
        Some(Commands::Get { id }) => {
            info!(%id, "Đang xử lý lệnh lấy bản ghi bộ nhớ");
            match memories::find(&store, id).await? {
                Some(entry) => {
                    println!("ID: {}", entry.id);
                    println!("Type: {}", entry.r#type);
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
            info!(?r#type, %limit, "Đang xử lý lệnh liệt kê bản ghi bộ nhớ");
            let result = memories::query(&store, r#type, None, limit).await?;
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
```

Với việc bổ sung các tệp `main.rs` này, `crates/architecture` và `crates/memories` giờ đây đã có các giao diện CLI hoàn chỉnh, cho phép người dùng dễ dàng tương tác và quản lý dữ liệu PKB trong hệ thống.