use anyhow::Result;
use async_trait::async_trait;

pub mod ollama;

#[derive(Debug, Clone, Default)]
pub struct GenerationMetadata {
    pub prompt_tokens: Option<u32>,
    pub completion_tokens: Option<u32>,
    pub total_tokens: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct GenerationOutput {
    pub message: String,
    pub metadata: GenerationMetadata,
}

#[async_trait]
pub trait Provider {
    fn name(&self) -> &str;
    async fn generate(&self, diff: &str, format: &str, language: &str) -> Result<GenerationOutput>;
}

pub fn get_provider(
    name: &str,
    model: &str,
    base_url: &str,
    debug_prompt: bool,
) -> Result<Box<dyn Provider>> {
    match name {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(
            model,
            base_url,
            debug_prompt,
        ))),
        _ => anyhow::bail!("Unknown provider: '{}'. Available: ollama", name),
    }
}