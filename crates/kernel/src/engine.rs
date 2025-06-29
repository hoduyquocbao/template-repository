#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Engine
//!
//! Thành phần trung tâm của framework, quản lý lifecycle, plugin, cấu hình, logging, routing.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Engine, Plugin};
//! use std::sync::Arc;
//!
//! struct MyPlugin;
//! #[async_trait::async_trait]
//! impl Plugin for MyPlugin { /* ... */ }
//!
//! #[tokio::main]
//! async fn main() {
//!     let engine = Engine::new().unwrap();
//!     engine.add("my", Arc::new(MyPlugin)).await.unwrap();
//!     engine.start().await.unwrap();
//!     // ...
//!     engine.stop().await.unwrap();
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::config::Config;
use crate::logger::Logger;
use crate::router::Router;
use crate::plugin::Plugin;

/// Core Engine quản lý lifecycle và điều phối framework
/// 
/// - Quản lý lifecycle (start, stop)
/// - Quản lý plugin
/// - Quản lý cấu hình, logging, routing
pub struct Engine {
    /// Cấu hình hệ thống
    config: Arc<Config>,
    /// Logger system
    logger: Arc<Logger>,
    /// Router cho request/command
    router: Arc<Router>,
    /// Plugin registry
    plugins: Arc<RwLock<HashMap<String, Arc<dyn Plugin>>>>,
    /// Trạng thái engine
    state: Arc<RwLock<State>>,
}

/// Trạng thái của Engine
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    /// Engine đang khởi tạo
    Init,
    /// Engine đã sẵn sàng
    Ready,
    /// Engine đang chạy
    Running,
    /// Engine đang dừng
    Stopping,
    /// Engine đã dừng
    Stopped,
    /// Engine gặp lỗi
    Error,
}

impl Engine {
    /// Tạo Engine mới với cấu hình mặc định
    ///
    /// # Returns
    /// - `Ok(Engine)` nếu khởi tạo thành công
    /// - `Err` nếu có lỗi
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config = Arc::new(Config::default());
        let logger = Arc::new(Logger::new(&config)?);
        let router = Arc::new(Router::new());
        let plugins = Arc::new(RwLock::new(HashMap::new()));
        let state = Arc::new(RwLock::new(State::Init));

        Ok(Self {
            config,
            logger,
            router,
            plugins,
            state,
        })
    }

    /// Khởi động Engine (async)
    ///
    /// - Chuyển trạng thái sang Ready, Running
    /// - Khởi tạo plugin, router
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut state = self.state.write().await;
            *state = State::Ready;
        }

        self.logger.info("Engine đang khởi động...");

        // Khởi tạo plugins
        self.setup().await?;

        // Khởi tạo router
        self.router.init().await?;

        {
            let mut state = self.state.write().await;
            *state = State::Running;
        }

        self.logger.info("Engine đã khởi động thành công");
        Ok(())
    }

    /// Dừng Engine (async)
    ///
    /// - Chuyển trạng thái sang Stopping, Stopped
    /// - Dừng plugin, router
    pub async fn stop(&self) -> Result<(), Box<dyn std::error::Error>> {
        {
            let mut state = self.state.write().await;
            *state = State::Stopping;
        }

        self.logger.info("Engine đang dừng...");

        // Dừng plugins
        self.shutdown().await?;

        // Dừng router
        self.router.shutdown().await?;

        {
            let mut state = self.state.write().await;
            *state = State::Stopped;
        }

        self.logger.info("Engine đã dừng thành công");
        Ok(())
    }

    /// Lấy trạng thái hiện tại
    pub async fn state(&self) -> State {
        self.state.read().await.clone()
    }

    /// Thêm plugin vào engine
    ///
    /// # Arguments
    /// * `name` - Tên plugin
    /// * `plugin` - Đối tượng plugin
    pub async fn add(&self, name: String, plugin: Arc<dyn Plugin>) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugins = self.plugins.write().await;
        plugins.insert(name.clone(), plugin);
        self.logger.info(&format!("Đã thêm plugin: {}", name));
        Ok(())
    }

    /// Xóa plugin theo tên
    pub async fn remove(&self, name: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut plugins = self.plugins.write().await;
        if plugins.remove(name).is_some() {
            self.logger.info(&format!("Đã xóa plugin: {}", name));
        }
        Ok(())
    }

    /// Lấy plugin theo tên
    pub async fn get(&self, name: &str) -> Option<Arc<dyn Plugin>> {
        let plugins = self.plugins.read().await;
        plugins.get(name).cloned()
    }

    /// Lấy danh sách tên tất cả plugin
    pub async fn list(&self) -> Vec<String> {
        let plugins = self.plugins.read().await;
        plugins.keys().cloned().collect()
    }

    /// Khởi tạo tất cả plugin (nội bộ)
    async fn setup(&self) -> Result<(), Box<dyn std::error::Error>> {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            plugin.init(&self.config).await?;
            self.logger.info(&format!("Đã khởi tạo plugin: {}", name));
        }
        Ok(())
    }

    /// Dừng tất cả plugin (nội bộ)
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let plugins = self.plugins.read().await;
        for (name, plugin) in plugins.iter() {
            plugin.shutdown().await?;
            self.logger.info(&format!("Đã dừng plugin: {}", name));
        }
        Ok(())
    }

    /// Lấy reference đến config
    pub fn config(&self) -> &Arc<Config> {
        &self.config
    }

    /// Lấy reference đến logger
    pub fn logger(&self) -> &Arc<Logger> {
        &self.logger
    }

    /// Lấy reference đến router
    pub fn router(&self) -> &Arc<Router> {
        &self.router
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new().expect("Không thể tạo Engine mặc định")
    }
}

/// Mock Plugin cho testing
struct _Mock;

impl _Mock {
    fn _new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl Plugin for _Mock {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn life() {
        let engine = Engine::new().unwrap();
        
        // Test start
        assert_eq!(engine.state().await, State::Init);
        engine.start().await.unwrap();
        assert_eq!(engine.state().await, State::Running);
        
        // Test stop
        engine.stop().await.unwrap();
        assert_eq!(engine.state().await, State::Stopped);
    }

    #[tokio::test]
    async fn plugin() {
        let engine = Engine::new().unwrap();
        engine.start().await.unwrap();
        
        // Test add plugin
        let plugin = _Mock::_new();
        engine.add("test".to_string(), Arc::new(plugin)).await.unwrap();
        
        // Test list plugins
        let plugins = engine.list().await;
        assert!(plugins.contains(&"test".to_string()));
        
        // Test remove plugin
        engine.remove("test").await.unwrap();
        let plugins = engine.list().await;
        assert!(!plugins.contains(&"test".to_string()));
        
        engine.stop().await.unwrap();
    }
} 