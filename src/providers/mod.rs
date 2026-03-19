use anyhow::Result;
use async_trait::async_trait;

pub mod ollama;

#[async_trait]
pub trait Provider {
    fn name(&self) -> &str;
    async fn generate(&self, diff: &str, format: &str, language: &str) -> Result<String>;
}

pub fn get_provider(name: &str, model: &str, base_url: &str) -> Result<Box<dyn Provider>> {
    match name {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(model, base_url))),
        _ => anyhow::bail!("Unknown provider: '{}'. Available: ollama", name),
    }
}