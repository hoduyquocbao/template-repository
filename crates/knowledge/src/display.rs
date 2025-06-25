//! Module chứa các tiện ích hiển thị chung cho các loại Summary.

use repository::Error; // Import Error để sử dụng trong Result
use shared::Showable; // Import Showable để sử dụng trong Result

/// Hàm trợ giúp chung để in Summary.
/// Mục đích: Hiển thị các bản tóm tắt một cách nhất quán.
pub fn show<S>(iter: Box<dyn Iterator<Item = Result<S, Error>> + Send>) -> Result<(), Error>
where
    S: std::fmt::Debug, // Cần Debug để in
    S: Showable, // Yêu cầu S phải triển khai Showable
{
    let mut count = 0;
    for result in iter {
        match result {
            Ok(summary) => {
                summary.show(); // Gọi phương thức show từ trait
                count += 1;
            }
            Err(e) => return Err(e),
        }
    }
    if count == 0 {
        println!("Không tìm thấy bản ghi nào.");
    }
    Ok(())
}