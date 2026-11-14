use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct BalanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub available: f64,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
struct ProviderState {
    first_seen: Option<BalanceSnapshot>,
    day_start: Option<BalanceSnapshot>,
    last_seen: Option<BalanceSnapshot>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct State {
    providers: HashMap<String, ProviderState>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Consumption {
    pub total: f64,
    pub today: f64,
}

impl State {
    pub fn load(path: &PathBuf) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|contents| serde_json::from_str(&contents).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, path: &PathBuf) {
        if let Some(parent) = path.parent() {
            let _ = fs::create_dir_all(parent);
        }
        if let Ok(contents) = serde_json::to_string_pretty(self) {
            let _ = fs::write(path, contents);
        }
    }

    pub fn update(&mut self, provider_id: &str, now: BalanceSnapshot) -> Consumption {
        let provider = self.providers.entry(provider_id.into()).or_default();

        if let Some(last) = provider.last_seen {
            if now.available > last.available {
                provider.first_seen = Some(now);
                provider.day_start = Some(now);
                provider.last_seen = Some(now);
                return Consumption { total: 0.0, today: 0.0 };
            }
        }

        let today = now.timestamp.date_naive();
        let day_start = provider
            .day_start
            .and_then(|s| {
                if s.timestamp.date_naive() == today {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or(now);
        provider.day_start = Some(day_start);

        if provider.first_seen.is_none() {
            provider.first_seen = Some(now);
        }
        provider.last_seen = Some(now);

        let first = provider.first_seen.expect("set above");
        Consumption {
            total: (first.available - now.available).max(0.0),
            today: (day_start.available - now.available).max(0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snap(available: f64) -> BalanceSnapshot {
        BalanceSnapshot {
            timestamp: Utc::now(),
            available,
        }
    }

    #[test]
    fn first_update_records_initial_snapshot_and_reports_zero_consumption() {
        let mut state = State::default();
        let consumption = state.update("kimi", snap(100.0));
        assert_eq!(consumption, Consumption { total: 0.0, today: 0.0 });
    }

    #[test]
    fn subsequent_update_reports_consumption_since_first_seen() {
        let mut state = State::default();
        state.update("kimi", snap(100.0));
        let consumption = state.update("kimi", snap(80.0));
        assert_eq!(consumption.total, 20.0);
        assert_eq!(consumption.today, 20.0);
    }

    #[test]
    fn top_up_resets_first_seen_and_day_start() {
        let mut state = State::default();
        state.update("kimi", snap(100.0));
        state.update("kimi", snap(80.0));
        let consumption = state.update("kimi", snap(120.0));
        assert_eq!(consumption, Consumption { total: 0.0, today: 0.0 });
    }

    #[test]
    fn new_day_resets_day_start_but_keeps_total() {
        let mut state = State::default();
        let yesterday = Utc::now() - chrono::Duration::days(1);
        state.providers.insert(
            "kimi".into(),
            ProviderState {
                first_seen: Some(BalanceSnapshot {
                    timestamp: yesterday,
                    available: 100.0,
                }),
                day_start: Some(BalanceSnapshot {
                    timestamp: yesterday,
                    available: 100.0,
                }),
                last_seen: Some(BalanceSnapshot {
                    timestamp: yesterday,
                    available: 80.0,
                }),
            },
        );

        let consumption = state.update("kimi", snap(70.0));
        assert_eq!(consumption.total, 30.0);
        assert_eq!(consumption.today, 0.0);
    }

    #[test]
    fn providers_are_tracked_independently() {
        let mut state = State::default();
        state.update("kimi", snap(100.0));
        state.update("openai", snap(50.0));
        let kimi = state.update("kimi", snap(90.0));
        let openai = state.update("openai", snap(40.0));
        assert_eq!(kimi.total, 10.0);
        assert_eq!(openai.total, 10.0);
    }
}
