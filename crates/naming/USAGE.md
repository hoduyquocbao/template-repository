### Ignore động path/module/file
- Thêm vào `naming.toml`:
```toml
ignore = ["src/tests/", "src/bin/", "mod_test.rs"]
```
- Mọi file có path chứa chuỗi trong danh sách ignore sẽ được bỏ qua hoàn toàn (không kiểm tra, không báo lỗi).
- Lưu ý: ignore là so khớp chuỗi con, có thể dùng cho path hoặc tên file/module. 

### Bật/tắt rule kiểm tra qua config
- Thêm vào `naming.toml`:
```toml
enable_pascal = true
enable_snake = false
enable_camel = true
```
- Có thể bật/tắt từng rule kiểm tra (PascalCase, snake_case, camelCase) cho từng thư mục/module.
- Nếu không khai báo, mặc định là bật (true). 

### Kiểm thử đa môi trường & edge-case
- Chạy script kiểm thử matrix:
```sh
bash test_matrix.sh
```
- Sinh file test edge-case tự động:
```sh
python3 gen_test_edge.py
```
- Các file test edge-case: test_edge_big.rs, test_edge_macro.rs, test_edge_empty.rs, test_edge_locked.rs, test_edge_denied.rs
- Có thể thêm các file này vào kiểm thử để phát hiện rủi ro thực tế, bottleneck, lỗi I/O, memory. 

### Rule kiểm tra định danh trùng lặp (Duplicate)
- Tool sẽ cảnh báo nếu một định danh (struct, fn, biến...) xuất hiện nhiều lần trong cùng file.
- Báo cáo sẽ có trường `Duplicate` trong cột warning.
- Nếu muốn tắt rule này, cần sửa code (hoặc có thể tách thành flag enable_duplicate trong tương lai). 

### Rule kiểm tra độ dài định danh (Length)
- Tool sẽ cảnh báo nếu tên định danh quá ngắn hoặc quá dài (theo min_length, max_length trong naming.toml).
- Thêm vào naming.toml:
```toml
enable_length = true
min_length = 3
max_length = 20
```
- Nếu không khai báo, mặc định bật rule và không giới hạn độ dài.
- Báo cáo sẽ có trường `Length` trong cột warning nếu vi phạm. 