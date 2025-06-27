# Kernel

## Triết Lý

- Mọi định danh (biến, hàm, struct, trait, module) là **một từ đơn tiếng Anh** – không viết tắt, không ghép từ.
- Kiến trúc module rõ ràng, tách biệt, interface tự nhiên, dễ mở rộng.
- Đơn giản hóa tối đa, trừu tượng hóa thanh lịch, không dư thừa.

## Kiến Trúc

- `Engine`: Điều phối lifecycle, plugin, config, logger, router.
- `Plugin`: Giao diện mở rộng động, quản lý lifecycle, metadata.
- `Logger`: Logging đa cấp độ, hỗ trợ context, performance.
- `Router`: Định tuyến request/command đến handler.
- `Builder`: Khởi tạo engine linh hoạt với config, plugin.
- `Validator`: Xác thực dữ liệu (text, number) theo rule.
- `Serializer`: Tuần tự hóa dữ liệu (JSON, Bincode).
- `Config`: Quản lý cấu hình hệ thống.

## Hướng Dẫn Sử Dụng

### Khởi tạo Engine với Plugin

```rust
use kernel::{Engine, Plugin};
use std::sync::Arc;
use async_trait::async_trait;

struct Hello;

#[async_trait]
impl Plugin for Hello {
    async fn init(&self, _config: &kernel::Config) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> { Ok(()) }
    fn name(&self) -> &str { "hello" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Say hello" }
}

#[tokio::main]
async fn main() {
    let engine = Engine::new().unwrap();
    engine.add("hello".to_string(), Arc::new(Hello)).await.unwrap();
    engine.start().await.unwrap();
    // ...
    engine.stop().await.unwrap();
}
```

### Logging

```rust
use kernel::{Logger, Config};
let logger = Logger::new(&Config::new()).unwrap();
logger.info("Info message");
logger.warn("Warning");
logger.error("Error");
logger.debug("Debug");
logger.trace("Trace");
logger.context("EXAMPLE", "Context message");
```

### Router

```rust
use kernel::{Router, router::{Handler, Request, Response}};
use std::sync::Arc;
use async_trait::async_trait;

struct Echo;
#[async_trait]
impl Handler for Echo {
    async fn handle(&self, req: Request) -> Result<Response, Box<dyn std::error::Error>> {
        Ok(Response { status: 200, headers: Default::default(), body: req.body })
    }
}

#[tokio::main]
async fn main() {
    let router = Router::new();
    router.register("/echo".to_string(), Arc::new(Echo)).await;
    let req = Request { path: "/echo".to_string(), method: "POST".to_string(), headers: Default::default(), body: b"hi".to_vec() };
    let res = router.route(req).await.unwrap();
    println!("Response: {} {:?}", res.status, String::from_utf8_lossy(&res.body));
}
```

### Validator

```rust
use kernel::validator::{System, Text, Number};
let validator = System::new();
let result = validator.text("hello", &[Text::Min(3), Text::Max(10)]);
assert!(result.is_ok());
let result = validator.number(&5.0, &[Number::Min(1.0), Number::Max(10.0)]);
assert!(result.is_ok());
```

### Serializer

```rust
use kernel::serializer::System;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, PartialEq, Debug)]
struct Data { name: String, value: i32 }
let system = System::new();
let data = Data { name: "test".to_string(), value: 42 };
let json = system.json(&data).unwrap();
let parsed: Data = system.parse(&json).unwrap();
assert_eq!(data, parsed);
```

## Best Practice

- Luôn đặt tên một từ đơn, không viết tắt, không ghép từ.
- Tách module rõ ràng, interface tự nhiên, không phụ thuộc vòng.
- Đọc kỹ ví dụ end-to-end trong thư mục `examples/` để hiểu cách sử dụng thực tế.
- Đọc doc comment từng module để nắm rõ API và pattern khuyến nghị.

## Đóng Góp

- Đóng góp code, issue, tài liệu đều phải tuân thủ triết lý một từ đơn tiếng Anh.
- Ưu tiên refactor, tinh chỉnh interface để đạt sự rõ ràng, thanh lịch tối đa. 