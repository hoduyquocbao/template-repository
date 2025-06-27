#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Validator
//!
//! Cung cấp hệ thống xác thực dữ liệu (validation) cho framework, hỗ trợ rule cho text/number.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::{validator::System, validator::Text, validator::Number};
//!
//! let validator = System::new();
//! let result = validator.text("hello", &[Text::Min(3), Text::Max(10)]);
//! assert!(result.is_ok());
//! let result = validator.number(&5.0, &[Number::Min(1.0), Number::Max(10.0)]);
//! assert!(result.is_ok());
//! ```

/// Validation error
#[derive(Debug, Clone)]
pub struct Error {
    /// Field name
    pub field: String,
    /// Error message
    pub message: String,
}

/// Validation result
pub type Result = std::result::Result<(), Vec<Error>>;

/// Validator trait
pub trait Validator<T> {
    /// Validate data
    fn validate(&self, data: &T) -> Result;
}

/// Validator cho Framework
/// 
/// Validator cung cấp các phương thức validation cho các kiểu dữ liệu khác nhau.
pub struct System {
    // map: HashMap<String, Box<dyn std::any::Any + Send + Sync>>, // TODO: Dành cho mở rộng custom rule
}

impl System {
    /// Tạo validator system mới
    pub fn new() -> Self {
        Self {
            // map: HashMap::new(),
        }
    }
    
    /// Đăng ký validator
    pub fn register<T: 'static + Send + Sync>(&mut self, _name: String, _validator: Box<dyn Validator<T>>) {
        // TODO: Implement validator registration
    }
    
    /// Validate string
    pub fn text(&self, value: &str, rule: &[Text]) -> Result {
        let mut errors = Vec::new();
        
        for r in rule {
            match r {
                Text::Required => {
                    if value.trim().is_empty() {
                        errors.push(Error {
                            field: "text".to_string(),
                            message: "Field is required".to_string(),
                        });
                    }
                }
                Text::Min(min) => {
                    if value.len() < *min {
                        errors.push(Error {
                            field: "text".to_string(),
                            message: format!("Minimum length is {}", min),
                        });
                    }
                }
                Text::Max(max) => {
                    if value.len() > *max {
                        errors.push(Error {
                            field: "text".to_string(),
                            message: format!("Maximum length is {}", max),
                        });
                    }
                }
                Text::Pattern(pattern) => {
                    if value.matches(pattern).next().is_none() {
                        errors.push(Error {
                            field: "text".to_string(),
                            message: format!("Must match pattern: {}", pattern),
                        });
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    /// Validate number
    pub fn number(&self, value: &f64, rule: &[Number]) -> Result {
        let mut errors = Vec::new();
        
        for r in rule {
            match r {
                Number::Required => {
                    // Numbers are always present if passed
                }
                Number::Min(min) => {
                    if *value < *min {
                        errors.push(Error {
                            field: "number".to_string(),
                            message: format!("Minimum value is {}", min),
                        });
                    }
                }
                Number::Max(max) => {
                    if *value > *max {
                        errors.push(Error {
                            field: "number".to_string(),
                            message: format!("Maximum value is {}", max),
                        });
                    }
                }
                Number::Positive => {
                    if *value <= 0.0 {
                        errors.push(Error {
                            field: "number".to_string(),
                            message: "Must be positive".to_string(),
                        });
                    }
                }
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// String validation rules
#[derive(Debug, Clone)]
pub enum Text {
    Required,
    Min(usize),
    Max(usize),
    Pattern(String),
}

/// Number validation rules
#[derive(Debug, Clone)]
pub enum Number {
    Required,
    Min(f64),
    Max(f64),
    Positive,
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn text() {
        let validator = System::new();
        
        // Test required
        let result = validator.text("", &[Text::Required]);
        assert!(result.is_err());
        
        // Test min
        let result = validator.text("abc", &[Text::Min(5)]);
        assert!(result.is_err());
        
        // Test max
        let result = validator.text("abcdef", &[Text::Max(3)]);
        assert!(result.is_err());
        
        // Test valid
        let result = validator.text("hello", &[Text::Min(3), Text::Max(10)]);
        assert!(result.is_ok());
    }

    #[test]
    fn number() {
        let validator = System::new();
        
        // Test min
        let result = validator.number(&5.0, &[Number::Min(10.0)]);
        assert!(result.is_err());
        
        // Test max
        let result = validator.number(&15.0, &[Number::Max(10.0)]);
        assert!(result.is_err());
        
        // Test positive
        let result = validator.number(&-5.0, &[Number::Positive]);
        assert!(result.is_err());
        
        // Test valid
        let result = validator.number(&5.0, &[Number::Min(1.0), Number::Max(10.0), Number::Positive]);
        assert!(result.is_ok());
    }
} 