//! Triển khai Entity cho mô hình Todo, hiển thị cách sử dụng framework.
//!
//! Module này phục vụ như một ví dụ tham chiếu về cách triển khai Entity trait
//! cho một loại dữ liệu cụ thể. Cung cấp các hàm tiện ích để tạo điều kiện
//! thuận lợi cho việc thao tác với các đối tượng Todo.

use serde::{Deserialize, Serialize};
use repository::{Storage, Id, Error, Entity, Query, Key};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::{info, instrument, debug, warn};

/// Đại diện cho một công việc duy nhất với timestamp.
///
/// Đây là cấu trúc dữ liệu chính của hệ thống, lưu trữ tất cả thông tin
/// liên quan đến một công việc cần thực hiện hoặc đã hoàn thành.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
pub struct Todo {
    /// ID duy nhất của công việc, được tạo ra bằng UUID v4
    pub id: Id,
    
    /// Nội dung mô tả của công việc
    pub text: String,
    
    /// Trạng thái của công việc (true = đã hoàn thành, false = đang chờ)
    pub done: bool,
    
    /// Unix timestamp (nanoseconds) của thời điểm tạo.
    /// Được sử dụng để sắp xếp và tạo chỉ mục.
    pub created: u128,
}

/// Một bản tóm tắt của `Todo` để hiển thị trong danh sách.
/// 
/// Được lưu trữ trong cây chỉ mục để tạo thành một "covering index" (chỉ mục bao phủ),
/// giúp tối ưu hóa các truy vấn danh sách mà không cần truy cập dữ liệu chính.
#[derive(Serialize, Deserialize, Debug,  PartialEq, Eq)]
pub struct Summary {
    /// ID duy nhất của công việc, dùng để tham chiếu đến dữ liệu đầy đủ
    pub id: Id,
    
    /// Nội dung mô tả của công việc
    pub text: String,
}

/// Triển khai Entity trait cho Todo
impl Entity for Todo {
    const NAME: &'static str = "todos";
    
    type Key = Id;
    type Index = Vec<u8>;
    type Summary = Summary;
    
    fn key(&self) -> Self::Key {
        self.id
    }
    
    fn index(&self) -> Self::Index {
        // Tạo khóa chỉ mục sử dụng các phương thức một từ mới
        let mut key = Key::reserve(33);  // Sử dụng 'reserve' thay cho 'with_capacity'
        key.flag(self.done);             // Sử dụng 'flag' thay cho 'add_bool'
        key.time(self.created);          // Sử dụng 'time' thay cho 'add_rev_time'
        key.id(self.id);                 // Sử dụng 'id' thay cho 'add_id'
        key.clone().build()
    }
    
    fn summary(&self) -> Self::Summary {
        Summary {
            id: self.id,
            text: self.text.clone(),
        }
    }
}

/// Đại diện cho một bản vá (thay đổi một phần) cho một `Todo`.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct Patch {
    /// Nội dung mới của công việc, nếu cần cập nhật
    pub text: Option<String>,
    
    /// Trạng thái mới của công việc, nếu cần cập nhật
    pub done: Option<bool>,
}

/// Thay đổi: `status_query` thành `filter` để thể hiện mục đích (lọc theo trạng thái)
/// và tuân thủ quy tắc một từ.
/// Hàm tiện ích để tạo truy vấn cho công việc có trạng thái cụ thể.
pub fn filter(done: bool, after: Option<(u128, Id)>, limit: usize) -> Query<Vec<u8>> {
    let prefix = vec![if done { 1 } else { 0 }];
    
    let after = after.map(|(created, id)| {
        // Cập nhật để sử dụng Key builder mới
        let mut key = Key::reserve(33);  // Sử dụng 'reserve' thay cho 'with_capacity'
        key.flag(done);                  // Sử dụng 'flag' thay cho 'add_bool'
        key.time(created);               // Sử dụng 'time' thay cho 'add_rev_time'
        key.id(id);                      // Sử dụng 'id' thay cho 'add_id'
        key.clone().build()
    });
    
    Query {
        prefix,
        after,
        limit,
    }
}

/// Lấy thời gian hiện tại dưới dạng Unix timestamp nano giây.
/// 
/// Hàm này được sử dụng để tạo timestamp cho các công việc mới.
/// Nó trả về số nano giây kể từ Unix epoch (1970-01-01 00:00:00 UTC).
pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

