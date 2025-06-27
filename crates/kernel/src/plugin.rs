#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Plugin
//!
//! Định nghĩa interface plugin cho framework, cho phép mở rộng chức năng động.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Plugin, Config};
//!
//! struct MyPlugin;
//! #[async_trait::async_trait]
//! impl Plugin for MyPlugin {
//!     async fn init(&self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//!     async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
//!     fn name(&self) -> &str { "my" }
//!     fn version(&self) -> &str { "1.0.0" }
//!     fn description(&self) -> &str { "My plugin" }
//! }
//! ```
//! 

use crate::Config;

/// Trait cho Plugin system
///
/// Định nghĩa interface cho plugin động, hỗ trợ lifecycle (init, shutdown), metadata (name, version, description).
#[async_trait::async_trait]
pub trait Plugin: Send + Sync {
    /// Khởi tạo plugin
    async fn init(&self, config: &Config) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Dừng plugin
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>>;
    
    /// Lấy tên plugin
    fn name(&self) -> &str;
    
    /// Lấy version plugin
    fn version(&self) -> &str;
    
    /// Lấy description plugin
    fn description(&self) -> &str;
}

/// Plugin Registry quản lý tất cả plugins
pub struct Registry {
    plugins: std::collections::HashMap<String, Box<dyn Plugin>>,
}

impl Registry {
    /// Tạo registry mới
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
        }
    }
    
    /// Đăng ký plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        let name = plugin.name().to_string();
        self.plugins.insert(name, plugin);
        Ok(())
    }
    
    /// Hủy đăng ký plugin
    pub fn unregister(&mut self, name: &str) -> Option<Box<dyn Plugin>> {
        self.plugins.remove(name)
    }
    
    /// Lấy plugin theo tên
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|b| b.as_ref())
    }
    
    /// Lấy danh sách tất cả plugins
    pub fn list(&self) -> Vec<&dyn Plugin> {
        self.plugins.values().map(|b| b.as_ref()).collect()
    }
    
    /// Lấy số lượng plugins
    pub fn count(&self) -> usize {
        self.plugins.len()
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Mock Plugin cho testing
    struct Test;

    impl Test {
        fn new() -> Self {
            Self
        }
    }

    #[async_trait::async_trait]
    impl Plugin for Test {
        async fn init(&self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn name(&self) -> &str {
            "test"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn description(&self) -> &str {
            "Test plugin"
        }
    }

    #[tokio::test]
    async fn registry() {
        let mut registry = Registry::new();
        
        // Test add
        let plugin = Test::new();
        registry.register(Box::new(plugin)).unwrap();
        
        // Test count
        assert_eq!(registry.count(), 1);
        
        // Test get
        let plugin = registry.get("test");
        assert!(plugin.is_some());
        
        // Test list
        let plugins = registry.list();
        assert_eq!(plugins.len(), 1);
        
        // Test remove
        let plugin = registry.unregister("test");
        assert!(plugin.is_some());
        assert_eq!(registry.count(), 0);
    }
} 