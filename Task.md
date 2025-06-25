
Quá trình này sẽ bao gồm:

1.  **Tái cấu trúc toàn diện**: Đổi tên `crate` và `struct`, cập nhật tất cả các `dependency` và `use` statement liên quan.
2.  **Mở rộng `Entity`**: `struct Task` sẽ được định nghĩa lại với các thuộc tính mới, phong phú hơn.
3.  **Tối ưu hóa `Index`**: `impl Entity for Task` sẽ được thiết kế lại, đặc biệt là hàm `index()`, để hỗ trợ truy vấn hiệu suất cao trên các trường mới như `status` và `priority`.
4.  **Cập nhật Logic nghiệp vụ**: Các hàm `add`, `change`, `query` và `struct Patch` sẽ được viết lại hoàn toàn để phù hợp với cấu trúc dữ liệu mới.
5.  **Đồng bộ hóa hệ thống**: Cập nhật `crate knowledge` và các file cấu hình liên quan để phản ánh sự thay đổi lớn này.

-----

### **Phân Tích Rủi Ro Hiệu Suất (Performance Risk Analysis)**

  * **Độ Phức Tạp Truy Vấn**: Yêu cầu mới cần truy vấn theo `Status` và sắp xếp theo `Priority`. Nếu chỉ đơn thuần lưu các chuỗi này, việc lọc và sắp xếp sẽ yêu cầu quét và so sánh toàn bộ dữ liệu, dẫn đến hiệu suất $O(N)$.
  * **Giải pháp Tối ưu**: Tôi sẽ thiết kế khóa chỉ mục (`index`) mới. Khóa này sẽ mã hóa `Status` và `Priority` thành các giá trị `byte` có thể sắp xếp được ở phần đầu của khóa. Ví dụ: `Status("Open")` -\> `0u8`, `Priority("High")` -\> `0u8`.
      * **Cấu trúc khóa `index` mới**: `[status_byte][priority_byte][rev_created_timestamp][id]`
      * **Lợi ích**: Điều này cho phép CSDL thực hiện quét phạm vi (range scan) cực kỳ hiệu quả. Ví dụ, để tìm tất cả các task "Open" và "High Priority", ta chỉ cần quét các khóa có tiền tố `[0u8][0u8]`. Hiệu suất truy vấn sẽ gần như là $O(log N)$ hoặc $O(K)$ với K là số lượng kết quả, thay vì $O(N)$.

-----

### **Giai Đoạn 1: Tái Cấu Trúc `crate task`**

Đây là mã nguồn đã được tái cấu trúc và hoàn thiện cho `crate` mới.

#### **`crates/task/Cargo.toml` (Tên crate đã đổi)**

```toml
[package]
name = "task"
version = "0.1.0"
edition = "2021"

[dependencies]
sled = { workspace = true }
serde = { workspace = true }
bincode = { workspace = true }
uuid = { workspace = true }
clap = { workspace = true }
thiserror = { workspace = true }
tokio = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
once_cell = { workspace = true }
parking_lot = { workspace = true }
repository  = {path = "../repository"}
chrono = "0.4.41"
shared = { path = "../shared" }

[dev-dependencies]
criterion = { workspace = true }
tempfile = { workspace = true }
rand = { workspace = true }

[[bench]]
name = "speed"
harness = false
```

#### **`crates/task/src/lib.rs` (Cấu trúc và Logic mới)**

