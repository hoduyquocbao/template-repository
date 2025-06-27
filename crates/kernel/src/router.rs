#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Router
//!
//! Định tuyến request/command đến handler tương ứng. Hỗ trợ đăng ký, hủy, xử lý route động.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{Router, Handler, Request, Response};
//! use std::sync::Arc;
//!
//! struct Echo;
//! #[async_trait::async_trait]
//! impl Handler for Echo {
//!     async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error>> {
//!         Ok(Response { status: 200, headers: Default::default(), body: req.body })
//!     }
//! }
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new();
//!     router.register("/echo".to_string(), Arc::new(Echo)).await;
//!     let req = Request { path: "/echo".to_string(), method: "POST".to_string(), headers: Default::default(), body: b"hi".to_vec() };
//!     let res = router.route(req).await.unwrap();
//!     assert_eq!(res.body, b"hi");
//! }
//! ```

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

/// Định nghĩa trait Handler cho router
#[async_trait]
pub trait Handler: Send + Sync {
    /// Xử lý request, trả về response
    async fn handle(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>>;
}

/// Định nghĩa request cho router
#[derive(Debug, Clone)]
pub struct Request {
    /// Path của request
    pub path: String,
    /// Method của request
    pub method: String,
    /// Headers của request
    pub headers: HashMap<String, String>,
    /// Body của request
    pub body: Vec<u8>,
}

/// Định nghĩa response cho router
#[derive(Debug, Clone)]
pub struct Response {
    /// Status code
    pub status: u16,
    /// Headers của response
    pub headers: HashMap<String, String>,
    /// Body của response
    pub body: Vec<u8>,
}

/// Router cho Framework
///
/// Định tuyến request đến handler dựa trên path/method. Hỗ trợ đăng ký, hủy, đếm, xử lý route động.
/// 
/// Router định tuyến request đến handler tương ứng dựa trên path và method.
/// Hỗ trợ pattern matching và middleware.
pub struct Router {
    /// Route registry
    routes: Arc<RwLock<HashMap<String, Arc<dyn Handler>>>>,
}

impl Router {
    /// Tạo router mới
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Đăng ký route
    pub async fn register(&self, path: String, handler: Arc<dyn Handler>) {
        let mut routes = self.routes.write().await;
        routes.insert(path, handler);
    }
    
    /// Hủy đăng ký route
    pub async fn unregister(&self, path: &str) {
        let mut routes = self.routes.write().await;
        routes.remove(path);
    }
    
    /// Route request
    pub async fn route(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        let routes = self.routes.read().await;
        
        // Tìm handler cho path
        if let Some(handler) = routes.get(&request.path) {
            handler.handle(request).await
        } else {
            // Return 404 if no handler found
            Ok(Response {
                status: 404,
                headers: HashMap::new(),
                body: b"Not Found".to_vec(),
            })
        }
    }
    
    /// Khởi tạo router
    pub async fn init(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Initialize default routes
        self.setup().await;
        Ok(())
    }
    
    /// Shutdown router
    pub async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        let mut routes = self.routes.write().await;
        routes.clear();
        Ok(())
    }
    
    /// Lấy số lượng routes
    pub async fn count(&self) -> usize {
        let routes = self.routes.read().await;
        routes.len()
    }
    
    /// Khởi tạo default routes
    async fn setup(&self) {
        // Health check route
        let health = Arc::new(Health);
        self.register("/health".to_string(), health).await;
        
        // Metrics route
        let metrics = Arc::new(Metrics);
        self.register("/metrics".to_string(), metrics).await;
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

/// Health check handler
struct Health;

#[async_trait::async_trait]
impl Handler for Health {
    async fn handle(&self, _request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        Ok(Response {
            status: 200,
            headers: HashMap::new(),
            body: b"OK".to_vec(),
        })
    }
}

/// Metrics handler
struct Metrics;

#[async_trait::async_trait]
impl Handler for Metrics {
    async fn handle(&self, _request: Request) -> Result<Response, Box<dyn std::error::Error>> {
        Ok(Response {
            status: 200,
            headers: HashMap::new(),
            body: b"metrics".to_vec(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test handler
    struct Test;

    #[async_trait::async_trait]
    impl Handler for Test {
        async fn handle(&self, request: Request) -> Result<Response, Box<dyn std::error::Error>> {
            Ok(Response {
                status: 200,
                headers: HashMap::new(),
                body: request.body,
            })
        }
    }

    #[tokio::test]
    async fn route() {
        let router = Router::new();
        
        // Test add
        let handler = Arc::new(Test);
        router.register("/test".to_string(), handler).await;
        
        // Test count
        assert_eq!(router.count().await, 1); // Chỉ có 1 route vừa đăng ký
        
        // Test route
        let request = Request {
            path: "/test".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: b"test body".to_vec(),
        };
        
        let response = router.route(request).await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"test body");
        
        // Test 404
        let request = Request {
            path: "/none".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: vec![],
        };
        
        let response = router.route(request).await.unwrap();
        assert_eq!(response.status, 404);
        
        // Test unregister
        router.unregister("/test").await;
        assert_eq!(router.count().await, 0); // Không còn route nào
    }

    #[tokio::test]
    async fn init() {
        let router = Router::new();
        router.init().await.unwrap();
        
        // Test health route
        let request = Request {
            path: "/health".to_string(),
            method: "GET".to_string(),
            headers: HashMap::new(),
            body: vec![],
        };
        
        let response = router.route(request).await.unwrap();
        assert_eq!(response.status, 200);
        assert_eq!(response.body, b"OK");
    }
} 