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
use shared::interaction::Interaction;

// Thêm ở đầu file:
// use naming::process;
// use naming::rules::report;

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
    /// Hiển thị số liệu thống kê hiệu suất của kho lưu trữ
    Stats,
    /// Phân tích mã nguồn để kiểm tra vi phạm quy tắc đặt tên
    Check {
        /// Đường dẫn đến file hoặc thư mục cần kiểm tra
        path: String,
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

// Thêm function helper để kiểm tra file Rust
fn check(content: &str, _file_path: &std::path::Path) -> Vec<String> {
    let mut fail = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    
    for (idx, line) in lines.iter().enumerate() {
        let idx = idx + 1;
        
        // Kiểm tra function definitions
        if line.contains("fn ") && !line.contains("//") {
            if let Some(func) = func(line) {
                if !word(func) {
                    fail.push(format!("Line {}: Function '{}' không phải single word", idx, func));
                }
            }
        }
        
        // Kiểm tra struct definitions
        if line.contains("struct ") && !line.contains("//") {
            if let Some(stru) = stru(line) {
                if !word(stru) {
                    fail.push(format!("Line {}: Struct '{}' không phải single word", idx, stru));
                }
            }
        }
        
        // Kiểm tra enum definitions
        if line.contains("enum ") && !line.contains("//") {
            if let Some(enu) = enu(line) {
                if !word(enu) {
                    fail.push(format!("Line {}: Enum '{}' không phải single word", idx, enu));
                }
            }
        }
        
        // Kiểm tra variable declarations
        if line.contains("let ") && !line.contains("//") {
            if let Some(var) = var(line) {
                if !word(var) {
                    fail.push(format!("Line {}: Variable '{}' không phải single word", idx, var));
                }
            }
        }
    }
    
    fail
}

fn func(line: &str) -> Option<&str> {
    if let Some(pos) = line.find("fn ") {
        let after = &line[pos + 3..];
        if let Some(space) = after.find(' ') {
            let func = &after[..space];
            if !func.is_empty() {
                return Some(func);
            }
        }
    }
    None
}

fn stru(line: &str) -> Option<&str> {
    if let Some(pos) = line.find("struct ") {
        let stru = &line[pos + 7..];
        if let Some(space) = stru.find(' ') {
            let stru = &stru[..space];
            if !stru.is_empty() {
                return Some(stru);
            }
        }
    }
    None
}

fn enu(line: &str) -> Option<&str> {
    if let Some(pos) = line.find("enum ") {
        let enu = &line[pos + 5..];
        if let Some(space) = enu.find(' ') {
            let enu = &enu[..space];
            if !enu.is_empty() {
                return Some(enu);
            }
        }
    }
    None
}

fn var(line: &str) -> Option<&str> {
    if let Some(pos) = line.find("let ") {
        let letv = &line[pos + 4..];
        if let Some(space) = letv.find(' ') {
            let var = &letv[..space];
            if !var.is_empty() && !var.contains('_') {
                return Some(var);
            }
        }
    }
    None
}

fn word(name: &str) -> bool {
    // Danh sách các từ được phép có underscore
    let allow = [
        "new_v4", "try_from", "as_str", "to_string", "clone", "build", "reserve",
        "read_file", "write_file", "file_path", "temp_dir", "test_db", "custom_path"
    ];
    
    if allow.contains(&name) {
        return true;
    }
    
    // Kiểm tra xem có underscore không
    !name.contains('_')
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
                let command = architecture::Add {
                    r#type,
                    context,
                    module,
                    name,
                    responsibility,
                    dependency,
                    performance,
                    naming,
                    prompt,
                    created: repository::now(),
                };
                let interaction = Interaction::new(command);
                let entry = architecture::add(&store, interaction).await?;
                println!("Đã thêm kiến trúc: [{}] {}", entry.id, entry.name);
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
                let command = memories::Add {
                    r#type,
                    context,
                    module,
                    subject,
                    description,
                    decision,
                    rationale,
                    created: repository::now(),
                };
                let interaction = Interaction::new(command);
                let entry = memories::add(&store, interaction).await?;
                println!("Đã thêm ký ức: [{}] {}", entry.id, entry.subject);
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
                let priority = task::Priority::try_from(priority)?;
                let status = task::Status::try_from(status)?;
                
                // Tạo command
                let command = task::Add {
                    context, module, task: task_desc,
                    priority, status,
                    assignee, due, notes,
                };
                
                // Đóng gói thành Interaction
                let interaction = Interaction::new(command);
                
                // Gọi handler mới
                let entry = task::add(&store, interaction).await?;
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
        Commands::Stats => {
            #[cfg(feature = "metrics")]
            {
                use repository::metric::Registry;
                
                println!("=== THỐNG KÊ HIỆU SUẤT KHO LƯU TRỮ ===");
                println!();
                
                // Tạo registry mới và hiển thị thông báo
                let _registry = Registry::new();
                println!("📊 TỔNG QUAN:");
                println!("  • Metrics registry đã được khởi tạo");
                println!("  • Chạy các thao tác để xem metrics thực tế");
                println!();
                
                println!("🔧 HƯỚNG DẪN:");
                println!("  • Thêm task: knowledge task add 'task name'");
                println!("  • Liệt kê task: knowledge task list");
                println!("  • Hoàn thành task: knowledge task done <id>");
                println!("  • Xóa task: knowledge task del --id <id>");
                println!();
                
                println!("📈 METRICS SẼ HIỂN THỊ:");
                println!("  • Insert: Số lần thêm dữ liệu");
                println!("  • Fetch: Số lần lấy dữ liệu");
                println!("  • Update: Số lần cập nhật dữ liệu");
                println!("  • Delete: Số lần xóa dữ liệu");
                println!("  • Query: Số lần truy vấn dữ liệu");
                println!("  • Mass: Số lần thao tác hàng loạt");
                println!("  • Keys: Số lần lấy danh sách keys");
            }
            
            #[cfg(not(feature = "metrics"))]
            {
                println!("Tính năng thống kê hiệu suất sẽ được cập nhật sau khi refactor actor pattern hoàn chỉnh.");
                println!("Để bật metrics, hãy chạy với flag: --features metrics");
            }
        }
        Commands::Check { path } => {
            println!("Bắt đầu kiểm tra quy tắc đặt tên cho: {}", path);
            
            // Kiểm tra xem có file naming.toml không
            let config = std::path::Path::new("naming.toml");
            if !config.exists() {
                println!("⚠️  Không tìm thấy file cấu hình naming.toml");
                println!("   Tạo file naming.toml với cấu hình mặc định...");
                
                // Tạo file naming.toml mặc định
                let default = r#"# Cấu hình quy tắc đặt tên
[general]
enforce_single_word = true
max_length = 50
allow_underscores = false

[patterns]
function_pattern = "^[a-z][a-z0-9]*$"
variable_pattern = "^[a-z][a-z0-9]*$"
struct_pattern = "^[A-Z][a-zA-Z0-9]*$"
enum_pattern = "^[A-Z][a-zA-Z0-9]*$"
module_pattern = "^[a-z][a-z0-9]*$"

[exceptions]
allowed_multi_word = [
    "new_v4",
    "try_from",
    "as_str",
    "to_string",
    "clone",
    "build",
    "reserve"
]
"#;
                
                if let Err(e) = std::fs::write("naming.toml", default) {
                    eprintln!("❌ Lỗi khi tạo file naming.toml: {}", e);
                } else {
                    println!("✅ Đã tạo file naming.toml với cấu hình mặc định");
                }
            }
            
            // Thực hiện kiểm tra đơn giản
            println!("🔍 Đang quét thư mục...");
            
            let mut fail = Vec::new();
            let mut files = 0;
            let mut violations = 0;
            
            if let Ok(entries) = std::fs::read_dir(&path) {
                for entry in entries.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                        files += 1;
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let err = check(&content, &path);
                            if !err.is_empty() {
                                fail.push((path, err.clone()));
                                violations += err.len();
                            }
                        }
                    }
                }
            }
            
            println!();
            println!("📊 KẾT QUẢ KIỂM TRA:");
            println!("  • Tổng số file Rust: {}", files);
            println!("  • Tổng số vi phạm: {}", violations);
            println!("  • File có vi phạm: {}", fail.len());
            
            if !fail.is_empty() {
                println!();
                println!("❌ CHI TIẾT VI PHẠM:");
                for (file_path, err) in fail {
                    println!("  📁 {}", file_path.display());
                    for violation in err {
                        println!("    • {}", violation);
                    }
                }
            } else {
                println!();
                println!("✅ Không tìm thấy vi phạm quy tắc đặt tên!");
            }
        }
        // Commands::Direct { command } => {
        //     // Logic cho Director sẽ được thêm vào đây
        // }
    }

    info!("Ứng dụng knowledge hoàn thành thành công");
    Ok(())
}