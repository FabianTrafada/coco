use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::{GenerationMetadata, GenerationOutput, Provider};

pub struct OllamaProvider {
    model: String,
    base_url: String,
    debug_prompt: bool,
}

impl OllamaProvider {
    pub fn new(model: &str, base_url: &str, debug_prompt: bool) -> Self {
        Self {
            model: model.to_string(),
            base_url: base_url.to_string(),
            debug_prompt,
        }
    }
}

#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    think: bool,
    options: OllamaOptions,
}

#[derive(Serialize)]
struct OllamaOptions {
    num_predict: i32,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    stop: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

#[async_trait]
impl Provider for OllamaProvider {
    fn name(&self) -> &str {
        "ollama"
    }

    async fn generate(&self, diff: &str, format: &str, language: &str) -> Result<GenerationOutput> {
        let prompt = build_prompt(diff, format, language);

        if self.debug_prompt {
            println!();
            println!(
                "=== coco debug: Ollama prompt (chars={}) ===",
                prompt.chars().count()
            );
            println!("{prompt}");
            println!("=== end coco debug prompt ===");
            println!();
        }

        let client = reqwest::Client::new();
        let url = format!("{}/api/generate", self.base_url);
        let options = OllamaOptions {
            num_predict: 32,
            temperature: 0.0,
            stop: None,
        };

        let response = client
            .post(&url)
            .json(&OllamaRequest {
                model: self.model.clone(),
                prompt,
                stream: false,
                think: false,
                options,
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

        let mut final_message = body.response.trim().to_string();
        let mut prompt_tokens = body.prompt_eval_count;
        let mut completion_tokens = body.eval_count;

        if !is_valid_commit_message(&final_message, format) {
            let repair_prompt = build_repair_prompt(&final_message, format, language);
            let repair_response = client
                .post(&url)
                .json(&OllamaRequest {
                    model: self.model.clone(),
                    prompt: repair_prompt,
                    stream: false,
                    think: false,
                    options: OllamaOptions {
                        num_predict: 32,
                        temperature: 0.1,
                        stop: None,
                    },
                })
                .send()
                .await
                .context("Failed to run Ollama repair pass")?;

            if !repair_response.status().is_success() {
                anyhow::bail!("Ollama repair pass error: {}", repair_response.status());
            }

            let repair_body: OllamaResponse = repair_response
                .json()
                .await
                .context("Failed to parse Ollama repair response")?;
            final_message = repair_body.response.trim().to_string();
            prompt_tokens = repair_body.prompt_eval_count;
            completion_tokens = repair_body.eval_count;
        }

        if !is_valid_commit_message(&final_message, format) {
            let fallback = fallback_commit_message(format, diff);
            final_message = fallback;
        }

        let total_tokens = match (prompt_tokens, completion_tokens) {
            (Some(prompt), Some(completion)) => Some(prompt + completion),
            _ => None,
        };

        Ok(GenerationOutput {
            message: final_message,
            metadata: GenerationMetadata {
                prompt_tokens,
                completion_tokens,
                total_tokens,
            },
        })
    }
}

fn is_valid_commit_message(message: &str, format: &str) -> bool {
    let trimmed = message.trim();
    if trimmed.is_empty() || trimmed.contains('\n') || trimmed.len() > 72 {
        return false;
    }
    if format == "conventional" {
        let has_type = [
            "feat", "fix", "chore", "refactor", "docs", "style", "test", "perf", "ci",
        ]
        .iter()
        .any(|t| trimmed.starts_with(&format!("{t}:")) || trimmed.starts_with(&format!("{t}(")));
        if !has_type {
            return false;
        }
    }
    true
}

fn build_repair_prompt(candidate: &str, format: &str, language: &str) -> String {
    let format_instruction = if format == "conventional" {
        "Output ONLY: <type>(<scope>): <description> or <type>: <description>."
    } else {
        "Output ONLY a short commit message."
    };
    format!(
        "Rewrite the text below into ONE valid git commit message.\n\
         {format_instruction}\n\
         Language: {language}\n\
         Rules: one line, max 72 chars, no explanation, no markdown.\n\
         Text to rewrite:\n\
         {candidate}"
    )
}

fn fallback_commit_message(format: &str, context: &str) -> String {
    let lc = context.to_lowercase();

    let conventional = if lc.contains("polar")
        || lc.contains("billing")
        || lc.contains("checkout")
        || lc.contains("subscription")
    {
        "feat(billing): add Polar billing integration".to_string()
    } else if lc.contains("router") || lc.contains("trpc") || lc.contains("api") {
        "feat(api): update routing and handlers".to_string()
    } else if lc.contains("readme") || lc.contains("docs") {
        "docs: update project documentation".to_string()
    } else if lc.contains("test") {
        "test: update and add test coverage".to_string()
    } else {
        "chore: update staged changes".to_string()
    };

    if format == "conventional" {
        conventional
    } else {
        conventional
            .split_once(": ")
            .map(|(_, rest)| rest.to_string())
            .unwrap_or_else(|| "update staged changes".to_string())
    }
}

fn build_prompt(diff: &str, format: &str, language: &str) -> String {
    let format_instruction = match format {
        "conventional" => {
            "Output ONLY a conventional commit message: <type>(<scope>): <description>\n\
             Types: feat, fix, chore, refactor, docs, style, test, perf, ci\n\
             Example: feat(billing): add Polar checkout and portal session"
        }
        _ => "Output ONLY a short commit message, nothing else.",
    };

    format!(
        "You are a git commit message generator. Your ONLY job is to output a single-line commit message.\n\
         {format_instruction}\n\
         Language: {language}\n\n\
         STRICT RULES:\n\
         - Output ONLY the commit message, one line, nothing else\n\
         - Do NOT review the code\n\
         - Do NOT explain what the code does\n\
         - Do NOT list changes or provide bullet points\n\
         - Do NOT add markdown formatting\n\
         - Do NOT add headers, suggestions, or additional text\n\
         - Do NOT wrap the message in quotes\n\
         - Maximum 72 characters\n\
         - Your entire response must be ONLY the commit message\n\n\
         Git context:\n\
         {diff}"
    )
}