use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Deserialize, PartialEq)]
pub struct Config {
    #[serde(default)]
    pub provider: Vec<ProviderConfig>,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub struct ProviderConfig {
    #[serde(rename = "type")]
    pub provider_type: String,
    pub id: Option<String>,
    pub label: Option<String>,
    pub base_url: Option<String>,
    pub currency: Option<String>,
    pub api_key_env: Option<String>,
    pub management_key_env: Option<String>,
}

impl Config {
    pub fn load(path: &PathBuf) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|contents| toml::from_str(&contents).ok())
            .unwrap_or_else(Self::default)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            provider: vec![ProviderConfig {
                provider_type: "kimi".into(),
                id: None,
                label: None,
                base_url: None,
                currency: None,
                api_key_env: None,
                management_key_env: None,
            }],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_includes_kimi() {
        let config = Config::default();
        assert_eq!(config.provider.len(), 1);
        assert_eq!(config.provider[0].provider_type, "kimi");
    }

    #[test]
    fn parses_multiple_providers() {
        let contents = r#"
[[provider]]
type = "kimi"

[[provider]]
type = "deepseek"
id = "deepseek"
currency = "USD"
"#;
        let config: Config = toml::from_str(contents).unwrap();
        assert_eq!(config.provider.len(), 2);
        assert_eq!(config.provider[0].provider_type, "kimi");
        assert_eq!(config.provider[1].provider_type, "deepseek");
        assert_eq!(config.provider[1].id, Some("deepseek".into()));
        assert_eq!(config.provider[1].currency, Some("USD".into()));
    }

    #[test]
    fn missing_file_returns_default() {
        let config = Config::load(&PathBuf::from("/does/not/exist.toml"));
        assert_eq!(config.provider.len(), 1);
        assert_eq!(config.provider[0].provider_type, "kimi");
    }
}
