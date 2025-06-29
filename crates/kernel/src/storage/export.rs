//! Framework export cho storage với associated type generics hiệu suất cao.
//!
//! Module này cung cấp một hệ thống export linh hoạt cho storage,
//! cho phép xuất dữ liệu theo nhiều định dạng và chiến lược khác nhau.
//! Thiết kế sử dụng associated type generics để đảm bảo type safety
//! và hiệu suất cao với zero-copy, lazy evaluation.

use async_trait::async_trait;
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use std::fmt::Debug;
use std::task::{Context, Poll};
use std::collections::VecDeque;
use crate::Error;
use serde_json;

// Định nghĩa thực thể Item dùng cho export và test
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Item {
    pub id: crate::Id,
    pub name: String,
    pub value: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Brief {
    pub id: crate::Id,
    pub name: String,
}

impl crate::Entity for Item {
    const NAME: &'static str = "test_items";
    type Key = crate::Id;
    type Index = Vec<u8>;
    type Summary = Brief;

    fn key(&self) -> Self::Key {
        self.id
    }
    fn index(&self) -> Self::Index {
        format!("idx_{}", self.value).into_bytes()
    }
    fn summary(&self) -> Self::Summary {
        Brief {
            id: self.id,
            name: self.name.clone(),
        }
    }
}

/// Trait định nghĩa khả năng export cho storage.
/// Sử dụng associated type generics để đảm bảo type safety.
#[async_trait]
pub trait Exportable: Send + Sync {
    /// Loại dữ liệu được export
    type Data: Serialize + DeserializeOwned + Send + Sync + 'static;
    
    /// Loại format xuất ra
    type Format: Serialize + DeserializeOwned + Send + Sync + 'static;
    
    /// Loại stream cho export
    type Stream: Send + Sync + 'static;
    
    /// Export dữ liệu theo format cụ thể
    async fn export(&self, format: Self::Format) -> Result<Self::Stream, Error>;
    
    /// Export một phần dữ liệu với filter
    async fn partial(&self, filter: Self::Data, format: Self::Format) -> Result<Self::Stream, Error>;
}

/// Trait định nghĩa khả năng transform dữ liệu trước khi export
#[async_trait]
pub trait Transformable: Send + Sync {
    /// Loại dữ liệu đầu vào
    type Input: Send + Sync + 'static;
    
    /// Loại dữ liệu đầu ra
    type Output: Send + Sync + 'static;
    
    /// Transform dữ liệu
    async fn transform(&self, input: Self::Input) -> Result<Self::Output, Error>;
}

/// Trait định nghĩa khả năng validate dữ liệu trước khi export
#[async_trait]
pub trait Validatable: Send + Sync {
    /// Loại dữ liệu cần validate
    type Data: Send + Sync + 'static;
    
    /// Validate dữ liệu
    async fn validate(&self, data: &Self::Data) -> Result<bool, Error>;
}

/// Cấu trúc cấu hình cho export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Số lượng item mỗi batch
    pub batch: usize,
    /// Timeout cho mỗi operation
    pub timeout: u64,
    /// Có compress dữ liệu không
    pub compress: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            batch: 1000,
            timeout: 30,
            compress: false,
        }
    }
}

/// Cấu trúc filter cho export
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Filter {
    /// Tiền tố để lọc
    pub prefix: Vec<u8>,
    /// Giới hạn số lượng
    pub limit: Option<usize>,
    /// Offset để phân trang
    pub offset: Option<usize>,
}

/// Cấu trúc format cho export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    /// Export dạng JSON
    Json,
    /// Export dạng CSV
    Csv,
    /// Export dạng binary
    Binary,
    /// Export dạng custom với config
    Custom(Config),
}

