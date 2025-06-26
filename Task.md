Chào bạn, tôi là Guardian, Kiến trúc sư Hệ thống.

Tôi đã tiến hành một đợt phân tích sâu rộng trên toàn bộ kho lưu trữ. Tôi không thể thực thi trực tiếp các lệnh `git diff` hay `git log`, nhưng dựa trên trạng thái các file bạn cung cấp và đối chiếu với các chỉ đạo trước, tôi đã xác định được các bước tiếp theo để nâng cấp hệ thống của chúng ta lên một tầm cao mới về **an toàn, khả năng quan sát và tính nhất quán kiến trúc**.

### 1\. Đánh giá Trạng thái Hiện tại và Chỉ đạo Tiếp theo

Tôi xác nhận các chỉ đạo gần đây nhất (`task-015`, `task-016`) về việc chuẩn hóa API và triển khai lớp xác thực đầu vào đã được Coder thực hiện. Các `struct Add` và phương thức `validate()` đã được đưa vào, tạo ra một vành đai phòng thủ đầu tiên.

Tuy nhiên, kiến trúc hiện tại vẫn còn một điểm yếu cốt lõi: các tương tác với hệ thống (như thêm một task, một memory) vẫn là các lời gọi hàm trực tiếp. Cách tiếp cận này thiếu một lớp trừu tượng trung gian, gây khó khăn cho việc:

1.  **Ghi nhật ký có cấu trúc (Structured Auditing):** Chúng ta không có một cách nhất quán để ghi lại *mọi* hành động được yêu cầu, các tham số của nó, và kết quả thực thi. Điều này làm cho việc truy vết các vấn đề tiềm ẩn hoặc các "mặt tối" trong luồng logic trở nên khó khăn.
2.  **Kiểm soát tập trung (Centralized Control):** Các logic như xác thực, cấp quyền (trong tương lai), và ghi nhật ký bị phân tán.
3.  **Tái sử dụng Framework:** Chúng ta chưa có một "framework" thực sự cho việc xử lý các yêu cầu nghiệp vụ.

**Quyết định Kiến trúc:** Chúng ta sẽ giới thiệu một **Lớp Tương tác (Interaction Layer)** dựa trên **Mẫu thiết kế Command**. Thay vì gọi trực tiếp các hàm nghiệp vụ, lớp giao diện (CLI) sẽ tạo ra các "lệnh" (Commands) đại diện cho một ý định. Các lệnh này sẽ được gói trong một ngữ cảnh "tương tác" (Interaction) có thể được ghi lại, xác thực, và xử lý một cách nhất quán.

### 2\. Yêu cầu Cải tiến Chi tiết cho Coder

**Gửi Coder:**

Hãy triển khai một Lớp Tương tác có cấu trúc để thay thế cho các lời gọi hàm trực tiếp, nhằm tăng cường khả năng quan sát, an toàn và tính nhất quán cho toàn bộ hệ thống.

**Mục tiêu:** Tái cấu trúc hệ thống để mọi yêu cầu nghiệp vụ được đóng gói thành một đối tượng `Interaction` chứa một `Command` cụ thể. Điều này sẽ tạo ra một framework xử lý yêu cầu có thể tái sử dụng và kiểm soát chặt chẽ.

**Nhiệm vụ 1: Nâng cấp Hệ thống Lỗi để Báo cáo Validation Chi tiết hơn**

Để hỗ trợ lớp tương tác mới, chúng ta cần một cơ chế báo lỗi xác thực mạnh mẽ hơn.

1.  **Tạo `ValidationError` Struct**: Trong file `crates/repository/src/error.rs`, định nghĩa một struct mới để mô tả một lỗi xác thực cụ thể.

    ```rust
    // THÊM VÀO crates/repository/src/error.rs
    use thiserror::Error; // Đảm bảo thiserror đã được import

    /// Đại diện cho một lỗi xác thực cụ thể cho một trường.
    #[derive(Error, Debug, Clone)]
    #[error("lỗi trường '{field}': {message}")]
    pub struct ValidationError {
        pub field: String,
        pub message: String,
    }
    ```

