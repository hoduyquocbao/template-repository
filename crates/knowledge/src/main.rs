// main.rs
// Binary crate là điểm vào trung tâm cho hệ thống tri thức.

use clap::{Parser, Subcommand};
use repository::{self, Sled, Id, Error};
use tracing::info;

// Import các submodule mới với tên đơn từ
use knowledge::{architecture, memories, task};
use knowledge::task::Status;
use knowledge::display;
use shared::Showable;

/// Hệ thống quản lý tri thức kiến trúc và phát triển.
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Đường dẫn đến thư mục cơ sở dữ liệu Sled cho tất cả các bản ghi.
    #[arg(short, long, default_value = "db")]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Quản lý các bản ghi kiến trúc
    Architecture {
        #[command(subcommand)]
        command: Architecture, // ĐÃ ĐỔI TÊN TỪ ArchCmd THÀNH Architecture
    },
    /// Quản lý các bản ghi bộ nhớ (quyết định, phân tích)
    Memories {
        #[command(subcommand)]
        command: Memories, // ĐÃ ĐỔI TÊN TỪ MemCmd THÀNH Memories
    },
    /// Quản lý các bản ghi công việc (task list)
    Task {
        #[command(subcommand)]
        command: Task, // ĐÃ ĐỔI TÊN TỪ TaskCmd THÀNH Task
    },
    // Lệnh cho Director để khởi tạo các luồng nghiệp vụ (sẽ được implement sau)
    // Direct {
    //     #[command(subcommand)]
    //     command: Direct, // ĐÃ ĐỔI TÊN TỪ DirectCmd THÀNH Direct
    // }
}

// --- Lệnh con cho Architecture (Architecture) ---
#[derive(Subcommand)]
enum Architecture { // ĐÃ ĐỔI TÊN
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
    Del {
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
        /// Lọc theo loại (ví dụ: 'Agent', 'Module')
        #[arg(long)]
        r#type: Option<String>,
        /// Lọc thêm theo ngữ cảnh (chỉ hoạt động nếu có --type)
        #[arg(long)]
        context: Option<String>,
        /// Lọc thêm theo module (chỉ hoạt động nếu có --type và --context)
        #[arg(long)]
        module: Option<String>,
        /// Số lượng tối đa hiển thị
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },
}

// --- Lệnh con cho Memories (Memories) ---
#[derive(Subcommand)]
enum Memories { // ĐÃ ĐỔI TÊN
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

// --- Lệnh con cho Task (Todo) ---
#[derive(Subcommand)]
enum Task { // ĐÃ ĐỔI TÊN
    /// Thêm một công việc mới
    Add {
        task: String,
        #[arg(long, default_value = "")]
        context: String,
        #[arg(long, default_value = "")]
        module: String,
        #[arg(long, default_value = "Medium")]
        priority: String,
        #[arg(long, default_value = "Open")]
        status: String,
        #[arg(long, default_value = "")]
        assignee: String,
        #[arg(long, default_value = "")]
        due: String,
        #[arg(long, default_value = "")]
        notes: String,
    },
    /// Lấy một công việc bằng ID
    Get { id: Id },
    /// Đánh dấu một công việc là đã hoàn thành
    Done { id: Id },
    /// Xóa một công việc
    Del {
        #[arg(long)]
        id: Id,
    },
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
    },
    /// Thay đổi một công việc hiện có
    Change {
        id: Id,
        #[arg(long)]
        text: Option<String>,
        #[arg(long)]
        done: Option<bool>,
    },
}

