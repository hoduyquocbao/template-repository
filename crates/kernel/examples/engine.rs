use kernel::{Engine, Plugin};
use std::sync::Arc;

struct Demo;

#[async_trait::async_trait]
impl Plugin for Demo {
    async fn init(&self, _config: &kernel::Config) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Demo] Plugin init");
        Ok(())
    }
    async fn shutdown(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[Demo] Plugin shutdown");
        Ok(())
    }
    fn name(&self) -> &str { "demo" }
    fn version(&self) -> &str { "1.0.0" }
    fn description(&self) -> &str { "Demo plugin" }
}

#[tokio::main]
async fn main() {
    let engine = Engine::new().unwrap();
    engine.add("demo".to_string(), Arc::new(Demo)).await.unwrap();
    engine.start().await.unwrap();
    println!("Engine state: {:?}", engine.state().await);
    engine.stop().await.unwrap();
    println!("Engine state: {:?}", engine.state().await);
} 