use kernel::{Plugin, Engine, Config};
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
    fn name(&self) -> &str {
        "hello"
    }
    fn version(&self) -> &str {
        "1.0.0"
    }
    fn description(&self) -> &str {
        "Say hello"
    }
}

#[tokio::main]
async fn main() {
    let engine = Engine::new().unwrap();
    let plugin = Arc::new(Hello);
    engine.add("hello".to_string(), plugin.clone()).await.unwrap();
    engine.start().await.unwrap();
    println!("Plugin: {} v{} - {}", plugin.name(), plugin.version(), plugin.description());
    engine.stop().await.unwrap();
} 