use crate::error::{Error, ValidationError};
use std::ops::{Deref, DerefMut};
use std::sync::Arc;
use tokio::sync::{Semaphore, SemaphorePermit};

/// Pool quản lý các kết nối database
impl<T: Clone + Send + Sync + 'static> Pool<T> {
    /// Tạo pool mới với số lượng kết nối cố định
    pub async fn get(&self) -> Result<T, Error> {
        let _permit = self
            .sem
            .acquire()
            .await
            .map_err(|_| Error::Validation(vec![ValidationError {
                field: "pool".to_string(),
                message: "Không thể lấy permit từ semaphore.".to_string()
            }]))?;
        Ok(self.conn[0].clone())
    }

    /// Trả về số lượng kết nối có sẵn (permit chưa dùng)
} 