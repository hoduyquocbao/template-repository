//! Quản lý pool kết nối cho storage backend.
//!
//! Module này cung cấp một pool kết nối thread-safe và hiệu quả
//! cho việc tái sử dụng các kết nối database.

use std::sync::Arc;
use tokio::sync::Semaphore;
use crate::Error;

/// Pool quản lý các kết nối database
#[derive(Clone)]
pub struct Pool<T> {
    /// Danh sách các kết nối có sẵn
    conn: Arc<Vec<T>>,
    /// Semaphore để kiểm soát số lượng kết nối đồng thời
    sem: Arc<Semaphore>,
}

impl<T: Clone + Send + Sync + 'static> Pool<T> {
    /// Tạo pool mới với số lượng kết nối cố định
    pub fn new(size: usize, init: impl Fn() -> Result<T, Error>) -> Result<Self, Error> {
        let mut conn = Vec::with_capacity(size);
        for _ in 0..size {
            conn.push(init()?);
        }
        
        Ok(Self {
            conn: Arc::new(conn),
            sem: Arc::new(Semaphore::new(size)),
        })
    }
    
    /// Lấy một kết nối từ pool
    pub async fn get(&self) -> Result<T, Error> {
        let _permit = self.sem.acquire().await.map_err(|_| Error::Input)?;
        Ok(self.conn[0].clone())
    }
    
    /// Trả về số lượng kết nối có sẵn
    pub fn free(&self) -> usize {
        self.sem.available_permits()
    }
} 