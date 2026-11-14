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
pub struct State {
    pub first_seen: Option<BalanceSnapshot>,
    pub day_start: Option<BalanceSnapshot>,
    pub last_seen: Option<BalanceSnapshot>,
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

    pub fn update(&mut self, now: BalanceSnapshot) -> Consumption {
        // A top-up makes the balance higher than the last reading. Reset the
        // baselines so consumption is calculated from the new starting point.
        if let Some(last) = self.last_seen {
            if now.available > last.available {
                self.first_seen = Some(now);
                self.day_start = Some(now);
                self.last_seen = Some(now);
                return Consumption { total: 0.0, today: 0.0 };
            }
        }

        let today = now.timestamp.date_naive();
        let day_start = self
            .day_start
            .and_then(|s| {
                if s.timestamp.date_naive() == today {
                    Some(s)
                } else {
                    None
                }
            })
            .unwrap_or(now);
        self.day_start = Some(day_start);

        if self.first_seen.is_none() {
            self.first_seen = Some(now);
        }
        self.last_seen = Some(now);

        let first = self.first_seen.expect("set above");
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
        let consumption = state.update(snap(100.0));
        assert_eq!(consumption, Consumption { total: 0.0, today: 0.0 });
        assert_eq!(state.first_seen.unwrap().available, 100.0);
        assert_eq!(state.day_start.unwrap().available, 100.0);
        assert_eq!(state.last_seen.unwrap().available, 100.0);
    }

    #[test]
    fn subsequent_update_reports_consumption_since_first_seen() {
        let mut state = State::default();
        state.update(snap(100.0));
        let consumption = state.update(snap(80.0));
        assert_eq!(consumption.total, 20.0);
        assert_eq!(consumption.today, 20.0);
    }

    #[test]
    fn top_up_resets_first_seen_and_day_start() {
        let mut state = State::default();
        state.update(snap(100.0));
        state.update(snap(80.0));
        let consumption = state.update(snap(120.0));
        assert_eq!(consumption, Consumption { total: 0.0, today: 0.0 });
        assert_eq!(state.first_seen.unwrap().available, 120.0);
        assert_eq!(state.day_start.unwrap().available, 120.0);
    }

    #[test]
    fn new_day_resets_day_start_but_keeps_total() {
        let mut state = State::default();
        let yesterday = Utc::now() - chrono::Duration::days(1);
        state.first_seen = Some(BalanceSnapshot {
            timestamp: yesterday,
            available: 100.0,
        });
        state.day_start = Some(BalanceSnapshot {
            timestamp: yesterday,
            available: 100.0,
        });
        state.last_seen = Some(BalanceSnapshot {
            timestamp: yesterday,
            available: 80.0,
        });

        let consumption = state.update(snap(70.0));
        assert_eq!(consumption.total, 30.0);
        assert_eq!(consumption.today, 0.0);
        assert_eq!(state.day_start.unwrap().available, 70.0);
    }
}