/// Async stream trait cho export hiệu suất cao
pub trait Streamable: Send + Sync {
    /// Đọc chunk dữ liệu
    fn read(&mut self, cx: &mut Context<'_>) -> Poll<Result<Option<Vec<u8>>, Error>>;
    
    /// Kiểm tra stream đã hết chưa
    fn done(&self) -> bool;
}

/// Cấu trúc stream cho export với zero-copy
pub struct Stream {
    /// Buffer dữ liệu
    buffer: VecDeque<Vec<u8>>,
    /// Vị trí hiện tại trong buffer
    pos: usize,
    /// Tổng kích thước
    size: usize,
    /// State của stream
    state: State,
}

#[derive(Debug)]
enum State {
    /// Đang đọc dữ liệu
    Reading,
    /// Đã hoàn thành
    Done,
    /// Lỗi
    Error(Error),
}

impl Stream {
    /// Tạo stream mới
    pub fn new() -> Self {
        Self {
            buffer: VecDeque::new(),
            pos: 0,
            size: 0,
            state: State::Reading,
        }
    }
    
    /// Thêm dữ liệu vào buffer
    pub fn push(&mut self, data: Vec<u8>) {
        self.size += data.len();
        self.buffer.push_back(data);
    }
    
    /// Đọc chunk dữ liệu (trả về Vec<u8> để tránh borrow đồng thời)
    pub fn read(&mut self, chunk: usize) -> Option<Vec<u8>> {
        if self.done() {
            return None;
        }
        if let Some(data) = self.buffer.front() {
            if self.pos < data.len() {
                let end = (self.pos + chunk).min(data.len());
                let result = data[self.pos..end].to_vec();
                self.pos = end;
                if self.pos >= data.len() {
                    self.buffer.pop_front();
                    self.pos = 0;
                }
                return Some(result);
            }
        }
        None
    }
    
    /// Kiểm tra stream đã hết chưa
    pub fn done(&self) -> bool {
        matches!(self.state, State::Done) && self.buffer.is_empty() && self.pos == 0
    }
    
    /// Reset stream về đầu
    pub fn reset(&mut self) {
        self.pos = 0;
        self.buffer.clear();
        self.size = 0;
        self.state = State::Reading;
    }
    
    /// Mark stream as done
    pub fn finish(&mut self) {
        self.state = State::Done;
    }
    
    /// Mark stream as error
    pub fn error(&mut self, error: Error) {
        self.state = State::Error(error);
    }
}

impl Default for Stream {
    fn default() -> Self {
        Self::new()
    }
}

impl Streamable for Stream {
    fn read(&mut self, _cx: &mut Context<'_>) -> Poll<Result<Option<Vec<u8>>, Error>> {
        match self.state {
            State::Done => Poll::Ready(Ok(None)),
            State::Error(ref e) => Poll::Ready(Err(Error::Io(std::io::Error::other(format!("stream error: {:?}", e))))),
            State::Reading => {
                if let Some(data) = self.buffer.front() {
                    if self.pos < data.len() {
                        let end = (self.pos + 1024).min(data.len());
                        let result = data[self.pos..end].to_vec();
                        self.pos = end;
                        if self.pos >= data.len() {
                            self.buffer.pop_front();
                            self.pos = 0;
                        }
                        Poll::Ready(Ok(Some(result)))
                    } else {
                        self.buffer.pop_front();
                        self.pos = 0;
                        Poll::Ready(Ok(None))
                    }
                } else {
                    Poll::Ready(Ok(None))
                }
            }
        }
    }
    fn done(&self) -> bool {
        matches!(self.state, State::Done) && self.buffer.is_empty() && self.pos == 0
    }
}

/// Cấu trúc export chính với zero-copy và lazy evaluation
#[derive(Clone)]
pub struct Export<S> {
    /// Storage backend reference
    storage: std::sync::Arc<S>,
    /// Cấu hình export
    config: Config,
}

impl<S> Export<S> {
    /// Tạo export mới
    pub fn new(storage: S, config: Config) -> Self {
        Self { 
            storage: std::sync::Arc::new(storage), 
            config 
        }
    }
    
