use std::env;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("missing API key: set MOONSHOT_API_KEY")]
    MissingApiKey,
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Currency {
    Usd,
    Cny,
}

#[derive(Debug, Clone)]
struct Balance {
    available: f64,
    cash: f64,
    voucher: f64,
    currency: Currency,
}

#[derive(Debug, Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

fn main() -> Result<(), Error> {
    let api_key = env::var("MOONSHOT_API_KEY").map_err(|_| Error::MissingApiKey)?;
    let base_url = env::var("MOONSHOT_BASE_URL")
        .unwrap_or_else(|_| "https://api.moonshot.ai/v1".to_string());
    let currency = env::var("MOONSHOT_CURRENCY")
        .ok()
        .as_deref()
        .and_then(parse_currency)
        .unwrap_or_else(|| infer_currency(&base_url));

    let balance = fetch_balance(&base_url, &api_key, currency)?;
    let output = format_waybar(balance, "Moonshot API");
    println!("{}", serde_json::to_string(&output).unwrap());
    Ok(())
}

fn fetch_balance(base_url: &str, api_key: &str, currency: Currency) -> Result<Balance, Error> {
    let url = format!("{}/users/me/balance", base_url.trim_end_matches('/'));
    let client = reqwest::blocking::Client::new();
    let response: MoonshotBalanceResponse = client
        .get(&url)
        .bearer_auth(api_key)
        .send()?
        .error_for_status()?
        .json()?;
    response.to_balance(currency)
}

#[derive(Debug, Deserialize)]
struct MoonshotBalanceResponse {
    data: MoonshotBalanceData,
}

#[derive(Debug, Deserialize)]
struct MoonshotBalanceData {
    available_balance: f64,
    cash_balance: f64,
    voucher_balance: f64,
}

impl MoonshotBalanceResponse {
    fn to_balance(self, currency: Currency) -> Result<Balance, Error> {
        Ok(Balance {
            available: self.data.available_balance,
            cash: self.data.cash_balance,
            voucher: self.data.voucher_balance,
            currency,
        })
    }
}

fn format_waybar(balance: Balance, provider_name: &str) -> WaybarOutput {
    WaybarOutput {
        text: format!(
            "󰚩 {}",
            format_amount(balance.available, balance.currency)
        ),
        tooltip: format!(
            "{}\nCash: {}\nVoucher: {}",
            provider_name,
            format_amount(balance.cash, balance.currency),
            format_amount(balance.voucher, balance.currency)
        ),
        class: "moonshot-balance".to_string(),
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

fn format_amount(amount: f64, currency: Currency) -> String {
    let symbol = match currency {
        Currency::Usd => '$',
        Currency::Cny => '¥',
    };
    format!("{symbol}{amount:.2}")
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
    fn formats_amounts_with_currency_symbol() {
        assert_eq!(format_amount(42.14854, Currency::Usd), "$42.15");
        assert_eq!(format_amount(100.0, Currency::Cny), "¥100.00");
    }

    #[test]
    fn converts_response_to_balance() {
        let response = MoonshotBalanceResponse {
            data: MoonshotBalanceData {
                available_balance: 42.14854,
                cash_balance: 42.14854,
                voucher_balance: 0.0,
            },
        };
        let balance = response.to_balance(Currency::Usd).unwrap();
        assert!((balance.available - 42.14854).abs() < f64::EPSILON);
        assert!((balance.cash - 42.14854).abs() < f64::EPSILON);
        assert!((balance.voucher).abs() < f64::EPSILON);
        assert_eq!(balance.currency, Currency::Usd);
    }

    #[test]
    fn formats_waybar_output() {
        let balance = Balance {
            available: 42.14854,
            cash: 42.14854,
            voucher: 0.0,
            currency: Currency::Usd,
        };
        let output = format_waybar(balance, "Moonshot API");
        assert_eq!(output.text, "󰚩 $42.15");
        assert_eq!(
            output.tooltip,
            "Moonshot API\nCash: $42.15\nVoucher: $0.00"
        );
        assert_eq!(output.class, "moonshot-balance");
    }
}
