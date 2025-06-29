use kernel::router::Router;
use kernel::{router::{Handler, Request, Response}};
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
    let handler = Arc::new(Echo);
    router.register("/echo".to_string(), handler).await;
    let req = Request { path: "/echo".to_string(), method: "POST".to_string(), headers: Default::default(), body: b"hi".to_vec() };
    let res = router.route(req).await.unwrap();
    println!("Response: {} {:?}", res.status, String::from_utf8_lossy(&res.body));
    let req = Request { path: "/none".to_string(), method: "GET".to_string(), headers: Default::default(), body: vec![] };
    let res = router.route(req).await.unwrap();
    println!("Response: {} {:?}", res.status, String::from_utf8_lossy(&res.body));
} 