2.  **Cập nhật `Error` Enum**: Thay đổi biến thể `Error::Validation` để nó có thể chứa một danh sách các `ValidationError`.

    ```rust
    // THAY THẾ biến thể Validation trong crates/repository/src/error.rs
    #[derive(Error, Debug)]
    pub enum Error {
        // ... các biến thể khác ...

        /// Được trả về khi đầu vào không hợp lệ, chứa một danh sách các lỗi cụ thể.
        #[error("dữ liệu không hợp lệ")]
        Validation(Vec<ValidationError>), // THAY ĐỔI: Chứa một Vec các lỗi

        // ... các biến thể khác ...
    }
    ```

**Nhiệm vụ 2: Xây dựng Nền tảng "Interaction" trong `shared`**

Đây là trái tim của framework mới.

1.  **Tạo file `crates/shared/src/interaction.rs`**:

    ```rust
    // NỘI DUNG CỦA FILE MỚI: crates/shared/src/interaction.rs
    use repository::{self, Id};
    use std::fmt::Debug;
    use std::time::SystemTime;

    /// Một trait đánh dấu một struct là một lệnh có thể thực thi.
    /// Mọi lệnh phải định nghĩa kiểu dữ liệu Output của nó.
    pub trait Command: Debug {
        type Output;
    }

    /// Đóng gói một Command với các metadata cho việc ghi nhật ký và truy vết.
    #[derive(Debug)]
    pub struct Interaction<C: Command> {
        /// ID duy nhất cho mỗi lần tương tác.
        pub id: Id,
        /// Thời điểm tương tác được tạo ra.
        pub timestamp: SystemTime,
        /// Lệnh cụ thể được yêu cầu.
        pub command: C,
    }

    impl<C: Command> Interaction<C> {
        pub fn new(command: C) -> Self {
            Self {
                id: Id::new_v4(),
                timestamp: SystemTime::now(),
                command,
            }
        }
    }
    ```

2.  **Tái xuất module**: Trong file `crates/shared/src/lib.rs`, tái xuất module mới này.

    ```rust
    // THÊM VÀO crates/shared/src/lib.rs
    pub mod interaction;
    ```

**Nhiệm vụ 3: Tái cấu trúc `knowledge::task` thành một "Module Tương tác"**

Chúng ta sẽ áp dụng framework mới này cho `task` đầu tiên.

1.  **Biến `Add` Struct thành một `Command`**: Mở `crates/knowledge/src/task.rs`.

    ```rust
    // TRONG crates/knowledge/src/task.rs
    use shared::interaction::Command; // Import Command

    // ... định nghĩa struct Add ...

    // Triển khai Command cho Add
    impl Command for Add {
        type Output = Entry; // Kết quả trả về sau khi thêm thành công là một Entry
    }

    // Cập nhật phương thức validate để trả về Vec<ValidationError>
    impl Add {
        pub fn validate(&self) -> Result<(), Vec<ValidationError>> {
            let mut errors = Vec::new();
            if self.task.trim().is_empty() {
                errors.push(ValidationError {
                    field: "task".to_string(),
                    message: "Mô tả công việc không được để trống.".to_string(),
                });
            }
            if self.task.len() > 256 {
                errors.push(ValidationError {
                    field: "task".to_string(),
                    message: "Mô tả công việc không được vượt quá 256 ký tự.".to_string(),
                });
            }
            // ... Thêm các kiểm tra khác cho các trường còn lại ...

            if errors.is_empty() {
                Ok(())
            } else {
                Err(errors)
            }
        }
    }
    ```

2.  **Tái cấu trúc Hàm `add` thành `handle_add`**: Vẫn trong `crates/knowledge/src/task.rs`, đổi tên và thay đổi signature của hàm `add` để nó nhận vào một `Interaction`.

    ```rust
    // THAY THẾ hàm add cũ bằng hàm handle_add mới TRONG crates/knowledge/src/task.rs
    use shared::interaction::Interaction;
    use tracing::info;

    pub async fn handle_add<S: Storage>(store: &S, interaction: Interaction<Add>) -> Result<Entry, Error> {
        info!(interaction_id = %interaction.id, command = ?interaction.command, "Đang xử lý lệnh AddTask");
        
        // 1. Xác thực
        interaction.command.validate().map_err(Error::Validation)?;
        
        // 2. Gọi logic nghiệp vụ cốt lõi
        let result = task::add(
            store,
            interaction.command.context,
            interaction.command.module,
            interaction.command.task,
            interaction.command.priority,
            interaction.command.status,
            interaction.command.assignee,
            interaction.command.due,
            interaction.command.notes,
        ).await;

        // 3. Ghi nhật ký kết quả
        match &result {
            Ok(entry) => info!(interaction_id = %interaction.id, task_id = %entry.id, "Hoàn thành xử lý AddTask"),
            Err(e) => tracing::error!(interaction_id = %interaction.id, error = ?e, "Xử lý AddTask thất bại"),
        }
        
        result
    }

    // Xóa hoặc comment lại hàm `add` cũ để tránh nhầm lẫn.
    ```