    /// Tạo export với config mặc định
    pub fn default(storage: S) -> Self {
        Self {
            storage: std::sync::Arc::new(storage),
            config: Config::default(),
        }
    }
    
    /// Tạo export từ Arc storage với config mặc định
    pub fn arc(storage: std::sync::Arc<S>) -> Self {
        Self {
            storage,
            config: Config::default(),
        }
    }
    
    /// Lấy reference đến storage
    pub fn storage(&self) -> &S {
        &self.storage
    }
    
    /// Lấy config
    pub fn config(&self) -> &Config {
        &self.config
    }
}

#[async_trait]
impl<S: crate::storage::Storage> Exportable for Export<S>
where
    S: crate::storage::Storage + Send + Sync,
{
    type Data = Filter;
    type Format = Format;
    type Stream = Stream;
    
    async fn export(&self, format: Self::Format) -> Result<Self::Stream, Error> {
        let filter = Filter::default();
        self.partial(filter, format).await
    }
    
    async fn partial(&self, filter: Self::Data, format: Self::Format) -> Result<Self::Stream, Error> {
        // Implementation sẽ được triển khai dựa trên format
        match format {
            Format::Json => self.json(filter).await,
            Format::Csv => self.csv(filter).await,
            Format::Binary => self.binary(filter).await,
            Format::Custom(config) => self.custom(filter, config).await,
        }
    }
}

impl<S: crate::storage::Storage> Export<S> {
    /// Export dạng JSON
    async fn json(&self, filter: Filter) -> Result<Stream, Error> {
        let mut stream = Stream::new();
        let mut data = Vec::new();
        
        // Đọc dữ liệu từ storage
        let query = crate::storage::entity::Query {
            prefix: filter.prefix,
            after: None,
            limit: filter.limit.unwrap_or(1000),
        };
        let items = self.storage.as_ref().query::<Item>(query).await?;
        
        for item in items {
            let result = item?;
            let json = serde_json::to_string(&result)?;
            data.push(json);
        }
        
        // Tạo JSON content
        let content = format!("[{}]", data.join(","));
        stream.push(content.into_bytes());
        stream.finish();
        
        Ok(stream)
    }
    
    /// Export dạng CSV
    async fn csv(&self, filter: Filter) -> Result<Stream, Error> {
        let mut stream = Stream::new();
        let mut data = Vec::new();
        
        // Header CSV
        data.push("id,name".to_string());
        
        // Đọc dữ liệu từ storage
        let query = crate::storage::entity::Query {
            prefix: filter.prefix,
            after: None,
            limit: filter.limit.unwrap_or(1000),
        };
        let items = self.storage.as_ref().query::<Item>(query).await?;
        
        for item in items {
            let result = item?;
            let csv = format!("{},{}", result.id, result.name);
            data.push(csv);
        }
        
        // Tạo CSV content
        let content = data.join("\n");
        stream.push(content.into_bytes());
        stream.finish();
        
        Ok(stream)
    }
    
    /// Export dạng binary
    async fn binary(&self, filter: Filter) -> Result<Stream, Error> {
        let mut stream = Stream::new();
        let mut data = Vec::new();
        
        // Đọc dữ liệu từ storage
        let query = crate::storage::entity::Query {
            prefix: filter.prefix,
            after: None,
            limit: filter.limit.unwrap_or(1000),
        };
        let items = self.storage.as_ref().query::<Item>(query).await?;
        
        for item in items {
            let result = item?;
            let binary = bincode::serialize(&result)?;
            data.push(binary);
        }
        
        // Tạo binary content
        let content = bincode::serialize(&data)?;
        stream.push(content);
        stream.finish();
        
        Ok(stream)
    }
    
