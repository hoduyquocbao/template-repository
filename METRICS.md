# Hướng Dẫn Sử Dụng Feature Metrics

## Tổng Quan

Feature `metrics` đã được thêm vào tất cả các crates trong workspace để cung cấp khả năng bật/tắt hệ thống đo lường hiệu suất một cách linh hoạt.

## Các Crates Hỗ Trợ Metrics

Tất cả các crates sau đều có feature `metrics`:

- `repository` - Hệ thống lưu trữ với Actor pattern và metric collection
- `shared` - Các thành phần chia sẻ chung
- `task` - Quản lý task với metric tracking
- `memories` - Quản lý memories với metric tracking  
- `architecture` - Quản lý kiến trúc với metric tracking
- `knowledge` - Facade layer với metric aggregation
- `naming` - Tool kiểm tra naming convention

## Cách Sử Dụng

### 1. Bật Metrics cho Crate Cụ Thể

```bash
# Build với metrics cho repository
cargo build --package repository --features metrics

# Build với metrics cho task
cargo build --package task --features metrics

# Build với metrics cho toàn bộ workspace
cargo build --workspace --features metrics
```

### 2. Chạy Test với Metrics

```bash
# Test repository với metrics
cargo test --package repository --features metrics

# Test toàn bộ workspace với metrics
cargo test --workspace --features metrics
```

### 3. Chạy Benchmark với Metrics

```bash
# Benchmark task với metrics
cargo bench --package task --features metrics

# Benchmark repository với metrics
cargo bench --package repository --features metrics
```

### 4. Chạy Application với Metrics

```bash
# Chạy knowledge CLI với metrics
cargo run --package knowledge --features metrics -- add task "Test task"

# Chạy naming tool với metrics
cargo run --package naming --features metrics -- crates/repository/src
```

## Cấu Trúc Metrics

### Repository Metrics

Repository crate cung cấp các metric sau:

- **insert** - Thao tác thêm dữ liệu
- **fetch** - Thao tác lấy dữ liệu  
- **update** - Thao tác cập nhật dữ liệu
- **delete** - Thao tác xóa dữ liệu
- **query** - Thao tác truy vấn dữ liệu
- **mass** - Thao tác bulk operations
- **keys** - Thao tác lấy danh sách keys

Mỗi metric bao gồm:
- Tổng số lần thực thi
- Số lần thành công/thất bại
- Thời gian trung bình thực thi

### Business Logic Metrics

Các crate nghiệp vụ (task, memories, architecture) có thể tích hợp với repository metrics để theo dõi:

- Hiệu suất các thao tác CRUD
- Tỷ lệ thành công/thất bại
- Thời gian phản hồi trung bình
- Concurrent access patterns

## Conditional Compilation

Feature metrics sử dụng conditional compilation để chỉ include code metrics khi cần thiết:

```rust
#[cfg(feature = "metrics")]
use crate::metric::Registry;

#[cfg(feature = "metrics")]
pub struct Actor {
    metric: Registry,
    // ... other fields
}

#[cfg(not(feature = "metrics"))]
pub struct Actor {
    // ... fields without metrics
}
```

## Performance Impact

### Khi Metrics Disabled (Default)
- Không có overhead về memory
- Không có overhead về CPU
- Code metrics được loại bỏ hoàn toàn

### Khi Metrics Enabled
- Memory overhead: ~100 bytes per metric type
- CPU overhead: <1% cho atomic operations
- Network overhead: Chỉ khi export metrics

## Monitoring và Alerting

### Real-time Monitoring
```bash
# Xem metrics real-time
cargo run --package knowledge --features metrics -- metrics show

# Export metrics cho Prometheus
cargo run --package knowledge --features metrics -- metrics export
```

### Alerting Rules
- Tỷ lệ lỗi > 5% trong 5 phút
- Thời gian phản hồi > 100ms trung bình
- Concurrent connections > 1000

## Best Practices

### 1. Development
```bash
# Development với metrics để debug
cargo run --features metrics

# Production build không có metrics
cargo build --release
```

### 2. Testing
```bash
# Unit tests với metrics
cargo test --features metrics

# Integration tests với metrics
cargo test --features metrics --test integration
```

### 3. CI/CD
```yaml
# GitHub Actions example
- name: Test with metrics
  run: cargo test --workspace --features metrics

- name: Build production
  run: cargo build --release
```

## Troubleshooting

### Metrics Không Hiển Thị
1. Kiểm tra feature được bật: `cargo build --features metrics`
2. Kiểm tra code sử dụng `#[cfg(feature = "metrics")]`
3. Kiểm tra Registry được khởi tạo đúng cách

### Performance Issues
1. Disable metrics trong production: `cargo build --release`
2. Kiểm tra atomic operations không bị contention
3. Monitor memory usage của Registry

### Integration Issues
1. Đảm bảo tất cả crates có feature metrics
2. Kiểm tra dependency chain
3. Verify conditional compilation flags

## Future Enhancements

- [ ] Prometheus exporter
- [ ] Grafana dashboards
- [ ] Custom metric types
- [ ] Distributed tracing
- [ ] Performance profiling
- [ ] Alerting integration 