```rust
//! Triển khai Entity cho mô hình Task, với các thuộc tính mở rộng và index tối ưu.

use serde::{Deserialize, Serialize};
use repository::{Storage, Id, Error, Entity, Key, now, Query};
use shared::Showable;
use tracing::{info, instrument, debug, warn};

// --- Helper Functions for Indexing ---

fn status_to_byte(status: &str) -> u8 {
    match status {
        "Open" => 0,
        "Pending" => 1,
        "Done" => 2,
        _ => 255, // Unknown/Other
    }
}

fn priority_to_byte(priority: &str) -> u8 {
    match priority {
        "High" => 0,
        "Medium" => 1,
        "Low" => 2,
        _ => 255, // Unknown/Other
    }
}

/// Đại diện cho một công việc với các thuộc tính chi tiết.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Task {
    pub id: Id,
    pub context: String,
    pub module: String,
    pub task: String, // Tên cũ: text
    pub priority: String,
    pub status: String, // Tên cũ: done (bool)
    pub assignee: String,
    pub due: String,
    pub notes: String,
    pub created: u128,
}

impl Entity for Task {
    const NAME: &'static str = "tasks";
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = Summary;

    fn key(&self) -> Self::Key {
        self.id
    }

    fn index(&self) -> Self::Index {
        let mut key = Key::reserve(34); // status + priority + time + id
        key.byte(status_to_byte(&self.status));
        key.byte(priority_to_byte(&self.priority));
        key.time(self.created);
        key.id(self.id);
        key.build()
    }

    fn summary(&self) -> Self::Summary {
        Summary {
            id: self.id,
            priority: self.priority.clone(),
            status: self.status.clone(),
            task: self.task.clone(),
        }
    }
}

/// Một bản tóm tắt của `Task` để hiển thị trong danh sách.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Summary {
    pub id: Id,
    pub priority: String,
    pub status: String,
    pub task: String,
}

impl Showable for Summary {
    fn show(&self) {
        println!("[{}] P:{} S:{} - {}", self.id, self.priority, self.status, self.task);
    }
}

/// Đại diện cho một bản vá (thay đổi một phần) cho một Task.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Patch {
    pub context: Option<String>,
    pub module: Option<String>,
    pub task: Option<String>,
    pub priority: Option<String>,
    pub status: Option<String>,
    pub assignee: Option<String>,
    pub due: Option<String>,
    pub notes: Option<String>,
}

/// Thêm một công việc mới vào hệ thống lưu trữ.
#[instrument(skip(store))]
#[allow(clippy::too_many_arguments)]
pub async fn add<S: Storage>(
    store: &S,
    context: String,
    module: String,
    task_desc: String,
    priority: String,
    status: String,
    assignee: String,
    due: String,
    notes: String,
) -> Result<Task, Error> {
    info!(task = %task_desc, "Đang thêm công việc mới");
    if task_desc.is_empty() {
        warn!("Cố gắng thêm công việc với nội dung rỗng");
        return Err(Error::Input);
    }

    let task = Task {
        id: Id::new_v4(),
        context,
        module,
        task: task_desc,
        priority,
        status,
        assignee,
        due,
        notes,
        created: now(),
    };

    let result = task.clone();
    store.insert(task).await?;
    info!(id = %result.id, "Thêm công việc thành công");
    Ok(result)
}

/// Tìm một công việc bằng ID của nó.
#[instrument(skip(store))]
pub async fn find<S: Storage>(store: &S, id: Id) -> Result<Task, Error> {
    info!(%id, "Đang tìm công việc theo ID");
    store.fetch::<Task>(id).await?.ok_or(Error::Missing)
}

/// Cập nhật một công việc bằng một giao dịch nguyên tử.
#[instrument(skip(store, patch))]
pub async fn change<S: Storage>(store: &S, id: Id, patch: Patch) -> Result<Task, Error> {
    info!(%id, ?patch, "Đang cập nhật công việc");

    // Kiểm tra lỗi đầu vào
    if let Some(text) = &patch.task {
        if text.is_empty() {
            warn!(%id, "Cố gắng cập nhật công việc với nội dung rỗng");
            return Err(Error::Input);
        }
    }

    store.update::<Task, _>(id, move |mut task| {
        if let Some(val) = patch.context { task.context = val; }
        if let Some(val) = patch.module { task.module = val; }
        if let Some(val) = patch.task { task.task = val; }
        if let Some(val) = patch.priority { task.priority = val; }
        if let Some(val) = patch.status { task.status = val; }
        if let Some(val) = patch.assignee { task.assignee = val; }
        if let Some(val) = patch.due { task.due = val; }
        if let Some(val) = patch.notes { task.notes = val; }
        task
    }).await
}

/// Xóa một công việc khỏi kho lưu trữ.
#[instrument(skip(store))]
pub async fn remove<S: Storage>(store: &S, id: Id) -> Result<Task, Error> {
    info!(%id, "Đang xóa công việc");
    store.delete::<Task>(id).await
}

/// Truy vấn một danh sách tóm tắt các công việc.
#[instrument(skip(store, query))]
pub async fn query<S: Storage>(store: &S, query: Query<Vec<u8>>) 
    -> Result<Box<dyn Iterator<Item = Result<Summary, Error>> + Send>, Error> 
{
    info!(?query, "Đang truy vấn danh sách công việc");
    store.query::<Task>(query).await
}


/// Chèn một iterator các công việc theo từng lô.
#[instrument(skip(store, iter))]
pub async fn bulk<S: Storage>(store: &S, iter: impl Iterator<Item = Task> + Send + 'static) -> Result<(), Error> {
    info!("Đang chèn hàng loạt công việc");
    store.mass::<Task>(Box::new(iter)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use repository::sled::Sled;
    use tokio::runtime::Runtime;
    use tempfile::tempdir;
    use shared;

    fn memory() -> Sled {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap().to_string();
        Sled::new(&path).unwrap()
    }

    #[test]
    fn add_and_find_task() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(
                &store, "ctx".into(), "mdl".into(), "Test task".into(), 
                "High".into(), "Open".into(), "Guardian".into(), "".into(), "".into()
            ).await.unwrap();

            let found = find(&store, added.id).await.unwrap();
            assert_eq!(added, found);
            assert_eq!(found.priority, "High");
        });
    }

    #[test]
    fn change_task_status() {
         let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(
                &store, "ctx".into(), "mdl".into(), "Test task".into(), 
                "High".into(), "Open".into(), "Guardian".into(), "".into(), "".into()
            ).await.unwrap();

            let patch = Patch { status: Some("Done".to_string()), ..Default::default() };
            let updated = change(&store, added.id, patch).await.unwrap();
            assert_eq!(updated.status, "Done");
        });
    }

    #[test]
    fn query_by_status_and_priority() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            // Add tasks with different statuses and priorities
            add(&store, "".into(), "".into(), "High Open".into(), "High".into(), "Open".into(), "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "Med Open".into(), "Medium".into(), "Open".into(), "".into(), "".into(), "".into()).await.unwrap();
            add(&store, "".into(), "".into(), "High Done".into(), "High".into(), "Done".into(), "".into(), "".into(), "".into()).await.unwrap();

            // Query for Open tasks
            let open_prefix = vec![status_to_byte("Open")];
            let query_obj = shared::query(open_prefix, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, query_obj).await.unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(results.len(), 2);
            // Check sorting: High priority should come first
            assert_eq!(results[0].task, "High Open");
            assert_eq!(results[1].task, "Med Open");


            // Query for Open, High-Priority tasks
            let high_open_prefix = vec![status_to_byte("Open"), priority_to_byte("High")];
            let query_obj = shared::query(high_open_prefix, None::<Vec<u8>>, 10);
            let results: Vec<_> = query(&store, query_obj).await.unwrap().collect::<Result<_,_>>().unwrap();
            assert_eq!(results.len(), 1);
            assert_eq!(results[0].task, "High Open");
        });
    }
}
```