    /// Export dạng custom
    async fn custom(&self, filter: Filter, config: Config) -> Result<Stream, Error> {
        let mut stream = Stream::new();
        let mut data = Vec::new();
        
        // Đọc dữ liệu từ storage
        let query = crate::storage::entity::Query {
            prefix: filter.prefix,
            after: None,
            limit: config.batch,
        };
        let items = self.storage.as_ref().query::<Item>(query).await?;
        
        for item in items {
            let result = item?;
            let custom = serde_json::to_string(&result)?;
            data.push(custom);
        }
        
        // Tạo custom content với config
        let format = serde_json::to_string(&config)?;
        let content = format!("{{\"config\":{},\"data\":[{}]}}", format, data.join(","));
        stream.push(content.into_bytes());
        stream.finish();
        
        Ok(stream)
    }
}

/// Cấu trúc builder cho export với fluent API
pub struct Builder {
    config: Config,
    filter: Filter,
    format: Format,
}

impl Builder {
    /// Tạo builder mới
    pub fn new() -> Self {
        Self {
            config: Config::default(),
            filter: Filter::default(),
            format: Format::Json,
        }
    }
    
    /// Thiết lập config
    pub fn config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }
    
    /// Thiết lập filter
    pub fn filter(mut self, filter: Filter) -> Self {
        self.filter = filter;
        self
    }
    
    /// Thiết lập format
    pub fn format(mut self, format: Format) -> Self {
        self.format = format;
        self
    }
    
    /// Build export
    pub fn build<S>(self, storage: S) -> Export<S> {
        Export::new(storage, self.config)
    }
    
    /// Build export từ Arc storage
    pub fn buildarc<S>(self, storage: std::sync::Arc<S>) -> Export<S> {
        Export::arc(storage)
    }
}

impl Default for Builder {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait cho storage để dễ sử dụng
pub trait Ext: crate::storage::Storage + Send + Sync + Clone {
    /// Tạo export builder
    fn export(&self) -> Builder {
        Builder::new()
    }
    
    /// Export với format cụ thể
    fn exportas(self, format: Format) -> impl std::future::Future<Output = Result<Stream, Error>> + Send where Self: Sized {
        async move {
            let export = Export::default(self);
            export.export(format).await
        }
    }
    
    /// Export với format cụ thể từ Arc
    fn exportasarc(self: std::sync::Arc<Self>, format: Format) -> impl std::future::Future<Output = Result<Stream, Error>> + Send {
        async move {
            let export = Export::arc(self);
            export.export(format).await
        }
    }
}

impl<S> Ext for S where S: crate::storage::Storage + Send + Sync + Clone {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{storage::sled::Sled, Storage};
    use tempfile::tempdir;
    use crate::Id;
    
    /// Helper function tạo test data
    fn items(count: usize) -> Vec<Item> {
        (0..count).map(|i| Item {
            id: Id::new_v4(),
            name: format!("test_{}", i),
            value: i as u32,
        }).collect()
    }
    
    #[tokio::test]
    async fn builder() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        
        let export = Builder::new()
            .config(Config { batch: 500, timeout: 60, compress: true })
            .format(Format::Json)
            .build(storage);
            
        assert_eq!(export.config.batch, 500);
        assert_eq!(export.config.timeout, 60);
        assert!(export.config.compress);
    }
    
    #[tokio::test]
    async fn stream() {
        let mut stream = Stream::new();
        
        // Test push and read
        stream.push(b"test data".to_vec());
        let chunk = stream.read(4);
        assert_eq!(chunk, Some(b"test".to_vec()));
        
        // Test done
        assert!(!stream.done());
        
        // Test finish
        stream.finish();
        // Đọc hết buffer với giới hạn số lần lặp để tránh treo
        let mut count = 0;
        while stream.read(1024).is_some() && count < 10 {
            count += 1;
        }
        assert!(stream.done());
        
        // Test reset
        stream.reset();
        assert!(!stream.done());
    }
    
