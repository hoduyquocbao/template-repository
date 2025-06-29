use kernel::builder::Builder;
use kernel::plugin::Plugin;
use kernel::config::Config;
use std::sync::Arc;
use async_trait::async_trait;

struct Hello;

#[async_trait]
impl Plugin for Hello {
    async fn init(&self, _config: &Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Hello] init");
        Ok(())
    }
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Hello] shutdown");
        Ok(())
    }
    fn name(&self) -> &str { "hello" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Say hello" }
}

#[tokio::main]
async fn main() {
    let config = Config::new();
    let engine = Builder::new()
        .config(config)
        .plugin("hello".to_string(), Arc::new(Hello))
        .build()
        .await
        .unwrap();
    engine.start().await.unwrap();
    println!("Engine started with plugin: hello");
    engine.stop().await.unwrap();
} 