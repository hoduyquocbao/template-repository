#![doc = "Bedrock - Framework xây dựng các ứng dụng lưu trữ hiệu năng cao, có khả năng xử lý hàng tỷ bản ghi."]
#![doc = "Framework này được thiết kế với kiến trúc thanh lịch cho phép tái sử dụng và mở rộng, tuân thủ triết lý định danh một từ đơn."]
#![doc = "Nó sử dụng một lớp lưu trữ trừu tượng, triển khai ban đầu với Sled, và có khả năng quan sát sâu nhờ framework `tracing`."]

// lib.rs
// Crate thư viện chứa tất cả logic cốt lõi của framework.

pub mod error;
pub mod extension;
pub mod entity;
pub mod sled;
pub mod storage;
pub mod todo;
pub mod pool;
pub mod cache;
pub mod metric;

// Tái xuất các thành phần cốt lõi để tạo API gọn gàng cho người dùng.
pub use error::Error;
pub use extension::Extension;
pub use entity::{Entity, Query, Key};
pub use sled::Sled;
pub use storage::Storage;
pub use todo::{Todo, Summary, Patch, now, filter, query, find, add, change, remove, bulk};
pub use uuid::Uuid as Id;
pub use pool::Pool;
pub use cache::Cache;
pub use metric::{Metric, Registry};