# Framework Export cho Storage

## Tổng Quan

Framework export cho storage cung cấp một hệ thống linh hoạt và mạnh mẽ để xuất dữ liệu từ các backend storage khác nhau. Thiết kế sử dụng associated type generics để đảm bảo type safety và hiệu suất cao.

## Kiến Trúc

### Core Traits

#### `Exportable`
Trait chính định nghĩa khả năng export cho storage:

```rust
#[async_trait]
pub trait Exportable: Send + Sync {
    type Data: Serialize + DeserializeOwned + Send + Sync + 'static;
    type Format: Serialize + DeserializeOwned + Send + Sync + 'static;
    type Stream: Send + Sync + 'static;
    
    async fn export(&self, format: Self::Format) -> Result<Self::Stream, Error>;
    async fn partial(&self, filter: Self::Data, format: Self::Format) -> Result<Self::Stream, Error>;
}
```

#### `Transformable`
Trait cho phép transform dữ liệu trước khi export:

```rust
#[async_trait]
pub trait Transformable: Send + Sync {
    type Input: Send + Sync + 'static;
    type Output: Send + Sync + 'static;
    
    async fn transform(&self, input: Self::Input) -> Result<Self::Output, Error>;
}
```

#### `Validatable`
Trait cho phép validate dữ liệu trước khi export:

```rust
#[async_trait]
pub trait Validatable: Send + Sync {
    type Data: Send + Sync + 'static;
    
    async fn validate(&self, data: &Self::Data) -> Result<bool, Error>;
}
```

### Cấu Trúc Dữ Liệu

#### `Config`
Cấu hình cho export:
- `batch`: Số lượng item mỗi batch
- `timeout`: Timeout cho mỗi operation
- `compress`: Có compress dữ liệu không

#### `Filter`
Filter cho export:
- `prefix`: Tiền tố để lọc
- `limit`: Giới hạn số lượng
- `offset`: Offset để phân trang

#### `Format`
Các định dạng export hỗ trợ:
- `Json`: Export dạng JSON
- `Csv`: Export dạng CSV
- `Binary`: Export dạng binary
- `Custom(Config)`: Export dạng custom với config

#### `Stream`
Stream cho export:
- `data`: Dữ liệu stream
- `pos`: Vị trí hiện tại
- `size`: Tổng kích thước

### Builder Pattern

```rust
let export = Builder::new()
    .config(Config { batch: 500, timeout: 60, compress: true })
    .format(Format::Json)
    .build(storage);
```

## Sử Dụng

### Export Cơ Bản

```rust
use crate::storage::{Export, Format, Builder};

// Tạo export với config mặc định
let export = Export::default(storage);

// Export toàn bộ dữ liệu dạng JSON
let stream = export.export(Format::Json).await?;
```

### Export Với Filter

```rust
use crate::storage::{Filter, Format};

let filter = Filter {
    prefix: b"user_".to_vec(),
    limit: Some(100),
    offset: Some(0),
};

let stream = export.partial(filter, Format::Csv).await?;
```

### Export Custom

```rust
let config = Config {
    batch: 1000,
    timeout: 30,
    compress: true,
};

let stream = export.export(Format::Custom(config)).await?;
```

## Hiệu Suất

### Tối Ưu Hóa

1. **Batch Processing**: Xử lý dữ liệu theo batch để giảm memory usage
2. **Streaming**: Sử dụng stream để tránh load toàn bộ dữ liệu vào memory
3. **Compression**: Hỗ trợ nén dữ liệu để giảm kích thước
4. **Async/Await**: Sử dụng async để không block thread

### Benchmark

Framework được thiết kế để xử lý hiệu quả với:
- Hàng triệu bản ghi
- Dữ liệu lớn (GB/TB)
- Concurrent access
- Real-time streaming

## Mở Rộng

### Thêm Format Mới

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Format {
    Json,
    Csv,
    Binary,
    Custom(Config),
    // Thêm format mới
    Xml,
    Yaml,
}
```

### Thêm Transform

```rust
impl Transformable for CustomTransform {
    type Input = Vec<u8>;
    type Output = Vec<u8>;
    
    async fn transform(&self, input: Self::Input) -> Result<Self::Output, Error> {
        // Logic transform
        Ok(input)
    }
}
```

### Thêm Validation

```rust
impl Validatable for CustomValidator {
    type Data = Vec<u8>;
    
    async fn validate(&self, data: &Self::Data) -> Result<bool, Error> {
        // Logic validation
        Ok(true)
    }
}
```

## Testing

### Unit Tests

```rust
#[tokio::test]
async fn test_export_builder() {
    let export = Builder::new()
        .config(Config { batch: 500, timeout: 60, compress: true })
        .format(Format::Json)
        .build(storage);
        
    assert_eq!(export.config.batch, 500);
    assert_eq!(export.config.timeout, 60);
    assert!(export.config.compress);
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_export_integration() {
    let storage = create_test_storage();
    let export = Export::default(storage);
    
    // Test export
    let stream = export.export(Format::Json).await.unwrap();
    assert!(!stream.done());
}
```

## Best Practices

1. **Sử dụng Builder Pattern**: Để tạo export với cấu hình phức tạp
2. **Validate Input**: Luôn validate dữ liệu trước khi export
3. **Handle Errors**: Xử lý lỗi một cách graceful
4. **Monitor Performance**: Theo dõi hiệu suất export
5. **Use Streaming**: Sử dụng stream cho dữ liệu lớn

## Troubleshooting

### Common Issues

1. **Memory Usage**: Sử dụng batch processing và streaming
2. **Timeout**: Tăng timeout trong config
3. **Format Errors**: Validate format trước khi export
4. **Performance**: Sử dụng compression và optimize batch size

### Debug

```rust
// Enable debug logging
tracing::debug!("Exporting with config: {:?}", config);

// Check stream status
if stream.done() {
    tracing::info!("Export completed");
}
``` 