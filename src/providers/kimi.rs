use std::env;

use serde::Deserialize;

use crate::Error;
use crate::config::ProviderConfig;
use crate::providers::{Balance, Currency, Provider};

const DEFAULT_BASE_URL: &str = "https://api.moonshot.ai/v1";
const DEFAULT_API_KEY_ENV: &str = "MOONSHOT_API_KEY";

pub struct KimiProvider {
    api_key_env: String,
    base_url: String,
    currency_override: Option<Currency>,
}

impl KimiProvider {
    pub fn from_config(config: &ProviderConfig) -> Self {
        Self {
            api_key_env: config
                .api_key_env
                .clone()
                .unwrap_or_else(|| DEFAULT_API_KEY_ENV.into()),
            base_url: config
                .base_url
                .clone()
                .unwrap_or_else(|| DEFAULT_BASE_URL.into()),
            currency_override: config.currency.as_deref().and_then(parse_currency),
        }
    }

    fn api_key(&self) -> Result<String, Error> {
        env::var(&self.api_key_env)
            .map_err(|_| Error::MissingApiKey(self.api_key_env.clone()))
    }

    fn currency(&self) -> Currency {
        self.currency_override
            .unwrap_or_else(|| infer_currency(&self.base_url))
    }
}

impl Provider for KimiProvider {
    fn id(&self) -> &'static str {
        "kimi"
    }

    fn label(&self) -> &'static str {
        "Kimi"
    }

    fn fetch_balance(&self) -> Result<Balance, Error> {
        let url = format!("{}/users/me/balance", self.base_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::new();
        let response: BalanceResponse = client
            .get(&url)
            .bearer_auth(self.api_key()?)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(Balance {
            available: response.data.available_balance,
            currency: self.currency(),
        })
    }
}

fn infer_currency(base_url: &str) -> Currency {
    if base_url.contains("moonshot.cn") {
        Currency::Cny
    } else {
        Currency::Usd
    }
}

fn parse_currency(s: &str) -> Option<Currency> {
    match s.to_ascii_uppercase().as_str() {
        "USD" => Some(Currency::Usd),
        "CNY" => Some(Currency::Cny),
        _ => None,
    }
}

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    data: BalanceData,
}

#[derive(Debug, Deserialize)]
struct BalanceData {
    available_balance: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;

    fn default_config() -> ProviderConfig {
        ProviderConfig {
            provider_type: "kimi".into(),
            id: None,
            label: None,
            base_url: None,
            currency: None,
            api_key_env: None,
            management_key_env: None,
        }
    }

    #[test]
    fn infers_usd_for_international_host() {
        assert_eq!(infer_currency("https://api.moonshot.ai/v1"), Currency::Usd);
    }

    #[test]
    fn infers_cny_for_china_host() {
        assert_eq!(infer_currency("https://api.moonshot.cn/v1"), Currency::Cny);
    }

    #[test]
    fn parses_currency_override() {
        assert_eq!(parse_currency("USD"), Some(Currency::Usd));
        assert_eq!(parse_currency("cny"), Some(Currency::Cny));
        assert_eq!(parse_currency("EUR"), None);
    }

    #[test]
    fn provider_identity() {
        let p = KimiProvider::from_config(&default_config());
        assert_eq!(p.id(), "kimi");
        assert_eq!(p.label(), "Kimi");
    }

    #[test]
    fn currency_override_from_config() {
        let mut config = default_config();
        config.currency = Some("CNY".into());
        let p = KimiProvider::from_config(&config);
        assert_eq!(p.currency(), Currency::Cny);
    }
}