#[tokio::main]
async fn main() -> Result<(), repository::Error> {
    tracing_subscriber::fmt::init();

    info!("Đang khởi động ứng dụng knowledge");

    let cli = Cli::parse();
    let store = Sled::new(&cli.path)?;

    match cli.command {
        Commands::Architecture { command } => match command {
            Architecture::Add {
                context,
                module,
                r#type,
                name,
                responsibility,
                dependency,
                performance,
                naming,
                prompt,
            } => {
                let args = architecture::Add {
                    context,
                    module,
                    r#type,
                    name,
                    responsibility,
                    dependency,
                    performance,
                    naming,
                    prompt,
                    created: repository::now(),
                };

                args.validate()?;

                let entry = architecture::add(&store, args).await?;
                println!("Đã thêm kiến trúc: {}", entry.name);
            }
            Architecture::Get {
                context,
                module,
                r#type,
                name,
            } => {
                let key = format!("{}:{}:{}:{}", context, module, r#type, name);
                match architecture::get(&store, context, module, r#type, name).await? {
                    Some(entry) => {
                        println!("Context: {}", entry.context);
                        println!("Module: {}", entry.module);
                        println!("Type: {:?}", entry.r#type);
                        println!("Name: {}", entry.name);
                        println!("Responsibility: {}", entry.responsibility);
                        println!("Dependency: {}", entry.dependency);
                        println!("Performance: {}", entry.performance);
                        println!("Naming: {}", entry.naming);
                        println!("Prompt: {}", entry.prompt);
                        println!("Created: {}", entry.created);
                    }
                    None => {
                        println!("Không tìm thấy kiến trúc với key: {}", key);
                    }
                }
            }
            Architecture::Del {
                context,
                module,
                r#type,
                name,
            } => {
                let key = format!("{}:{}:{}:{}", context, module, r#type, name);
                match architecture::del(&store, context, module, r#type, name).await {
                    Ok(entry) => println!(
                        "Đã xóa kiến trúc: [{}:{}:{}] {}",
                        entry.context, entry.module, entry.r#type, entry.name
                    ),
                    Err(Error::Missing) => println!("Không tìm thấy kiến trúc để xóa: {}", key),
                    Err(e) => return Err(e),
                }
            }
            Architecture::List { r#type, context, module, limit } => {
                let result = architecture::list(&store, r#type, context, module, limit).await?;
                display::show(result)?;
            }
        },
        Commands::Memories { command } => match command {
            Memories::Add {
                r#type,
                context,
                module,
                subject,
                description,
                decision,
                rationale,
            } => {
                let args = memories::Add {
                    r#type,
                    context,
                    module,
                    subject,
                    description,
                    decision,
                    rationale,
                    created: repository::now(),
                };

                args.validate()?;

                let entry = memories::add(&store, args).await?;
                println!(
                    "Đã thêm bộ nhớ: [{}] [{:?}]: {}",
                    entry.id, entry.r#type, entry.subject
                );
            }
            Memories::Get { id } => { // Cập nhật tên enum
                match memories::get(&store, id).await? {
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
                        println!("Không tìm thấy bộ nhớ với ID: {}", id);
                    }
                }
            }
            Memories::List { r#type, limit } => { // Cập nhật tên enum
                let result = memories::list(&store, r#type, limit).await?;
                display::show(result)?;
            }
        },
        Commands::Task { command } => match command {
            Task::Add {
                context,
                module,
                task: task_desc,
                priority,
                status,
                assignee,
                due,
                notes,
            } => {
                let priority_enum = task::Priority::try_from(priority)?;
                let status_enum = task::Status::try_from(status)?;

                let args = task::Add {
                    context,
                    module,
                    task: task_desc,
                    priority: priority_enum,
                    status: status_enum,
                    assignee,
                    due,
                    notes,
                };

                args.validate()?;

                let entry = task::add(&store, args).await?;
                println!("Đã thêm công việc: [{}], {}", entry.id, entry.task);
            }
            Task::Get { id } => {
                // let task_id = Id::try_from(id)?;
                let task = task::get(&store, id).await?;
                let status = match task.status {
                    Status::Done => "hoàn thành",
                    Status::Pending => "đang chờ",
                    Status::Open => "mở",
                };
                println!("[{}] {} ({})", task.id, task.task, status);
            }
            Task::Done { id } => { // Cập nhật tên enum
                let task = task::done(&store, id).await?;
                println!("Đã hoàn thành công việc: [{}], {}", task.id, task.task);
            }
            Task::Del { id } => { // Cập nhật tên enum
                let task = task::del(&store, id).await?;
                println!("Đã xóa công việc: [{}], {}", task.id, task.task);
            }
            Task::List { done, pending: _, limit } => {
                // Sử dụng hàm `filter` từ shared để tạo query
                let query = shared::filter(done, None, limit);

                let results = task::list(&store, query).await?;
                if results.is_empty() {
                    println!("Không tìm thấy công việc nào.");
                } else {
                    for summary in results {
                        summary.show();
                    }
                }
            }
            Task::Change { id, text, done } => {
                let task = task::get(&store, id).await?;
                let status = done.map(|d| if d { task::Status::Done } else { task::Status::Open });

                let patch = task::Patch {
                    task: text,
                    status,
                    ..Default::default()
                };
                let task = task::change(&store, task.id, patch).await?;
                println!("Đã thay đổi công việc: [{}], {}", task.id, task.task);
            }
        },
        // Commands::Direct { command } => {
        //     // Logic cho Director sẽ được thêm vào đây
        // }
    }

    info!("Ứng dụng knowledge hoàn thành thành công");
    Ok(())
}