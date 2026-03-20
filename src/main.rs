use anyhow::Result;
use clap::Parser;
use owo_colors::OwoColorize;
use std::time::Instant;

use coco::cli::Cli;
use coco::config::Config;
use coco::ui::Action;
use coco::{formatters, git, providers, ui};

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();
    check_for_update().await;

    let mut config = Config::load().unwrap_or_else(|_| Config::default());
    config.apply_overrides(args.provider, args.model);

    let provider_config = config.provider.clone().unwrap_or_default();
    let core_config = config.core.clone().unwrap_or_default();

    let provider_name = provider_config.name.as_deref().unwrap_or("ollama");
    let model = provider_config.model.as_deref().unwrap_or("llama3.2");
    let base_url = provider_config
        .base_url
        .as_deref()
        .unwrap_or("http://localhost:11434");
    let format = core_config.format.as_deref().unwrap_or("conventional");
    let language = core_config.language.as_deref().unwrap_or("english");

    ui::print_analyzing();

    let diff = match git::get_staged_diff() {
        Ok(d) => d,
        Err(e) => {
            ui::print_error(&format!("{}", e));
            std::process::exit(1);
        }
    };

    if diff.is_empty() {
        ui::print_no_staged_changes();
        std::process::exit(0);
    }

    ui::print_staged_diff(&diff);

    let provider = match providers::get_provider(provider_name, model, base_url, args.debug) {
        Ok(p) => p,
        Err(e) => {
            ui::print_error(&format!("{}", e));
            std::process::exit(1);
        }
    };

    let formatter = formatters::get_formatter(format);

    let (llm_context, condensed) = diff.context_for_llm();
    if condensed {
        println!(
            "  {} Large staged diff: using OpenCode-style git context (stat/name-status/status + capped -U0 excerpt). Set {} to send the full patch.",
            "✦".bright_purple(),
            "COCO_LLM_FULL_DIFF=1".bright_white()
        );
        println!();
    }

    let first_start = Instant::now();
    let mut generation = match provider.generate(&llm_context, format, language).await {
        Ok(g) => g,
        Err(e) => {
            ui::print_error(&format!("{}", e));
            std::process::exit(1);
        }
    };
    let mut last_elapsed_ms = first_start.elapsed().as_millis();
    let mut message = formatter.format(&generation.message);

    if args.always_trust {
        ui::print_suggested_message(&message);
        ui::print_generation_stats(
            last_elapsed_ms,
            generation.metadata.prompt_tokens,
            generation.metadata.completion_tokens,
            generation.metadata.total_tokens,
        );
        commit_and_exit(&message);
        return Ok(());
    }

    loop {
        ui::print_suggested_message(&message);
        ui::print_generation_stats(
            last_elapsed_ms,
            generation.metadata.prompt_tokens,
            generation.metadata.completion_tokens,
            generation.metadata.total_tokens,
        );

        match ui::prompt_action()? {
            Action::Commit => {
                commit_and_exit(&message);
                break;
            }
            Action::Edit => {
                message = ui::prompt_edit(&message)?;
            }
            Action::Regenerate => {
                println!();
                ui::print_analyzing();
                let regenerate_start = Instant::now();
                let (ctx, _) = diff.context_for_llm();
                generation = match provider.generate(&ctx, format, language).await {
                    Ok(g) => g,
                    Err(e) => {
                        ui::print_error(&format!("{}", e));
                        std::process::exit(1);
                    }
                };
                last_elapsed_ms = regenerate_start.elapsed().as_millis();
                message = formatter.format(&generation.message);
            }
            Action::Abort => {
                ui::print_aborted();
                break;
            }
            Action::Unknown => {
                ui::print_error("Invalid input. Press C, E, R, or A.");
            }
        }
    }

    Ok(())
}

async fn check_for_update() {
    let current = env!("CARGO_PKG_VERSION");

    let Ok(response) = reqwest::Client::new()
        .get("https://api.github.com/repos/FabianTrafada/coco/releases/latest")
        .header("User-Agent", "coco")
        .send()
        .await
    else {
        return;
    };

    let Ok(json) = response.json::<serde_json::Value>().await else {
        return;
    };

    let Some(latest) = json["tag_name"].as_str() else {
        return;
    };

    // strip "v" prefix — "v0.1.1" → "0.1.1"
    let latest_clean = latest.trim_start_matches('v');

    if latest_clean != current {
        ui::print_update_available(current, latest_clean);
    }
}

fn commit_and_exit(message: &str) {
    match git::commit(message) {
        Ok(_) => ui::print_committed(),
        Err(e) => {
            ui::print_error(&format!("{}", e));
            std::process::exit(1);
        }
    }
}