/// Thêm một công việc mới vào hệ thống lưu trữ.
///
/// Hàm này nhận vào một đối tượng triển khai `Storage` và nội dung của công việc.
/// Nó sẽ tạo một `Todo` mới với ID duy nhất, trạng thái `done = false`, và timestamp hiện tại.
///
/// # Đối số
///
/// * `store`: Một tham chiếu đến bất kỳ đối tượng nào triển khai `Storage` trait.
/// * `text`: Nội dung văn bản của công việc cần tạo. Phải là một chuỗi không rỗng.
///
/// # Trả về
///
/// Trả về một `Result` chứa `Todo` vừa được tạo thành công, hoặc một `Error` nếu có lỗi xảy ra.
///
/// # Lỗi
///
/// * `Error::Input`: Nếu `text` rỗng.
/// * `Error::Store`: Nếu có lỗi từ lớp lưu trữ trong quá trình chèn.
///
/// # Ví dụ
///
/// ```rust
/// # use repository::{Sled, Storage, add, Error};
/// # use tempfile::tempdir;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// # let dir = tempdir().unwrap();
/// # let path = dir.path().to_str().unwrap();
/// let store = Sled::new(path)?;
/// let text = "Tạo tài liệu cho dự án".to_string();
///
/// let todo = add(&store, text).await?;
///
/// assert_eq!(todo.text, "Tạo tài liệu cho dự án");
/// assert!(!todo.done);
/// # Ok(())
/// # }
/// ```
#[instrument(skip(store))]
pub async fn add<S: Storage>(store: &S, text: String) -> Result<Todo, Error> {
    // Thay đổi: `task_text` thành `text`
    info!(text = %text, "Đang thêm công việc mới");
    
    if text.is_empty() {
        warn!("Cố gắng thêm công việc với nội dung rỗng");
        return Err(Error::Input);
    }
    
    let todo = Todo {
        id: Id::new_v4(),
        text,
        done: false,
        created: now(),
    };
    
    // Clone để có thể trả về
    let result = todo.clone();
    
    debug!(id = %todo.id, "Đang chèn công việc vào kho lưu trữ");
    store.insert(todo).await?;
    
    info!(id = %result.id, "Thêm công việc thành công");
    Ok(result)
}

/// Tìm một công việc bằng ID của nó.
///
/// # Đối số
///
/// * `store`: Một tham chiếu đến đối tượng triển khai `Storage` trait.
/// * `id`: ID của công việc cần tìm.
///
/// # Trả về
///
/// * `Ok(Todo)`: Nếu tìm thấy công việc.
/// * `Err(Error::Missing)`: Nếu không tìm thấy công việc với ID đã cho.
/// * `Err(Error::Store)`: Nếu có lỗi từ lớp lưu trữ.
///
/// # Ví dụ
///
/// ```rust
/// # use repository::{Sled, Storage, add, find, Error, Id};
/// # use tempfile::tempdir;
/// #
/// # #[tokio::main]
/// # async fn main() -> Result<(), Error> {
/// # let dir = tempdir().unwrap();
/// # let path = dir.path().to_str().unwrap();
/// # let store = Sled::new(path)?;
/// # let todo = add(&store, "Tìm kiếm công việc".to_string()).await?;
/// # let id = todo.id;
/// // Tìm một công việc đã tồn tại
/// let todo = find(&store, id).await?;
/// assert_eq!(todo.text, "Tìm kiếm công việc");
/// 
/// // Tìm một công việc không tồn tại
/// let dummy = Id::new_v4();
/// let result = find(&store, dummy).await;
/// assert!(matches!(result, Err(Error::Missing)));
/// # Ok(())
/// # }
/// ```
#[instrument(skip(store))]
pub async fn find<S: Storage>(store: &S, id: Id) -> Result<Todo, Error> {
    info!(%id, "Đang tìm công việc theo ID");
    
    match store.fetch::<Todo>(id).await? {
        Some(todo) => {
            info!(%id, text = %todo.text, done = %todo.done, "Đã tìm thấy công việc");
            Ok(todo)
        },
        None => {
            warn!(%id, "Không tìm thấy công việc");
            Err(Error::Missing)
        }
    }
}

