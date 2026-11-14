pub mod deepseek;
pub mod kimi;

use crate::Error;
use crate::config::ProviderConfig;

const ICON_FONT_FAMILY: &str = "AI Provider Logos";

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
    fn fetch_balance(&self) -> Result<Balance, Error>;

    fn icon_markup(&self) -> String {
        icon_markup(self.id())
    }
}

pub fn icon_glyph(provider_id: &str) -> Option<&'static str> {
    match provider_id {
        "kimi" => Some("\u{E000}"),
        "deepseek" => Some("\u{E001}"),
        "claude" => Some("\u{E002}"),
        "ollama" => Some("\u{E003}"),
        "openrouter" => Some("\u{E004}"),
        _ => None,
    }
}

pub fn icon_markup(provider_id: &str) -> String {
    match icon_glyph(provider_id) {
        Some(glyph) => format!(
            r#"<span font_family="{}">{}</span>"#,
            ICON_FONT_FAMILY, glyph
        ),
        None => "●".to_string(),
    }
}

pub fn build(config: &ProviderConfig) -> Result<Box<dyn Provider>, Error> {
    match config.provider_type.as_str() {
        "kimi" => Ok(Box::new(kimi::KimiProvider::from_config(config))),
        "deepseek" => Ok(Box::new(deepseek::DeepSeekProvider::from_config(config))),
        other => Err(Error::UnknownProvider(other.into())),
    }
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
                currency: Currency::Usd,
            })
        }
    }

    #[test]
    fn provider_exposes_identity() {
        let p = TestProvider {
            id: "kimi",
            label: "Kimi",
        };
        assert_eq!(p.id(), "kimi");
        assert_eq!(p.label(), "Kimi");
    }

    #[test]
    fn known_providers_have_icon_glyph() {
        assert_eq!(icon_glyph("kimi"), Some("\u{E000}"));
        assert_eq!(icon_glyph("deepseek"), Some("\u{E001}"));
        assert_eq!(icon_glyph("ollama"), Some("\u{E003}"));
        assert_eq!(icon_glyph("openrouter"), Some("\u{E004}"));
    }

    #[test]
    fn unknown_provider_falls_back_to_bullet() {
        assert_eq!(icon_glyph("unknown"), None);
        assert_eq!(icon_markup("unknown"), "●");
    }

    #[test]
    fn icon_markup_uses_custom_font_family() {
        let markup = icon_markup("kimi");
        assert!(markup.contains(r#"font_family="AI Provider Logos""#));
        assert!(markup.contains('\u{E000}'));
    }

    #[test]
    fn formats_amounts_with_currency_symbol() {
        assert_eq!(format_amount(42.14854, Currency::Usd), "$42.15");
        assert_eq!(format_amount(100.0, Currency::Cny), "¥100.00");
    }
}
