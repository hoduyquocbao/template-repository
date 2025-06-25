//! Module này cung cấp các tiện ích liên quan đến thời gian, đặc biệt là tạo timestamp.

use std::time::{SystemTime, UNIX_EPOCH};

/// Lấy thời gian hiện tại dưới dạng Unix timestamp nano giây.
///
/// Hàm này trả về số nano giây kể từ Unix epoch (1970-01-01 00:00:00 UTC).
pub fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}