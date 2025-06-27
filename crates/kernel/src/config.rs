//! Configuration System cho Framework
//!
//! Config system quản lý cấu hình của framework một cách linh hoạt.
//! Tuân thủ nguyên tắc đơn từ và hiệu suất theo thiết kế.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Configuration cho Framework
/// 
/// Config quản lý tất cả cấu hình của framework bao gồm:
/// - Database settings
/// - Logging settings
/// - Plugin settings
/// - Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Database configuration
    pub database: Database,
    /// Logging configuration
    pub log: Log,
    /// Plugin configuration
    pub addon: Addon,
    /// Performance configuration
    pub performance: Performance,
    /// Custom settings
    pub custom: HashMap<String, String>,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    /// Database path
    pub path: String,
    /// Connection pool
    pub pool: usize,
    /// Cache
    pub cache: usize,
    /// Enable metrics
    pub metrics: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Log {
    /// Log level
    pub level: String,
    /// Log file path
    pub file: Option<String>,
    /// Enable console output
    pub console: bool,
    /// Enable structured logging
    pub structured: bool,
}

/// Plugin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Addon {
    /// Plugin directory
    pub dir: String,
    /// Auto load plugins
    pub auto: bool,
    /// Plugin timeout
    pub timeout: u64,
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Performance {
    /// Worker threads
    pub worker: usize,
    /// Buffer
    pub buffer: usize,
    /// Enable profiling
    pub profiling: bool,
}

impl Config {
    /// Tạo config mới với giá trị mặc định
    pub fn new() -> Self {
        Self {
            database: Database::default(),
            log: Log::default(),
            addon: Addon::default(),
            performance: Performance::default(),
            custom: HashMap::new(),
        }
    }
    
    /// Load config từ file
    pub fn load(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = serde_json::from_str(&content)?;
        Ok(config)
    }
    
    /// Save config ra file
    pub fn save(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
    
    /// Lấy custom setting
    pub fn get(&self, key: &str) -> Option<&String> {
        self.custom.get(key)
    }
    
    /// Set custom setting
    pub fn set(&mut self, key: String, value: String) {
        self.custom.insert(key, value);
    }
    
    /// Merge config khác
    pub fn merge(&mut self, other: Config) {
        self.database = other.database;
        self.log = other.log;
        self.addon = other.addon;
        self.performance = other.performance;
        for (k, v) in other.custom {
            self.custom.insert(k, v);
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for Database {
    fn default() -> Self {
        Self {
            path: "./db".to_string(),
            pool: 10,
            cache: 1000,
            metrics: true,
        }
    }
}

impl Default for Log {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            console: true,
            structured: true,
        }
    }
}

impl Default for Addon {
    fn default() -> Self {
        Self {
            dir: "./plugins".to_string(),
            auto: false,
            timeout: 30,
        }
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self {
            worker: num_cpus::get(),
            buffer: 1024,
            profiling: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create() {
        let config = Config::new();
        assert_eq!(config.database.path, "./db");
        assert_eq!(config.log.level, "info");
        assert!(!config.addon.auto);
        assert_eq!(config.performance.worker, num_cpus::get());
    }

    #[test]
    fn custom() {
        let mut config = Config::new();
        
        // Test set
        config.set("test".to_string(), "value".to_string());
        
        // Test get
        let value = config.get("test");
        assert_eq!(value, Some(&"value".to_string()));
        
        // Test get non-existent
        let value = config.get("none");
        assert_eq!(value, None);
    }

    #[test]
    fn merge() {
        let mut config1 = Config::new();
        config1.set("key1".to_string(), "value1".to_string());
        
        let mut config2 = Config::new();
        config2.set("key2".to_string(), "value2".to_string());
        
        config1.merge(config2);
        
        assert_eq!(config1.get("key1"), Some(&"value1".to_string()));
        assert_eq!(config1.get("key2"), Some(&"value2".to_string()));
    }
} 