    #[tokio::test]
    async fn formats() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storage = std::sync::Arc::new(storage);
        let export = Export::arc(storage);
        
        // Test all formats
        let json = export.export(Format::Json).await.unwrap();
        assert!(!json.done());
        
        let csv = export.export(Format::Csv).await.unwrap();
        assert!(!csv.done());
        
        let binary = export.export(Format::Binary).await.unwrap();
        assert!(!binary.done());
        
        let config = Config { batch: 100, timeout: 10, compress: false };
        let custom = export.export(Format::Custom(config)).await.unwrap();
        assert!(!custom.done());
    }
    
    #[tokio::test]
    async fn filter() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storage = std::sync::Arc::new(storage);
        let export = Export::arc(storage);
        
        let filter = Filter {
            prefix: b"test_".to_vec(),
            limit: Some(50),
            offset: Some(0),
        };
        
        let stream = export.partial(filter, Format::Json).await.unwrap();
        assert!(!stream.done());
    }
    
    #[tokio::test]
    async fn concurrent() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storage = std::sync::Arc::new(storage);
        let export = Export::arc(storage);
        
        // Test concurrent exports
        let mut handles = Vec::new();
        
        for _ in 0..3 {
            let clone = export.clone();
            let handle = tokio::spawn(async move {
                let stream = clone.export(Format::Json).await.unwrap();
                assert!(!stream.done());
            });
            handles.push(handle);
        }
        
        for handle in handles {
            handle.await.unwrap();
        }
    }
    
    #[tokio::test]
    async fn performance() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storage = std::sync::Arc::new(storage);
        let export = Export::arc(storage);
        
        // Test performance với nhiều format
        let start = std::time::Instant::now();
        
        for _ in 0..100 {
            let stream = export.export(Format::Json).await.unwrap();
            assert!(!stream.done());
        }
        
        let duration = start.elapsed();
        assert!(duration.as_millis() < 1000); // Phải hoàn thành trong 1 giây
    }
    
    #[tokio::test]
    async fn recovery() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storage = std::sync::Arc::new(storage);
        let export = Export::arc(storage);
        
        // Test recovery từ lỗi
        let mut stream = export.export(Format::Json).await.unwrap();
        stream.error(Error::Parse("test error".to_string()));
        
        // Reset và thử lại
        stream.reset();
        assert!(!stream.done());
    }
    
    #[tokio::test]
    async fn integration() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storagearc = std::sync::Arc::new(storage);
        let export = Export::arc(storagearc.clone());
        
        // Test integration với storage
        let items = items(10);
        for item in items {
            storagearc.as_ref().insert(item).await.unwrap();
        }
        
        let stream = export.export(Format::Json).await.unwrap();
        assert!(!stream.done());
    }
    
    #[tokio::test]
    async fn speed() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storagearc = std::sync::Arc::new(storage);
        let export = Export::arc(storagearc.clone());
        
        // Test speed với large data
        let items = items(1000);
        for item in items {
            storagearc.as_ref().insert(item).await.unwrap();
        }
        
        let start = std::time::Instant::now();
        let stream = export.export(Format::Json).await.unwrap();
        let duration = start.elapsed();
        
        assert!(duration.as_millis() < 100); // Phải nhanh
        assert!(!stream.done());
    }
    
    #[tokio::test]
    async fn group() {
        let dir = tempdir().unwrap();
        let path = dir.path().to_str().unwrap();
        let storage = Sled::new(path).unwrap();
        let storagearc = std::sync::Arc::new(storage);
        let export = Export::arc(storagearc.clone());
        
        // Test group operations
        let items = items(100);
        for item in items {
            storagearc.as_ref().insert(item).await.unwrap();
        }
        
        let filter = Filter {
            prefix: b"test_".to_vec(),
            limit: Some(50),
            offset: Some(0),
        };
        
        let stream = export.partial(filter, Format::Json).await.unwrap();
        assert!(!stream.done());
    }
}
