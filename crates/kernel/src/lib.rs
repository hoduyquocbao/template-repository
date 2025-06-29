#![doc = "kernel - Framework lưu trữ, actor, metric, plugin, logging, validation, ... với triết lý tảng băng chìm: API một từ, ẩn toàn bộ phức tạp backend."]

//! # kernel - Framework lưu trữ & actor pattern
//!
//! ## Triết lý tảng băng chìm
//! - API public chỉ cần một từ, dễ dùng, dễ mở rộng.
//! - Toàn bộ concurrency, metric, error, backend, actor... được ẩn hoàn toàn.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Storage, Sled, Config, Plugin, Registry};
//!
//! let config = Config::new();
//! let storage = Sled::new("./db").unwrap();
//! // Insert, fetch, update, delete, query, mass...
//! ```

// lib.rs
// Crate thư viện chứa tất cả logic cốt lõi của framework.
// Mục tiêu: Định nghĩa điểm vào trung tâm, gom nhóm và tái xuất các module quan trọng.
// Thành tựu: Đảm bảo mọi định danh public đều là một từ tiếng Anh, API rõ ràng, dễ dùng.

// --- Định nghĩa các module con ---
// Mỗi module đại diện cho một khía cạnh cốt lõi của hệ thống, được đặt tên một từ duy nhất.
pub mod error;      // Module quản lý lỗi, chuẩn hóa toàn bộ hệ thống lỗi
pub mod extension;  // Module mở rộng, chuyển đổi lỗi từ bên ngoài về hệ thống
pub mod storage;    // Module trait Storage, trừu tượng hóa backend lưu trữ (bao gồm entity, pool, cache, time)
pub mod metric;     // Module thu thập metric, quan sát hiệu năng
pub mod engine;     // Module engine nền tảng
pub mod config;     // Module cấu hình
pub mod plugin;     // Module plugin system
pub mod logger;     // Module logging
pub mod builder;    // Module builder pattern
pub mod serializer; // Module serialization
pub mod router;     // Module router
pub mod validator;  // Module validator

// --- API framework: tái xuất abstraction một từ ---
pub use storage::Storage;
pub use storage::sled::Sled;
pub use storage::actor::Actor;
pub use metric::Registry;
pub use plugin::Plugin;
pub use config::Config;

// --- Tái xuất các thành phần cốt lõi ---
// Mục đích: Tạo API gọn gàng, giúp người dùng chỉ cần import từ crate gốc
// Thành tựu: Đảm bảo mọi định danh public đều là một từ tiếng Anh, không lộ chi tiết nội bộ
pub use error::Error; // Enum lỗi chuẩn hóa, một từ duy nhất
pub use extension::Extension; // Trait mở rộng lỗi, một từ duy nhất
pub use storage::entity::{Entity, Query, Key}; // Trait thực thể, struct truy vấn, builder khóa
pub use uuid::Uuid as Id; // Định danh duy nhất, tái xuất với tên Id (một từ)
pub use storage::pool::Pool; // Struct pool kết nối, một từ duy nhất
pub use storage::cache::Cache; // Struct cache, một từ duy nhất
pub use storage::time::now; // Tái xuất hàm now()