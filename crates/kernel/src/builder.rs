//! # Module Builder
//!
//! Cung cấp pattern builder để khởi tạo Engine với cấu hình và plugin linh hoạt.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Builder, Plugin, Config};
//! use std::sync::Arc;
//!
//! struct MyPlugin;
//! #[async_trait::async_trait]
//! impl Plugin for MyPlugin { /* ... */ }
//!
//! #[tokio::main]
//! async fn main() {
//!     let config = Config::new();
//!     let engine = Builder::new()
//!         .config(config)
//!         .plugin("my".to_string(), Arc::new(MyPlugin))
//!         .build()
//!         .await
//!         .unwrap();
//!     // ...
//! }
//! ```

use crate::engine::Engine;
use crate::config::Config;
use crate::plugin::Plugin;
use std::sync::Arc;

/// Builder cho Engine
///
/// Cho phép cấu hình Engine theo từng bước (fluent API): config, plugin, build.
pub struct Builder {
    config: Option<Config>,
    plugins: Vec<(String, Arc<dyn Plugin>)>,
}

impl Builder {
    /// Tạo builder mới
    pub fn new() -> Self {
        Self {
            config: None,
            plugins: Vec::new(),
        }
    }
    /// Set configuration cho Engine
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }
    /// Thêm plugin vào Engine
    pub fn plugin(mut self, name: String, plugin: Arc<dyn Plugin>) -> Self {
        self.plugins.push((name, plugin));
        self
    }
    /// Build Engine (async)
    ///
    /// # Returns
    /// - `Ok(Engine)` nếu thành công
    /// - `Err` nếu có lỗi
    pub async fn build(self) -> Result<Engine, Box<dyn std::error::Error>> {
        let _ = self.config.unwrap_or_default();
        let engine = Engine::new()?;
        for (name, plugin) in self.plugins {
            engine.add(name, plugin).await?;
        }
        Ok(engine)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugin::Plugin;

    /// Mock Plugin cho testing
    struct Mock;

    #[async_trait::async_trait]
    impl Plugin for Mock {
        async fn init(&self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
            Ok(())
        }

        fn name(&self) -> &str {
            "mock"
        }

        fn version(&self) -> &str {
            "1.0.0"
        }

        fn description(&self) -> &str {
            "Mock plugin for testing"
        }
    }

    #[tokio::test]
    async fn basic() {
        let engine = Builder::new()
            .build()
            .await
            .unwrap();
        
        assert_eq!(engine.state().await, crate::engine::State::Init);
    }

    #[tokio::test]
    async fn config() {
        let config = Config::new();
        let engine = Builder::new()
            .config(config)
            .build()
            .await
            .unwrap();
        
        assert_eq!(engine.state().await, crate::engine::State::Init);
    }

    #[tokio::test]
    async fn plugin() {
        let plugin = Mock;
        let engine = Builder::new()
            .plugin("test".to_string(), Arc::new(plugin))
            .build()
            .await
            .unwrap();
        
        let plugins = engine.list().await;
        assert!(plugins.contains(&"test".to_string()));
    }
} 