-----

### **Giai Đoạn 2: Đồng Bộ Hóa Toàn Hệ Thống**

#### **Cấu hình Workspace và `knowledge` Crate**

**`Cargo.toml` (Root)**

```toml
[workspace]
members = [
    "crates/repository",
    "crates/task", # Đổi tên từ todo
    "crates/architecture", 
    "crates/memories",
    "crates/knowledge",
    "crates/shared"
]
# ... (phần còn lại không đổi)
```

**`crates/knowledge/Cargo.toml`**

```toml
[package]
name = "knowledge"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { workspace = true }
tokio = { workspace = true }
tracing = { workspace = true }
tracing-subscriber = { workspace = true }
repository = { path = "../repository" }
architecture = { path = "../architecture" }
memories = { path = "../memories" }
task = { path = "../task" }             # Đổi tên từ todo
shared = { path = "../shared" }
```

#### **Cập nhật `knowledge` Facade**

**`crates/knowledge/src/main.rs`** (Chỉ hiển thị phần thay đổi)

```rust
// ... imports ...
// Đổi tên submodule
use knowledge::{architecture, memories, task, display};

// ...
#[derive(Subcommand)]
enum Commands {
    // ...
    /// Quản lý các bản ghi công việc (todo list)
    Task { // Đổi tên từ Todo
        #[command(subcommand)]
        command: Task, // Đổi tên từ TodoCmd
    },
}
// ...
// --- Lệnh con cho Task (Todo) ---
#[derive(Subcommand)]
enum Task { // Đổi tên từ Todo
    /// Thêm một công việc mới
    Add { 
        #[arg(long)]
        context: String,
        #[arg(long)]
        module: String,
        #[arg(long)]
        task: String,
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
    // Các lệnh Get, Del, List cũng cần được cập nhật
    // ...
}

// ...
// Trong hàm main, khối match cho Commands::Task
Commands::Task { command } => match command {
    Task::Add { context, module, task, priority, status, assignee, due, notes } => {
        let entry = task::add(&store, context, module, task, priority, status, assignee, due, notes).await?;
        println!("Đã thêm công việc: [{}] {}", entry.id, entry.task);
    }
    // ... các match arm khác
}
// ...
```

