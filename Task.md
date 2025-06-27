
**Nhiệm vụ 1: Tích hợp Metric vào Actor Model**

Logic `with_metric` hiện tại trong `repository/src/sled.rs` không thể hoạt động vì các thao tác đã được chuyển vào `actor.rs`. Bạn cần di chuyển logic đo lường vào đúng nơi nó cần thực thi: bên trong message loop của actor.

1.  **Truyền `Registry` Metric vào `Actor`:** Đảm bảo `Actor` sở hữu hoặc có tham chiếu đến `Registry` metric. Bạn có thể truyền nó vào khi `Actor::new` được gọi.

2.  **Đo lường trong Message Loop:** Trong file `crates/repository/src/actor.rs`, tại vòng lặp `while let Some(msg) = rx.blocking_recv()`, hãy bọc các lệnh gọi đến `inner.db` để ghi lại metric.

      * Bạn cần `use std::time::Instant;` và `use crate::metric::Registry;`.

      * **Ví dụ sửa đổi cho `Message::Insert`:**

        ```rust
        // TRONG crates/repository/src/actor.rs
        // ... bên trong thread::spawn của Actor::new

        // GIẢ SỬ `metric_registry` đã được truyền vào

        while let Some(msg) = rx.blocking_recv() {
            match msg {
                Message::Insert { key, value, respond } => {
                    let start = Instant::now(); // Bắt đầu đo
                    let res = inner.db.insert(&key, &value).map(|_| ()).map_err(Error::Store);
                    
                    // Ghi lại metric với tên "insert" và kết quả của thao tác
                    metric_registry.record("insert", res.is_err()); 
                    
                    let _ = respond.send(res);
                }
                Message::Fetch { key, respond } => {
                    let start = Instant::now();
                    let res = inner.db.get(&key).map(|opt| opt.map(|v| v.to_vec())).map_err(Error::Store);
                    
                    // Ghi lại metric với tên "fetch"
                    metric_registry.record("fetch", res.is_err());

                    let _ = respond.send(res);
                }
                // ... ÁP DỤNG TƯƠNG TỰ CHO CÁC MESSAGE CÒN LẠI ...
                // (Update, Delete, Query, Mass, Keys) với các tên metric tương ứng.
            }
        }
        ```

      * **Lưu ý:** Vì `metric.record` là hàm `async`, bạn cần điều chỉnh `Actor` để có thể gọi nó. Một cách đơn giản là làm cho `Registry` có các phương thức `record` đồng bộ sử dụng `Arc<AtomicU64>` như trong file `crates/repository/src/metric.rs`. Rất may, thiết kế hiện tại của `Metric` đã sử dụng `AtomicU64` nên nó an toàn cho thread. Bạn chỉ cần đảm bảo `Actor` có thể gọi nó.

-----
