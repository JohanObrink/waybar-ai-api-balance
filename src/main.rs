mod config;
mod providers;
mod state;

use std::env;
use std::path::PathBuf;
use std::process::Command;

use chrono::Utc;
use serde::Serialize;
use thiserror::Error;

use providers::{build, format_amount, Balance, Provider};
use state::{BalanceSnapshot, Consumption, State};

#[derive(Debug, Error)]
enum Error {
    #[error("missing API key: set {0}")]
    MissingApiKey(String),
    #[error("unknown provider type: {0}")]
    UnknownProvider(String),
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),
    #[error("invalid response: {0}")]
    InvalidResponse(String),
    #[error("notification failed")]
    Notification,
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

struct ProviderOutput {
    id: &'static str,
    text: String,
    tooltip: String,
}

fn main() -> Result<(), Error> {
    let config_path = config_path();
    let config = config::Config::load(&config_path);
    let providers: Vec<Box<dyn Provider>> = config
        .provider
        .iter()
        .map(build)
        .collect::<Result<Vec<_>, _>>()?;
    let mode = parse_mode();
    let state_path = state_path();
    let mut state = State::load(&state_path);

    let mut outputs = Vec::new();
    for provider in providers.iter().map(|p| p.as_ref()) {
        match provider.fetch_balance() {
            Ok(balance) => {
                let consumption = state.update(
                    provider.id(),
                    BalanceSnapshot {
                        timestamp: Utc::now(),
                        available: balance.available,
                    },
                );
                outputs.push(provider_output(provider, &balance, &consumption));
            }
            Err(e) => outputs.push(error_output(provider, e)),
        }
    }
    state.save(&state_path);

    match mode {
        Mode::Waybar => println!("{}", serde_json::to_string(&combine_outputs(outputs)).unwrap()),
        Mode::Notify => send_notifications(outputs)?,
    }

    Ok(())
}

fn config_path() -> PathBuf {
    dirs::config_dir()
        .map(|p| p.join("waybar-ai-api-balance/config.toml"))
        .unwrap_or_else(|| PathBuf::from("/tmp/waybar-ai-api-balance-config.toml"))
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

fn provider_output(provider: &dyn Provider, balance: &Balance, consumption: &Consumption) -> ProviderOutput {
    ProviderOutput {
        id: provider.id(),
        text: format!("{} {}", provider.icon_markup(), balance.available_text()),
        tooltip: format!(
            "{}\nAvailable: {}\nToday's consumption: {}\nTotal consumption: {}",
            provider.label(),
            balance.available_text(),
            format_amount(consumption.today, balance.currency),
            format_amount(consumption.total, balance.currency)
        ),
    }
}

fn error_output(provider: &dyn Provider, error: Error) -> ProviderOutput {
    ProviderOutput {
        id: provider.id(),
        text: format!("{} ?", provider.icon_markup()),
        tooltip: format!("{}\nError: {}", provider.label(), error),
    }
}

fn combine_outputs(outputs: Vec<ProviderOutput>) -> WaybarOutput {
    let text = outputs
        .iter()
        .map(|o| o.text.clone())
        .collect::<Vec<_>>()
        .join("  ");
    let tooltip = outputs
        .iter()
        .map(|o| o.tooltip.clone())
        .collect::<Vec<_>>()
        .join("\n\n");
    let class = outputs
        .iter()
        .map(|o| format!("{}-balance", o.id))
        .collect::<Vec<_>>()
        .join(" ");
    WaybarOutput {
        text,
        tooltip,
        class,
    }
}

fn send_notifications(outputs: Vec<ProviderOutput>) -> Result<(), Error> {
    for output in outputs {
        let status = Command::new("notify-send")
            .arg(output.id)
            .arg(output.tooltip)
            .status()
            .map_err(|_| Error::Notification)?;
        if !status.success() {
            return Err(Error::Notification);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        id: &'static str,
        label: &'static str,
    }

    impl Provider for TestProvider {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            self.label
        }
        fn fetch_balance(&self) -> Result<Balance, Error> {
            Ok(Balance {
                available: 10.0,
                currency: providers::Currency::Usd,
            })
        }
    }

    #[test]
    fn parses_waybar_mode_by_default() {
        assert_eq!(parse_mode_from(&[]), Mode::Waybar);
    }

    #[test]
    fn parses_notify_mode_from_flag() {
        assert_eq!(parse_mode_from(&["--notify"]), Mode::Notify);
    }

    #[test]
    fn provider_output_includes_icon_and_balance() {
        let provider = TestProvider {
            id: "kimi",
            label: "Kimi",
        };
        let balance = Balance {
            available: 42.0,
            currency: providers::Currency::Usd,
        };
        let consumption = Consumption { today: 1.0, total: 5.0 };
        let output = provider_output(&provider, &balance, &consumption);
        assert!(output.text.contains(r#"font_family="AI Provider Logos""#));
        assert!(output.text.contains("$42.00"));
        assert!(output.tooltip.contains("Available: $42.00"));
        assert!(output.tooltip.contains("Today's consumption: $1.00"));
        assert!(output.tooltip.contains("Total consumption: $5.00"));
    }

    #[test]
    fn combines_multiple_provider_outputs() {
        let outputs = vec![
            ProviderOutput {
                id: "kimi",
                text: "<span>kimi-icon</span> $10.00".into(),
                tooltip: "Kimi details".into(),
            },
            ProviderOutput {
                id: "openai",
                text: "<span>openai-icon</span> $5.00".into(),
                tooltip: "OpenAI details".into(),
            },
        ];
        let combined = combine_outputs(outputs);
        assert!(combined.text.contains("kimi-icon"));
        assert!(combined.text.contains("openai-icon"));
        assert!(combined.text.contains("$10.00"));
        assert!(combined.text.contains("$5.00"));
        assert_eq!(combined.tooltip, "Kimi details\n\nOpenAI details");
        assert_eq!(combined.class, "kimi-balance openai-balance");
    }

    #[test]
    fn error_output_shows_icon_and_message() {
        let provider = TestProvider {
            id: "deepseek",
            label: "DeepSeek",
        };
        let output = error_output(&provider, Error::MissingApiKey("DEEPSEEK_API_KEY".into()));
        assert!(output.text.contains(r#"font_family="AI Provider Logos""#));
        assert!(output.text.contains('?'));
        assert!(output.tooltip.contains("DeepSeek"));
        assert!(output.tooltip.contains("DEEPSEEK_API_KEY"));
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
