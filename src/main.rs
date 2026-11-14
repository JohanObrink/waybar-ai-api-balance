mod state;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("missing API key: set MOONSHOT_API_KEY")]
    MissingApiKey,
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("notification failed")]
    Notification,
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

impl Balance {
    fn available_text(&self) -> String {
        format_amount(self.available, self.currency)
    }

    fn details(&self, provider_name: &str, consumption: &state::Consumption) -> String {
        format!(
            "{}\nAvailable: {}\nToday's consumption: {}\nTotal consumption: {}",
            provider_name,
            self.available_text(),
            format_amount(consumption.today, self.currency),
            format_amount(consumption.total, self.currency)
        )
    }
}

#[derive(Debug, Serialize)]
struct WaybarOutput {
    text: String,
    tooltip: String,
    class: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Waybar,
    Notify,
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
    let mode = parse_mode();

    let balance = fetch_balance(&base_url, &api_key, currency)?;
    let provider_name = "Moonshot API";
    let state_path = state_path();
    let mut state = state::State::load(&state_path);
    let consumption = state.update(state::BalanceSnapshot {
        timestamp: Utc::now(),
        available: balance.available,
    });
    state.save(&state_path);

    match mode {
        Mode::Waybar => print_waybar(balance, &consumption, provider_name),
        Mode::Notify => send_notification(balance, &consumption, provider_name)?,
    }

    Ok(())
}

fn state_path() -> PathBuf {
    dirs::state_dir()
        .map(|p| p.join("waybar-ai-api-balance/state.json"))
        .unwrap_or_else(|| PathBuf::from("/tmp/waybar-ai-api-balance-state.json"))
}

fn parse_mode() -> Mode {
    if env::args().any(|arg| arg == "--notify") {
        Mode::Notify
    } else {
        Mode::Waybar
    }
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

fn format_waybar(
    balance: Balance,
    consumption: &state::Consumption,
    provider_name: &str,
) -> WaybarOutput {
    WaybarOutput {
        text: balance.available_text(),
        tooltip: balance.details(provider_name, consumption),
        class: "moonshot-balance".to_string(),
    }
}

fn print_waybar(balance: Balance, consumption: &state::Consumption, provider_name: &str) {
    println!(
        "{}",
        serde_json::to_string(&format_waybar(balance, consumption, provider_name)).unwrap()
    );
}

fn send_notification(
    balance: Balance,
    consumption: &state::Consumption,
    provider_name: &str,
) -> Result<(), Error> {
    let status = Command::new("notify-send")
        .arg(provider_name)
        .arg(balance.details(provider_name, consumption))
        .status()
        .map_err(|_| Error::Notification)?;
    if status.success() {
        Ok(())
    } else {
        Err(Error::Notification)
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
        let consumption = state::Consumption {
            today: 1.0,
            total: 5.0,
        };
        let output = format_waybar(balance, &consumption, "Moonshot API");
        assert_eq!(output.text, "$42.15");
        assert_eq!(
            output.tooltip,
            "Moonshot API\nAvailable: $42.15\nToday's consumption: $1.00\nTotal consumption: $5.00"
        );
        assert_eq!(output.class, "moonshot-balance");
    }

    #[test]
    fn parses_waybar_mode_by_default() {
        assert_eq!(parse_mode_from(&[]), Mode::Waybar);
    }

    #[test]
    fn parses_notify_mode_from_flag() {
        assert_eq!(parse_mode_from(&["--notify"]), Mode::Notify);
    }

    fn parse_mode_from(args: &[&str]) -> Mode {
        let mut collected: Vec<String> = vec!["waybar-ai-api-balance".into()];
        collected.extend(args.iter().map(|s| s.to_string()));
        if collected.iter().any(|arg| arg == "--notify") {
            Mode::Notify
        } else {
            Mode::Waybar
        }
    }
}
