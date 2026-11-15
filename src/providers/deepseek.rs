use std::env;

use serde::Deserialize;

use crate::Error;
use crate::config::ProviderConfig;
use crate::providers::{Balance, Currency, Provider};

const DEFAULT_BASE_URL: &str = "https://api.deepseek.com";
const DEFAULT_API_KEY_ENV: &str = "DEEPSEEK_API_KEY";

pub struct DeepSeekProvider {
    api_key_env: String,
    base_url: String,
    currency: Currency,
}

impl DeepSeekProvider {
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
            currency: config
                .currency
                .as_deref()
                .and_then(parse_currency)
                .unwrap_or(Currency::Usd),
        }
    }

    fn api_key(&self) -> Result<String, Error> {
        env::var(&self.api_key_env)
            .map_err(|_| Error::MissingApiKey(self.api_key_env.clone()))
    }
}

impl Provider for DeepSeekProvider {
    fn id(&self) -> &'static str {
        "deepseek"
    }

    fn label(&self) -> &'static str {
        "DeepSeek"
    }

    fn fetch_balance(&self) -> Result<Balance, Error> {
        let url = format!("{}/user/balance", self.base_url.trim_end_matches('/'));
        let client = reqwest::blocking::Client::new();
        let response: BalanceResponse = client
            .get(&url)
            .bearer_auth(self.api_key()?)
            .send()?
            .error_for_status()?
            .json()?;

        let info = response
            .balance_infos
            .iter()
            .find(|info| info.total().is_some_and(|total| total > 0.0))
            .or_else(|| response.balance_infos.iter().find(|info| info.total().is_some()))
            .ok_or_else(|| Error::InvalidResponse("DeepSeek returned no balance".into()))?;

        let currency = match info.currency.as_deref() {
            Some("USD") => Currency::Usd,
            Some("CNY") | None => Currency::Cny,
            Some(_) => self.currency,
        };

        Ok(Balance {
            available: info.total().unwrap_or(0.0),
            currency,
            voucher: None,
        })
    }
}

#[derive(Debug, Deserialize)]
struct BalanceResponse {
    #[serde(default)]
    balance_infos: Vec<BalanceInfo>,
}

#[derive(Debug, Deserialize)]
struct BalanceInfo {
    currency: Option<String>,
    #[serde(default)]
    total_balance: Option<String>,
}

impl BalanceInfo {
    fn total(&self) -> Option<f64> {
        self.total_balance
            .as_deref()?
            .trim()
            .parse()
            .ok()
            .filter(|total: &f64| total.is_finite())
    }
}

fn parse_currency(s: &str) -> Option<Currency> {
    match s.to_ascii_uppercase().as_str() {
        "USD" => Some(Currency::Usd),
        "CNY" => Some(Currency::Cny),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ProviderConfig;

    fn default_config() -> ProviderConfig {
        ProviderConfig {
            provider_type: "deepseek".into(),
            id: None,
            label: None,
            base_url: None,
            currency: None,
            api_key_env: None,
            management_key_env: None,
        }
    }

    #[test]
    fn provider_identity() {
        let p = DeepSeekProvider::from_config(&default_config());
        assert_eq!(p.id(), "deepseek");
        assert_eq!(p.label(), "DeepSeek");
    }

    #[test]
    fn parses_balance_info_total() {
        let info = BalanceInfo {
            currency: Some("USD".into()),
            total_balance: Some("12.50".into()),
        };
        assert_eq!(info.total(), Some(12.5));
        assert_eq!(info.currency.as_deref(), Some("USD"));
    }
}
