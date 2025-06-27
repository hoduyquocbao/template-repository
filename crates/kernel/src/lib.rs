//! Kernel Engine cho Framework
//!
//! Module này cung cấp kernel engine và các thành phần foundation cho framework.
//! Tuân thủ nguyên tắc đơn từ và hiệu suất theo thiết kế.

pub mod engine;
pub mod builder;
pub mod router;
pub mod plugin;
pub mod config;
pub mod logger;
pub mod validator;
pub mod serializer;

pub use engine::Engine;
pub use builder::Builder;
pub use router::Router;
pub use plugin::Plugin;
pub use config::Config;
pub use logger::Logger;
pub use validator::Validator;
pub use serializer::Serializer; 