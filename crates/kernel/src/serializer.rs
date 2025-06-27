#![cfg_attr(doctest, allow(unused_imports))]
//! # Module Serializer
//!
//! Cung cấp hệ thống tuần tự hóa (serialization) cho framework, hỗ trợ JSON, Bincode.
//!
//! ## Ví dụ sử dụng
//! ```rust,ignore
//! use kernel::serializer::System;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize, PartialEq, Debug)]
//! struct Data { name: String, value: i32 }
//!
//! let system = System::new();
//! let data = Data { name: "test".to_string(), value: 42 };
//! let json = system.json(&data).unwrap();
//! let parsed: Data = system.parse(&json).unwrap();
//! assert_eq!(data, parsed);
//! ```

use serde::{Serialize, Deserialize};

/// Trait tuần tự hóa generic
pub trait Serializer<T> {
    /// Serialize data
    fn serialize(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>>;
    
    /// Deserialize data
    fn deserialize(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>>;
}

/// Serializer cho JSON
pub struct Json;

impl<T: Serialize + for<'de> Deserialize<'de>> Serializer<T> for Json {
    fn serialize(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let json = serde_json::to_string(data)?;
        Ok(json.into_bytes())
    }
    
    fn deserialize(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        let json = String::from_utf8(bytes.to_vec())?;
        let data = serde_json::from_str(&json)?;
        Ok(data)
    }
}

/// Serializer cho Bincode
pub struct Bincode;

impl<T: Serialize + for<'de> Deserialize<'de>> Serializer<T> for Bincode {
    fn serialize(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let bytes = bincode::serialize(data)?;
        Ok(bytes)
    }
    
    fn deserialize(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        let data = bincode::deserialize(bytes)?;
        Ok(data)
    }
}

/// Hệ thống tuần tự hóa cho framework
///
/// Hỗ trợ encode/decode JSON, Bincode động.
pub struct System {
    // map: std::collections::HashMap<String, Box<dyn std::any::Any + Send + Sync>>, // TODO: Dành cho mở rộng custom serializer
}

impl System {
    /// Tạo serializer system mới
    pub fn new() -> Self {
        Self {
            // map: std::collections::HashMap::new(),
        }
    }
    
    /// Serialize to JSON
    pub fn json<T: Serialize + for<'de> Deserialize<'de>>(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let serializer = Json;
        serializer.serialize(data)
    }
    
    /// Deserialize from JSON
    pub fn parse<T: for<'de> Deserialize<'de> + Serialize>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        let serializer = Json;
        serializer.deserialize(bytes)
    }
    
    /// Serialize to Bincode
    pub fn encode<T: Serialize + for<'de> Deserialize<'de>>(&self, data: &T) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let serializer = Bincode;
        serializer.serialize(data)
    }
    
    /// Deserialize from Bincode
    pub fn decode<T: for<'de> Deserialize<'de> + Serialize>(&self, bytes: &[u8]) -> Result<T, Box<dyn std::error::Error>> {
        let serializer = Bincode;
        serializer.deserialize(bytes)
    }
}

impl Default for System {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde::{Serialize, Deserialize};

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    struct Data {
        name: String,
        value: i32,
    }

    #[test]
    fn json() {
        let serializer = Json;
        let data = Data {
            name: "test".to_string(),
            value: 42,
        };
        
        // Test serialize
        let bytes = serializer.serialize(&data).unwrap();
        
        // Test deserialize
        let parsed = serializer.deserialize(&bytes).unwrap();
        assert_eq!(data, parsed);
    }

    #[test]
    fn bincode() {
        let serializer = Bincode;
        let data = Data {
            name: "test".to_string(),
            value: 42,
        };
        
        // Test serialize
        let bytes = serializer.serialize(&data).unwrap();
        
        // Test deserialize
        let parsed = serializer.deserialize(&bytes).unwrap();
        assert_eq!(data, parsed);
    }

    #[test]
    fn system() {
        let system = System::new();
        let data = Data {
            name: "test".to_string(),
            value: 42,
        };
        
        // Test JSON
        let json = system.json(&data).unwrap();
        let parsed = system.parse::<Data>(&json).unwrap();
        assert_eq!(data, parsed);
        
        // Test Bincode
        let bin = system.encode(&data).unwrap();
        let parsed = system.decode::<Data>(&bin).unwrap();
        assert_eq!(data, parsed);
    }
} 