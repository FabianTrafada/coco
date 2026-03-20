# coco 🥥

> Generate commit messages from staged changes using local AI — powered by Ollama.

![coco preview](.github/assets/coco-preview.webp)

---

## Features

- **Local-first** — runs entirely on your machine via Ollama, no data leaves your system
- **Conventional Commits** — supports `feat:`, `fix:`, `chore:`, `refactor:`, and more
- **Interactive** — review, edit, or regenerate before committing
- **Configurable** — set your preferred provider, model, and format via `config.toml`
- **Extensible** — provider and formatter system designed for easy contribution

---

## Installation

### Prerequisites

- [Ollama](https://ollama.com/) running locally

### Linux & macOS

```sh
curl -fsSL https://raw.githubusercontent.com/FabianTrafada/coco/main/install.sh | sh
```

### Windows

```powershell
irm https://raw.githubusercontent.com/FabianTrafada/coco/main/install.ps1 | iex
```

### From source

```bash
git clone https://github.com/FabianTrafada/coco
cd coco
cargo install --path .
```

### Uninstall

```sh
# Linux & macOS
sudo rm /usr/local/bin/coco

# Windows
Remove-Item "$env:USERPROFILE\.coco\bin\coco.exe"
```

---

## Usage

```bash
# Stage your changes first
git add .

# Run coco
coco
```

### Flags

| Flag | Short | Description |
|---|---|---|
| `--always-trust` | `-y` | Skip confirmation and commit immediately |
| `--provider` | `-p` | Override provider (e.g. `ollama`, `openai`) |
| `--model` | `-m` | Override model (e.g. `qwen3.5`, `llama3.2`) |
| `--help` | `-h` | Print help |
| `--version` | `-V` | Print version |

### Examples

```bash
coco                          # use default config
coco -y                       # auto-commit, no confirmation
coco -m qwen3.5               # use specific model
coco -p openai -m gpt-4o      # use different provider
coco -p ollama -m qwen3.5 -y  # combine flags
```

### Large diffs & performance

If your staged diff is very large, sending the full patch to Ollama can take a long time (most of the wait is **prompt evaluation** on your machine). By default, **coco** sends a **file-level summary** to the model when the staged diff exceeds ~50k characters; the full diff is still shown in the terminal summary above.

To force sending the entire unified diff to the model:

```bash
COCO_LLM_FULL_DIFF=1 coco
```

---

## Configuration

Config file is located at `~/.config/coco/config.toml`. Created automatically on first run with defaults.

```toml
[core]
format = "conventional"   # or "freeform"
language = "english"

[provider]
name = "ollama"
model = "qwen3.5"

[provider.ollama]
base_url = "http://localhost:11434"

[provider.openai]
api_key = "sk-..."
base_url = "https://api.openai.com/v1"
```

### Priority chain

```
CLI flags  →  config.toml  →  defaults
```

---

## Supported Providers

| Provider | Status |
|---|---|
| Ollama (local) | ✅ Available |
| OpenAI | 🔜 Soon |
| Anthropic | 🔜 Soon |
| Groq | 🔜 Soon |

---

## Supported Models (via Ollama)

Any model available in Ollama works with coco. Recommended:

```bash
ollama pull qwen3.5        # recommended, best quality
ollama pull qwen3.5:0.8b   # lightweight, faster
ollama pull llama3.2       # alternative
```

---

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for how to add new providers, formatters, and more.

---

## License

MIT