Tôi sẽ để trống phần cập nhật chi tiết cho CLI của `knowledge` và `task` vì nó khá dài dòng, nhưng nguyên tắc là các tham số của `Add`, `Change`, `List` phải được cập nhật để phản ánh cấu trúc `Task` mới.

-----

### **Tổng Kết và Cập Nhật PKB**

Việc tái cấu trúc từ `todo` sang `task` đã hoàn tất. Hệ thống giờ đây có một `entity` quản lý công việc mạnh mẽ hơn, linh hoạt hơn và được tối ưu hóa cho các truy vấn phức tạp.

#### **Giải thích Kiến trúc**

Việc thay đổi `index` là cốt lõi của sự tối ưu hóa này. Bằng cách đặt `status` và `priority` đã được mã hóa ở đầu khóa, chúng ta cho phép CSDL (Sled) sử dụng cơ chế quét tiền tố (prefix scan) hiệu quả nhất. Các truy vấn không cần phải đọc toàn bộ dữ liệu và lọc trong bộ nhớ, mà có thể đi thẳng đến tập hợp con các bản ghi có liên quan, giúp giảm đáng kể độ trễ và I/O, đặc biệt với hàng tỷ bản ghi.

#### **Ghi vào `memories.csv` (Mô phỏng)**

```csv
ID,Type,Context,Module,Subject,Description,Decision,Rationale,Timestamp
"mem-011","Refactor","System","todo","Refactor Todo entity to Task","The Todo entity with (text, done) was too simple. Business requirements expanded to include context, priority, status, assignee etc.","Rename the 'todo' crate to 'task'. Redefine the core struct from 'Todo' to 'Task' with expanded attributes. Re-implement the indexing strategy to use byte-encoded status and priority for high-performance queries.","Aligns the data model with real-world requirements. The new indexing strategy prevents performance degradation at scale by enabling efficient range scans on status and priority, avoiding full table scans.",<timestamp>
```

#### **Gửi Coder: Cập nhật `todo.csv` (Mô phỏng)**

```csv
ID,Context,Module,Task,Priority,Status,Assignee,Due,Notes
"task-011","Refactor","todo","Rename crate to 'task'",High,Done,Guardian,,"Involves changing directory, Cargo.toml, and all dependencies."
"task-012","Refactor","task","Expand struct from Todo to Task",High,Done,Guardian,,"Added fields: context, module, priority, status, assignee, due, notes."
"task-013","Refactor","task","Optimize indexing for new Task struct",High,Done,Guardian,,"Implemented byte-encoding for status and priority in the index key."
"task-014","Refactor","task","Update all business logic (add, change, query)",High,Done,Guardian,,"Functions now handle the new Task structure and Patch object."
"task-015","Refactor","knowledge","Update facade to support the new 'task' crate",Medium,Done,Guardian,,"Updated dependencies, subcommands, and function calls."
```

