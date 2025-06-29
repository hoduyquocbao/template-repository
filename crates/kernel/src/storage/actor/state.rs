//! Module quản lý trạng thái nội bộ cho Actor
//! - State: Idle, Running, Stopped, Error
//! - Cell: Arc<Mutex<State>> để chia sẻ trạng thái giữa thread và API

use std::sync::{Arc, Mutex};

/// Trạng thái động của Actor
#[derive(Debug, Clone, PartialEq)]
pub enum State {
    Idle,      // Chưa nhận message nào
    Running,   // Đang xử lý message
    Stopped,   // Đã dừng thread
    Error,     // Gặp lỗi nghiêm trọng
}

/// Wrapper cho state thread-safe
#[derive(Clone)]
pub struct Cell(pub Arc<Mutex<State>>);

impl Cell {
    /// Tạo Cell mới với trạng thái ban đầu
    pub fn new(init: State) -> Self {
        Self(Arc::new(Mutex::new(init)))
    }
    /// Lấy trạng thái hiện tại
    pub fn get(&self) -> State {
        self.0.lock().unwrap().clone()
    }
    /// Đặt trạng thái mới
    pub fn set(&self, state: State) {
        *self.0.lock().unwrap() = state;
    }
}

// Hiện tại Actor không có state riêng biệt ngoài state của Sled/Handle, file này để chuẩn bị cho việc mở rộng stateful actor nếu cần. 