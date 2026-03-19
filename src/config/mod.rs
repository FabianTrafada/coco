use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CoreConfig {
    pub format: Option<String>,
    pub language: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ProviderConfig {
    pub name: Option<String>,
    pub model: Option<String>,
    pub base_url: Option<String>,
    pub api_key: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    pub core: Option<CoreConfig>,
    pub provider: Option<ProviderConfig>,
}

impl Default for CoreConfig {
    fn default() -> Self {
        CoreConfig {
            format: Some("conventional".to_string()),
            language: Some("english".to_string()),
        }
    }
}

impl Default for ProviderConfig {
    fn default() -> Self {
        ProviderConfig {
            name: Some("ollama".to_string()),
            model: Some("llama3.2".to_string()),
            base_url: Some("http://localhost:11434".to_string()),
            api_key: None,
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config {
            core: Some(CoreConfig::default()),
            provider: Some(ProviderConfig::default()),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let path = config_path();
    
        if !path.exists() {
            let default = Self::default();
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let toml_str = toml::to_string_pretty(&default)?;
            fs::write(&path, toml_str)?;
            return Ok(default);
        }
    
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn apply_overrides(&mut self, provider: Option<String>, model: Option<String>) {
        if let Some(p) = provider {
            if let Some(ref mut prov) = self.provider {
                prov.name = Some(p);
            }
        }
        if let Some(m) = model {
            if let Some(ref mut prov) = self.provider {
                prov.model = Some(m);
            }
        }
    }
}

fn config_path() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("coco");
    path.push("config.toml");
    path
}