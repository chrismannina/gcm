use anyhow::{Context, Result};
use dirs::home_dir;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default = "default_max_tokens")]
    pub max_tokens: u16,

    #[serde(default = "default_context")]
    pub context: Vec<String>,

    #[serde(default)]
    pub api_key: String,

    #[serde(default)]
    pub provider: String,
}

fn default_model() -> String {
    "gpt-4o-mini".to_string()
}

fn default_max_tokens() -> u16 {
    100
}

fn default_context() -> Vec<String> {
    vec!["README.md".to_string()]
}

impl Default for Config {
    fn default() -> Self {
        Config {
            model: default_model(),
            max_tokens: default_max_tokens(),
            context: default_context(),
            api_key: String::new(),
            provider: "openai".to_string(),
        }
    }
}

impl Config {
    pub fn load() -> Result<Self> {
        let mut config = Config::default();

        dotenv::dotenv().ok();

        if let Some(home) = home_dir() {
            let user_config = home.join(".gcmrc");
            if user_config.exists() {
                config.merge_from_file(&user_config)?;
            }
        }

        let project_config = Path::new(".gcm.yml");
        if project_config.exists() {
            config.merge_from_file(project_config)?;
        }

        if let Ok(api_key) = env::var("OPENAI_API_KEY") {
            config.api_key = api_key;
            config.provider = "openai".to_string();
        } else if let Ok(api_key) = env::var("ANTHROPIC_API_KEY") {
            config.api_key = api_key;
            config.provider = "anthropic".to_string();
        }

        Ok(config)
    }

    fn merge_from_file(&mut self, path: &Path) -> Result<()> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file: {:?}", path))?;

        let file_config: Config = serde_yaml::from_str(&content)
            .with_context(|| format!("Failed to parse config file: {:?}", path))?;

        if !file_config.model.is_empty() {
            self.model = file_config.model;
        }
        if file_config.max_tokens > 0 {
            self.max_tokens = file_config.max_tokens;
        }
        if !file_config.context.is_empty() {
            self.context = file_config.context;
        }
        if !file_config.api_key.is_empty() {
            self.api_key = file_config.api_key;
        }
        if !file_config.provider.is_empty() {
            self.provider = file_config.provider;
        }

        Ok(())
    }
}