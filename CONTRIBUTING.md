# Contributing to coco 🥥

Thanks for your interest in contributing! coco is designed to be easy to extend — whether you're adding a new AI provider, a new commit format, or improving the core experience.

---

## Getting Started

```bash
git clone https://github.com/FabianTrafada/coco
cd coco
cargo build
```

Make sure you have Rust edition 2024 and Ollama running locally before you start.

---

## Project Structure

```
coco/
├── src/
│   ├── main.rs              ← entry point, orchestrates the flow
│   ├── lib.rs               ← exposes all modules
│   ├── cli/
│   │   └── mod.rs           ← CLI flags via clap
│   ├── config/
│   │   └── mod.rs           ← config loading & priority chain
│   ├── git/
│   │   └── mod.rs           ← diff parsing & git commit
│   ├── ui/
│   │   └── mod.rs           ← terminal output, colors, prompts
│   ├── providers/
│   │   ├── mod.rs           ← Provider trait + get_provider()
│   │   └── ollama.rs        ← Ollama implementation
│   └── formatters/
│       ├── mod.rs           ← Formatter trait + get_formatter()
│       ├── conventional.rs  ← Conventional Commits format
│       └── freeform.rs      ← Free-form format
├── Cargo.toml
├── README.md
└── CONTRIBUTING.md
```

---

## How to Add a New Provider

This is the most common contribution. The provider system is trait-based — you only need to touch two files.

### Step 1 — Create the provider file

Create `src/providers/<name>.rs`. Implement the `Provider` trait:

```rust
use anyhow::{Context, Result};
use async_trait::async_trait;

use super::Provider;

pub struct MyProvider {
    model: String,
    api_key: String,
}

impl MyProvider {
    pub fn new(model: &str, api_key: &str) -> Self {
        Self {
            model: model.to_string(),
            api_key: api_key.to_string(),
        }
    }
}

#[async_trait]
impl Provider for MyProvider {
    fn name(&self) -> &str {
        "myprovider"
    }

    async fn generate(&self, diff: &str, format: &str, language: &str) -> Result<String> {
        // call your provider's API here
        // return the commit message as a String
        todo!()
    }
}
```

### Step 2 — Register in `mod.rs`

Open `src/providers/mod.rs` and add your provider:

```rust
pub mod ollama;
pub mod myprovider;  // add this

pub fn get_provider(name: &str, model: &str, base_url: &str) -> Result<Box<dyn Provider>> {
    match name {
        "ollama" => Ok(Box::new(ollama::OllamaProvider::new(model, base_url))),
        "myprovider" => Ok(Box::new(myprovider::MyProvider::new(model, api_key))),  // add this
        _ => anyhow::bail!("Unknown provider: '{}'", name),
    }
}
```

That's it — no changes to core logic needed.

---

## How to Add a New Formatter

### Step 1 — Create the formatter file

Create `src/formatters/<name>.rs`:

```rust
use super::Formatter;

pub struct MyFormatter;

impl Formatter for MyFormatter {
    fn name(&self) -> &str {
        "myformat"
    }

    fn format(&self, message: &str) -> String {
        // clean up or transform the LLM output
        message.trim().to_string()
    }
}
```

### Step 2 — Register in `mod.rs`

Open `src/formatters/mod.rs` and add your formatter:

```rust
pub mod conventional;
pub mod freeform;
pub mod myformat;  // add this

pub fn get_formatter(name: &str) -> Box<dyn Formatter> {
    match name {
        "conventional" => Box::new(conventional::ConventionalFormatter),
        "myformat" => Box::new(myformat::MyFormatter),  // add this
        _ => Box::new(freeform::FreeformFormatter),
    }
}
```

---

## Guidelines

- **Keep PRs focused** — one feature or fix per PR
- **Follow existing code style** — run `cargo fmt` before committing
- **No warnings** — run `cargo clippy` and fix any warnings
- **Test your changes** — stage some files and run `cargo run` to verify the full flow works
- **Update README.md** — if you add a provider, add it to the supported providers table

---

## Running Locally

```bash
# build
cargo build

# run with staged changes
git add <some-file>
cargo run

# run with flags
cargo run -- -m qwen3.5 -y

# check for issues
cargo fmt
cargo clippy
```

---

## Opening a PR

1. Fork the repo
2. Create a branch: `git checkout -b feat/add-groq-provider`
3. Make your changes
4. Run `cargo fmt && cargo clippy`
5. Open a PR with a clear description of what you added and why

---

## Questions?

Open an issue and we'll help you out. 🥥
