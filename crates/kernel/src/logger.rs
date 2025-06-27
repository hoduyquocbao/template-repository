#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Logger
//!
//! Cung cấp hệ thống logging cho framework, hỗ trợ nhiều level (info, warn, error, debug, trace).
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Logger, Config};
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::new();
//!     let logger = Logger::new(&config).unwrap();
//!     logger.info("Hello world");
//!     logger.warn("Warning");
//!     logger.error("Error");
//!     logger.debug("Debug");
//!     logger.trace("Trace");
//! }
//! ```

use crate::Config;

/// Logger cho Framework
///
/// Cung cấp các phương thức logging với các level khác nhau.
/// Hỗ trợ cả console và file logging.
pub struct Logger {
    // config: Config, // TODO: Nếu cần mở rộng logging theo config, giữ lại. Nếu không, loại bỏ.
}

impl Logger {
    /// Tạo logger mới
    pub fn new(_config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
        // Initialize tracing
        let _ = tracing_subscriber::fmt()
            .with_level(true)
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .try_init();
        Ok(Self {
            // config: config.clone(),
        })
    }
    
    /// Log info message
    pub fn info(&self, message: &str) {
        tracing::info!(target: "kernel::logger", "{}", message);
    }
    
    /// Log warning message
    pub fn warn(&self, message: &str) {
        tracing::warn!(target: "kernel::logger", "{}", message);
    }
    
    /// Log error message
    pub fn error(&self, message: &str) {
        tracing::error!(target: "kernel::logger", "{}", message);
    }
    
    /// Log debug message
    pub fn debug(&self, message: &str) {
        tracing::debug!(target: "kernel::logger", "{}", message);
    }
    
    /// Log trace message
    pub fn trace(&self, message: &str) {
        tracing::trace!(target: "kernel::logger", "{}", message);
    }
    
    /// Log với context
    pub fn context(&self, context: &str, message: &str) {
        tracing::info!(target: "kernel::logger", "[{}] {}", context, message);
    }
    
    /// Log performance metric
    pub fn performance(&self, operation: &str, duration: std::time::Duration) {
        tracing::info!(target: "kernel::logger", "PERFORMANCE: {} took {:?}", operation, duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let config = Config::new();
        let logger = Logger::new(&config);
        assert!(logger.is_ok());
    }

    #[test]
    fn method() {
        let config = Config::new();
        let logger = Logger::new(&config).unwrap();
        
        // Test các method logging
        logger.info("Test info message");
        logger.warn("Test warning message");
        logger.error("Test error message");
        logger.debug("Test debug message");
        logger.trace("Test trace message");
        logger.context("TEST", "Test context message");
        logger.performance("test_operation", std::time::Duration::from_millis(100));
    }
} 