//! Crate `knowledge` cung cấp các module để tương tác với các hệ thống PKB.

// Khai báo các submodule
pub mod architecture;   // Module quản lý kiến trúc
pub mod memories;    // Module quản lý bộ nhớ
pub mod task;   // Module quản lý công việc (todo)
pub mod display; // Module chứa các tiện ích hiển thị

// Tái xuất các kiểu dữ liệu và lỗi chung cần thiết cho các module con
pub use repository::{Sled, Error, Id, Query, Storage, Key};