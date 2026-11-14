pub mod kimi;

use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Currency {
    Usd,
    Cny,
}

#[derive(Debug, Clone)]
pub struct Balance {
    pub available: f64,
    pub currency: Currency,
}

impl Balance {
    pub fn available_text(&self) -> String {
        format_amount(self.available, self.currency)
    }
}

pub trait Provider {
    fn id(&self) -> &'static str;
    fn label(&self) -> &'static str;
    fn icon(&self) -> &'static str;
    fn fetch_balance(&self) -> Result<Balance, Error>;
}

pub fn format_amount(amount: f64, currency: Currency) -> String {
    let symbol = match currency {
        Currency::Usd => '$',
        Currency::Cny => '¥',
    };
    format!("{symbol}{amount:.2}")
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestProvider {
        id: &'static str,
        label: &'static str,
        icon: &'static str,
    }

    impl Provider for TestProvider {
        fn id(&self) -> &'static str {
            self.id
        }
        fn label(&self) -> &'static str {
            self.label
        }
        fn icon(&self) -> &'static str {
            self.icon
        }
        fn fetch_balance(&self) -> Result<Balance, Error> {
            Ok(Balance {
                available: 10.0,
                currency: Currency::Usd,
            })
        }
    }

    #[test]
    fn provider_exposes_identity() {
        let p = TestProvider {
            id: "kimi",
            label: "Kimi",
            icon: "",
        };
        assert_eq!(p.id(), "kimi");
        assert_eq!(p.label(), "Kimi");
        assert_eq!(p.icon(), "");
    }

    #[test]
    fn formats_amounts_with_currency_symbol() {
        assert_eq!(format_amount(42.14854, Currency::Usd), "$42.15");
        assert_eq!(format_amount(100.0, Currency::Cny), "¥100.00");
    }
}
