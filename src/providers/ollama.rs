use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::Provider;

pub struct OllamaProvider {
    model: String,
    base_url: String,
}

impl OllamaProvider {
    pub fn new(model: &str, base_url: &str) -> Self {
        Self {
            model: model.to_string(),
            base_url: base_url.to_string(),
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn generate(&self, diff: &str, format: &str, language: &str) -> Result<String> {
        let prompt = build_prompt(diff, format, language);

        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", self.base_url);

        let response = client
            .post(&url)
            .json(&OllamaRequest {
                model: self.model.clone(),
                prompt,
                stream: false,
            })
            .send()
            .await
            .context("Failed to connect to Ollama. Is it running?")?;

        if !response.status().is_success() {
            anyhow::bail!("Ollama returned error: {}", response.status());
        }

        let body: OllamaResponse = response
            .json()
            .await
            .context("Failed to parse Ollama response")?;

        Ok(body.response.trim().to_string())
    }
}

fn build_prompt(diff: &str, format: &str, language: &str) -> String {
    let format_instruction = match format {
        "conventional" => {
            "Use conventional commits format: <type>(<scope>): <description>\n\
             Types: feat, fix, chore, refactor, docs, style, test, perf, ci\n\
             Example: feat(auth): add JWT refresh token support"
        }
        _ => "Write a clear, concise commit message describing what changed and why.",
    };

    format!(
        "You are an expert developer writing a git commit message.\n\
         {format_instruction}\n\
         Language: {language}\n\
         Rules:\n\
         - Be concise, max 72 characters for the first line\n\
         - Only output the commit message, nothing else\n\
         - No backticks, no explanations, no extra text\n\n\
         Git diff:\n\
         {diff}"
    )
}