3.  **Cập nhật `knowledge/src/main.rs`**: Sửa đổi CLI để tạo và gửi `Interaction`.

    ```rust
    // THAY ĐỔI trong crates/knowledge/src/main.rs, bên trong match Task::Add
    use shared::interaction::Interaction; // Import ở đầu file

    // ...
    Task::Add { ... } => {
        let priority_enum = task::Priority::try_from(priority)?;
        let status_enum = task::Status::try_from(status)?;
        
        // Tạo command
        let command = task::Add {
            context, module, task: task_desc,
            priority: priority_enum, status: status_enum,
            assignee, due, notes,
        };
        
        // Đóng gói thành Interaction
        let interaction = Interaction::new(command);
        
        // Gọi handler mới
        let entry = task::handle_add(&store, interaction).await?;
        println!("Đã thêm công việc: [{}], {}", entry.id, entry.task);
    }
    // ...
    ```

**Nhiệm vụ 4: Lặp lại Pattern cho `memories` và `architecture`**

Áp dụng chính xác quy trình 3 bước tương tự cho `knowledge::memories` và `knowledge::architecture`:

1.  **Implement `Command`** cho các struct `memories::Add` và `architecture::Add`.
2.  **Cập nhật `validate()`** của chúng để trả về `Result<(), Vec<ValidationError>>`.
3.  **Đổi tên hàm** `add` thành `handle_add` và cập nhật signature để nhận `Interaction<...>` tương ứng.
4.  **Thêm logic ghi nhật ký** vào đầu và cuối các handler mới.
5.  **Cập nhật `main.rs`** để tạo `Interaction` và gọi các handler mới cho `Memories::Add` và `Architecture::Add`.

### 3\. Cập nhật PKB

Tôi sẽ tạo các mục mới trong PKB để ghi lại quyết định kiến trúc này và giao nhiệm vụ cho bạn.

**`memories.csv` (Mục mới được đề xuất)**

```csv
"mem-011","Decision","System","All","Implement a structured Interaction Layer using the Command pattern","Direct function calls from the UI to business logic lack a centralized point for control, auditing, and structured logging. This poses a risk to observability and makes implementing cross-cutting concerns difficult.","Replaced direct function calls with a Command pattern. A shared 'Interaction' struct now wraps all business logic 'Commands' (e.g., AddTask, AddMemory). This wrapper contains metadata like a unique ID and timestamp. Facade functions were refactored into 'Handlers' that process these Interactions, enabling structured logging and validation at a single entry point.","This architectural shift massively improves system observability and auditability by creating a structured, loggable record for every attempted action. It decouples the UI from the business logic and provides a consistent, reusable framework for handling all future commands, enhancing security and maintainability."
```

**`todo.csv` (Nhiệm vụ mới)**

```csv
"task-017","Refactor","System","Implement a structured Interaction Layer","High","Open","Coder","","1. Enhance `repository::Error` to support `Vec<ValidationError>`. 2. Create the `Interaction`/`Command` framework in `crates/shared`. 3. Refactor `knowledge::task` to use the new framework (impl Command for Add, rename add->handle_add, update main.rs). 4. Apply the same pattern to `knowledge::memories`. 5. Apply the same pattern to `knowledge::architecture`. 6. Ensure all validation methods are updated and all tests pass."
```

Đây là một bước nhảy vọt về kiến trúc, không chỉ là một thay đổi nhỏ. Nó sẽ đặt nền móng cho một hệ thống vững chắc, an toàn và dễ kiểm soát hơn rất nhiều. Hãy tiến hành một cách cẩn trọng.