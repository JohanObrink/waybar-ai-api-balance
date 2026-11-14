use std::env;

use serde::Deserialize;

use crate::Error;
use crate::providers::{Balance, Currency, Provider};

const DEFAULT_BASE_URL: &str = "https://api.moonshot.ai/v1";
const API_KEY_ENV: &str = "MOONSHOT_API_KEY";
const BASE_URL_ENV: &str = "MOONSHOT_BASE_URL";
const CURRENCY_ENV: &str = "MOONSHOT_CURRENCY";

pub struct KimiProvider;

impl KimiProvider {
    pub fn new() -> Self {
        Self
    }

    fn base_url() -> String {
        env::var(BASE_URL_ENV).unwrap_or_else(|_| DEFAULT_BASE_URL.to_string())
    }

    fn api_key() -> Result<String, Error> {
        env::var(API_KEY_ENV).map_err(|_| Error::MissingApiKey(API_KEY_ENV))
    }

    fn currency() -> Currency {
        env::var(CURRENCY_ENV)
            .ok()
            .as_deref()
            .and_then(parse_currency)
            .unwrap_or_else(|| infer_currency(&Self::base_url()))
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
        let url = format!("{}/users/me/balance", Self::base_url().trim_end_matches('/'));
        let client = reqwest::blocking::Client::new();
        let response: BalanceResponse = client
            .get(&url)
            .bearer_auth(Self::api_key()?)
            .send()?
            .error_for_status()?
            .json()?;
        Ok(Balance {
            available: response.data.available_balance,
            currency: Self::currency(),
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
        let p = KimiProvider::new();
        assert_eq!(p.id(), "kimi");
        assert_eq!(p.label(), "Kimi");
    }
}