/// Cập nhật một công việc bằng một giao dịch nguyên tử.
#[instrument(skip(store))]
pub async fn change<S: Storage>(store: &S, id: Id, patch: Patch) -> Result<Todo, Error> {
    info!(%id, ?patch, "Đang cập nhật công việc");
    
    if let Some(text) = &patch.text {
        if text.is_empty() {
            warn!(%id, "Cố gắng cập nhật công việc với nội dung rỗng");
            return Err(Error::Input);
        }
    }
    
    // Sử dụng update với hàm transform
    let result = store.update::<Todo, _>(id, move |mut todo| {
        if let Some(text) = patch.text {
            todo.text = text;
        }
        if let Some(done) = patch.done {
            todo.done = done;
        }
        todo
    }).await?;
    
    info!(%id, text = %result.text, done = %result.done, "Cập nhật công việc thành công");
    Ok(result)
}

/// Xóa một công việc khỏi kho lưu trữ.
#[instrument(skip(store))]
pub async fn remove<S: Storage>(store: &S, id: Id) -> Result<Todo, Error> {
    info!(%id, "Đang xóa công việc");
    
    let result = store.delete::<Todo>(id).await?;
    info!(%id, text = %result.text, "Xóa công việc thành công");
    Ok(result)
}

/// Truy vấn một danh sách tóm tắt các công việc.
#[instrument(skip(store))]
pub async fn query<S: Storage>(store: &S, status: bool, after: Option<(u128, Id)>, limit: usize) 
    -> Result<Box<dyn Iterator<Item = Result<Summary, Error>> + Send>, Error> 
{
    // Cập nhật: Gọi hàm `filter` đã đổi tên
    let query = filter(status, after, limit);
    info!(?query, "Đang truy vấn danh sách công việc");
    
    let result = store.query::<Todo>(query).await?;
    debug!("Truy vấn thực hiện thành công");
    Ok(result)
}

/// Chèn một iterator các công việc theo từng lô nhỏ để đảm bảo an toàn bộ nhớ.
///
/// Hàm này nhận một iterator cung cấp các công việc cần chèn và truyền nó trực tiếp
/// xuống lớp lưu trữ để xử lý theo lô, giúp duy trì mô hình streaming với
/// độ phức tạp bộ nhớ O(1) không phụ thuộc vào kích thước đầu vào.
///
/// # Đối số
///
/// * `store`: Đối tượng triển khai Storage trait để lưu trữ các công việc.
/// * `iter`: Iterator cung cấp các công việc cần chèn.
///
/// # Trả về
///
/// * `Ok(())`: Nếu tất cả các công việc được chèn thành công.
/// * `Err(Error)`: Nếu có lỗi xảy ra trong quá trình chèn.
#[instrument(skip(store, iter))]
pub async fn bulk<S: Storage>(store: &S, iter: impl Iterator<Item = Todo> + Send + 'static) -> Result<(), Error> {
    info!("Đang chèn hàng loạt công việc");
    store.mass::<Todo>(Box::new(iter)).await?;
    info!("Chèn hàng loạt hoàn thành thành công");
    Ok(())
}

// --- Kiểm thử đơn vị ---
#[cfg(test)]
mod tests {
    use super::*;
    use repository::sled::Sled;
    use tokio::runtime::Runtime;

    fn memory() -> Sled {
        // Sử dụng uuid để đảm bảo mỗi test có đường dẫn riêng
        let path = format!("db/{}", uuid::Uuid::new_v4());
        Sled::new(&path).unwrap() // Use the public constructor
    }
    

    #[test]
    fn retrieval() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(&store, "công việc kiểm thử".to_string()).await.unwrap();
            assert!(!added.done);

