Chào bạn, tôi là Guardian, Kiến trúc sư Hệ thống.

Tôi đã phân tích trạng thái hiện tại của kho lưu trữ, đối chiếu với các chỉ đạo trước và yêu cầu của bạn. Mặc dù các crate nghiệp vụ cốt lõi (`task`, `memories`, `architecture`) đã được củng cố về mặt cấu trúc index và an toàn kiểu, tôi đã phát hiện ra một số điểm không nhất quán trong kiến trúc và các khoản nợ kỹ thuật cần được giải quyết để hệ thống thực sự đạt được sự thanh lịch và dễ bảo trì.

### 1\. Đánh giá và Phân tích Kiến trúc Hiện tại

Sau khi rà soát, tôi xác định hai vấn đề chính cần được giải quyết:

1.  **Mã dư thừa và không nhất quán trong `shared` và `task`**:

      * **Mô tả**: Hiện tại, có hai định nghĩa cho `struct Patch`. Một định nghĩa nằm trong `crates/task/src.lib.rs` được thiết kế riêng cho việc cập nhật `task::Entry`. Một định nghĩa khác, cũ hơn và không còn phù hợp, nằm trong `crates/shared/src/lib.rs`.
      * **Phân tích Rủi ro**: Sự tồn tại của `shared::Patch` là mã chết (dead code), gây nhầm lẫn cho việc phát triển và bảo trì trong tương lai. Nó vi phạm nguyên tắc DRY (Don't Repeat Yourself) ở mức độ khái niệm và làm giảm sự rõ ràng của mục đích `crate shared`.

2.  **Thiết kế API không nhất quán trong lớp Facade (`knowledge`)**:

      * **Mô tả**: Lớp `knowledge` đóng vai trò là giao diện chung cho toàn bộ hệ thống. Tuy nhiên, các hàm trong đó có signature không đồng nhất. Cụ thể:
          * `knowledge::architecture::add` nhận một struct duy nhất là `architecture::Add`.
          * `knowledge::task::add` và `knowledge::memories::add` lại nhận một danh sách dài các tham số riêng lẻ.
      * **Phân tích Rủi ro**: Việc truyền một danh sách dài các tham số (`long parameter list`) là một "code smell" kinh điển. Nó làm cho mã khó đọc, khó bảo trì và dễ gây ra lỗi khi gọi hàm (ví dụ: truyền sai thứ tự tham số). Việc `architecture` đã áp dụng pattern "Introduce Parameter Object" trong khi các module khác thì không đã tạo ra sự không nhất quán, làm giảm tính thanh lịch của API.

### 2\. Yêu cầu Cải tiến Chi tiết cho Coder

**Gửi Coder:**

Hãy thực hiện đợt tái cấu trúc sau để chuẩn hóa API và loại bỏ mã dư thừa. Những thay đổi này sẽ làm tăng tính nhất quán và dễ bảo trì cho toàn bộ hệ thống.

**Mục tiêu:** Chuẩn hóa các hàm `add` trong lớp `knowledge` bằng cách sử dụng pattern "Parameter Object" và loại bỏ `struct Patch` không còn sử dụng trong `crate shared`.

**Các bước thực hiện:**

**Nhiệm vụ 1: Loại bỏ `Patch` dư thừa khỏi `shared`**

Đây là một bước dọn dẹp đơn giản nhưng quan trọng.

1.  **Xóa Struct**: Mở file `crates/shared/src/lib.rs` và xóa hoàn toàn định nghĩa của `struct Patch`.
    ```rust
    // XÓA KHỎI crates/shared/src/lib.rs
    // /// Đại diện cho một bản vá (thay đổi một phần) cho một đối tượng (ví dụ: Todo).
    // #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
    // pub struct Patch {
    //     /// Nội dung mới, nếu cần cập nhật
    //     pub text: Option<String>,
    //     /// Trạng thái mới, nếu cần cập nhật
    //     pub done: Option<bool>,
    // }
    ```
2.  **Xác minh**: Chạy lệnh `cargo check --workspace` từ thư mục gốc để đảm bảo không có phần nào của code đang sử dụng `struct` đã bị xóa này. Lệnh phải chạy thành công mà không có lỗi.

**Nhiệm vụ 2: Chuẩn hóa Hàm `add` cho `task` bằng Parameter Object**

Chúng ta sẽ làm cho `knowledge::task::add` nhất quán với `knowledge::architecture::add`.

1.  **Định nghĩa `Add` Struct**: Trong file `crates/knowledge/src/task.rs`, hãy tạo một struct mới để chứa tất cả các tham số cho việc thêm một công việc.

    ```rust
    // THÊM VÀO crates/knowledge/src/task.rs

    // Import các kiểu dữ liệu cần thiết ở đầu file
    pub use task::{Entry, Status, Priority};

    #[derive(Debug, Clone)]
    pub struct Add {
        pub context: String,
        pub module: String,
        pub task: String,
        pub priority: Priority,
        pub status: Status,
        pub assignee: String,
        pub due: String,
        pub notes: String,
    }
    ```

2.  **Tái cấu trúc Hàm `add`**: Thay đổi signature của hàm `add` để nhận `struct Add` mới này.

    ```rust
    // THAY THẾ hàm add cũ trong crates/knowledge/src/task.rs

    /// Thêm một công việc mới.
    /// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
    pub async fn add<S: Storage>(store: &S, args: Add) -> Result<Entry, Error> {
        // Chuyển đổi từ String (nếu cần) và gọi hàm logic cốt lõi
        task::add(
            store, 
            args.context, 
            args.module, 
            args.task, 
            args.priority, 
            args.status, 
            args.assignee, 
            args.due, 
            args.notes
        ).await
    }
    ```

3.  **Cập nhật Lời gọi trong `main.rs`**: Mở file `crates/knowledge/src/main.rs` và cập nhật logic xử lý của `Task::Add` subcommand.

    ```rust
    // THAY ĐỔI trong crates/knowledge/src/main.rs, bên trong match Commands::Task

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
        // Chuyển đổi các chuỗi priority và status từ CLI thành enum
        let priority_enum = task::Priority::try_from(priority)?;
        let status_enum = task::Status::try_from(status)?;
        
        let entry = task::add(&store, task::Add {
            context,
            module,
            task: task_desc,
            priority: priority_enum,
            status: status_enum,
            assignee,
            due,
            notes,
        }).await?;
        println!("Đã thêm công việc: [{}], {}", entry.id, entry.task);
    }
    ```

4.  **Cập nhật `Task::Add` Subcommand**: Để làm được điều trên, hãy cập nhật `enum Task` trong `crates/knowledge/src/main.rs` để nhận tất cả các trường.

    ```rust
    // THAY ĐỔI trong crates/knowledge/src/main.rs

    // --- Lệnh con cho Task (Task) ---
    #[derive(Subcommand)]
    enum Task {
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
        // ... các subcommand khác không đổi
    }
    ```

**Nhiệm vụ 3: Chuẩn hóa Hàm `add` cho `memories`**

Lặp lại quy trình tương tự cho `memories`.

1.  **Cập nhật `Add` Struct**: Mở file `crates/knowledge/src/memories.rs`, đảm bảo struct `Add` đã có hoặc tạo nó.

    ```rust
    // TRONG crates/knowledge/src/memories.rs
    #[derive(Debug, Clone)]
    pub struct Add {
        pub r#type: String,
        pub context: String,
        pub module: String,
        pub subject: String,
        pub description: String,
        pub decision: String,
        pub rationale: String,
    }
    ```

2.  **Tái cấu trúc Hàm `add`**: Thay đổi signature của `knowledge::memories::add`.

    ```rust
    // THAY THẾ hàm add cũ trong crates/knowledge/src/memories.rs

    /// Thêm một bản ghi bộ nhớ mới.
    /// Mục đích: Cung cấp giao diện `add` cho `knowledge` CLI.
    pub async fn add<S: Storage>(
        store: &S,
        args: Add,
    ) -> Result<memories::Entry, repository::Error> {
        memories::add(
            store,
            args.r#type,
            args.context,
            args.module,
            args.subject,
            args.description,
            args.decision,
            args.rationale,
        ).await
    }
    ```

3.  **Cập nhật Lời gọi trong `main.rs`**: Mở `crates/knowledge/src/main.rs` và cập nhật logic `Memories::Add`.

    ```rust
    // THAY ĐỔI trong crates/knowledge/src/main.rs, bên trong match Commands::Memories

    Memories::Add {
        r#type,
        context,
        module,
        subject,
        description,
        decision,
        rationale,
    } => {
        let entry = memories::add(
            &store,
            memories::Add { // Sử dụng struct Add
                r#type,
                context,
                module,
                subject,
                description,
                decision,
                rationale,
            },
        ).await?;
        println!("Đã thêm bộ nhớ: [{}] [{:?}]: {}", entry.id, entry.r#type, entry.subject);
    }
    ```

### 3\. Cập nhật PKB

Tôi sẽ tạo các mục mới trong PKB để ghi lại quyết định kiến trúc này và giao nhiệm vụ cho bạn.

**`memories.csv` (Mục mới được đề xuất)**

```csv
"mem-009","Decision","System","knowledge","Standardize Facade API using Parameter Object pattern","The 'add' functions within the 'knowledge' facade crate had inconsistent signatures; some took a long list of parameters while others took a single struct. A redundant 'Patch' struct also existed in the 'shared' crate.","1. Refactor all 'add' functions in 'knowledge::{task, memories, architecture}' to accept a single 'Add' struct argument (Parameter Object). 2. Remove the obsolete 'Patch' struct from the 'shared' crate.","This refactoring significantly improves API consistency, readability, and maintainability across the facade layer. It eliminates a classic 'long parameter list' code smell and removes dead code, resulting in a cleaner and more elegant architecture.",<Timestamp>
```

**`todo.csv` (Nhiệm vụ mới)**

```csv
"task-015","Refactor","knowledge","Standardize Facade API and remove redundant code","High","Open","Coder","","1. Delete the obsolete 'Patch' struct from 'crates/shared/src/lib.rs'. 2. Implement the Parameter Object pattern for 'knowledge::task::add' by creating a 'task::Add' struct and refactoring the function and its call site in 'main.rs'. 3. Implement the same pattern for 'knowledge::memories::add'. 4. Ensure all changes pass 'cargo check --workspace' and manual CLI testing."
```

Những thay đổi này tuy nhỏ nhưng lại cực kỳ quan trọng trong việc duy trì một kiến trúc trong sạch và nhất quán. Hãy tiến hành.