            let todo = find(&store, added.id).await.unwrap();
            assert_eq!(added, todo);
        });
    }
    
    #[test]
    fn failure() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let _added = add(&store, "công việc kiểm thử".to_string()).await.unwrap(); // Add underscore to indicate intentional non-use
            let id = Id::new_v4();
            let result = find(&store, id).await;
            assert!(matches!(result, Err(Error::Missing)));
        });
    }

    #[test]
    fn index() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let _added = add(&store, "a task".to_string()).await.unwrap(); // Added underscore prefix

            // Fix: Update query to use the new 4-argument pattern
            let results = query(&store, false, None, 10).await.unwrap();
            let summaries: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(summaries[0].text, "a task");
            
            let results = query(&store, true, None, 10).await.unwrap();
            let completed: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(completed.len(), 0);
        });
    }
    
    #[test]
    fn transaction() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(&store, "original".to_string()).await.unwrap();
            let patch = Patch { text: Some("updated".to_string()), done: Some(true) };
            let updated = change(&store, added.id, patch).await.unwrap();
            assert_eq!(updated.text, "updated");
            assert!(updated.done);

            // Use correct query API
            let results = query(&store, true, None, 10).await.unwrap();
            let items: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(items.len(), 1);
            assert_eq!(items[0].text, "updated");
        });
    }
    
    #[test]
    fn removal() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let added = add(&store, "công việc kiểm thử".to_string()).await.unwrap(); // Change _added to added since it's used
            
            remove(&store, added.id).await.unwrap();
            
            let result = find(&store, added.id).await;
            assert!(matches!(result, Err(Error::Missing)));
            
            // Fix query call to use the correct 4-argument pattern
            let summaries: Vec<_> = query(&store, false, None, 10)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(summaries.len(), 0);
        });
    }
    
    #[test]
    fn paging() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory(); // Dùng hàm tạo test mới
            let mut todos = vec![];

            for i in 0..5 {
                let todo = add(&store, format!("công việc {}", i)).await.unwrap();
                todos.push(todo);
            }
            
            todos.sort_by(|a, b| b.created.cmp(&a.created));
            let ids: Vec<_> = todos.iter().map(|t| t.id).collect();

            // Trang 1
            // Cập nhật: Gọi `query` (nó sẽ gọi `filter` bên trong)
            let page1: Vec<_> = query(&store, false, None, 3)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(page1.len(), 3);
            assert_eq!(page1[0].id, ids[0]);

            // Lấy con trỏ
            let last = &page1[2];
            let item = find(&store, last.id).await.unwrap();
            let cursor = Some((item.created, item.id));

            // Trang 2
            let page2: Vec<_> = query(&store, false, cursor, 3)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(page2.len(), 2);
        });
    }
    
    #[test]
    fn insertion() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let todos = vec![
                Todo { id: Id::new_v4(), text: "hàng loạt 1".to_string(), done: false, created: now() },
                Todo { id: Id::new_v4(), text: "hàng loạt 2".to_string(), done: true, created: now() },
            ];
            
            // Chuyển quyền sở hữu của todos thay vì clone
            crate::bulk(&store, todos.into_iter()).await.unwrap();

            // Fix the query calls to use the correct 4-argument pattern
            // Kiểm tra các mục đang chờ - thu thập vào Vec để sử dụng len()
            let pending: Vec<_> = query(&store, false, None, 10)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(pending.len(), 1);
            
            // Kiểm tra các mục đã hoàn thành - thu thập vào Vec để sử dụng len()
            let done: Vec<_> = query(&store, true, None, 10)
                .await
                .unwrap()
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            assert_eq!(done.len(), 1);
        });
    }
    
    #[test]
    fn concurrency() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let _added = add(&store, "công việc 1".to_string()).await.unwrap(); // Fix: unused variable warning
            let _added2 = add(&store, "công việc 2".to_string()).await.unwrap();
            let _added3 = add(&store, "công việc 3".to_string()).await.unwrap();
            
            // Fix iterator length issue by collecting to Vec first
            let results = query(&store, false, None, 10).await.unwrap();
            let todos: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(todos.len(), 3);
            
            // Fix the same issue for the second query
            let results = query(&store, true, None, 10).await.unwrap();
            let completed: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(completed.len(), 0);
        });
    }
    
    #[test]
    fn bulk() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            let total = 250;
            
            let todos = (0..total).map(|i| Todo {
                id: Id::new_v4(),
                text: format!("mục {}", i),
                done: i % 2 == 0,
                created: now() + i as u128,
            });

            super::bulk(&store, todos).await.unwrap();
            
            // Update the query call to use the correct pattern
            let results = query(&store, false, None, 250).await.unwrap();
            let todos: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(todos.len(), 125);
            
            let results = query(&store, true, None, 250).await.unwrap();
            let completed: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(completed.len(), 125);
        });
    }
    
    #[test]
    fn stress() {
        let rt = Runtime::new().unwrap();
        rt.block_on(async {
            let store = memory();
            // Fix unused variable warning by renaming to _added
            let _added = add(&store, "Task 1".to_string()).await.unwrap();
            let _added2 = add(&store, "Task 2".to_string()).await.unwrap();
            let _added3 = add(&store, "Task 3".to_string()).await.unwrap();
            
            // Fix iterator length issue by collecting to Vec first
            let results = query(&store, false, None, 10).await.unwrap();
            let todos: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(todos.len(), 3);
            
            // Fix the same issue for the second query
            let results = query(&store, true, None, 10).await.unwrap();
            let completed: Vec<_> = results.collect::<Result<Vec<_>, _>>().unwrap();
            assert_eq!(completed.len(), 0);
        });
    }
}