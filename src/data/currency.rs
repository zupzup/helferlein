use super::Vat;
use eframe::egui::{RichText, WidgetText};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

pub const SCALE: u32 = 2;

pub fn default_currency_value() -> Decimal {
    Decimal::new(0, SCALE)
}

fn default_currency() -> Currency {
    Currency::Euro
}

#[derive(Debug)]
pub(crate) struct VatCalculationResult {
    pub(crate) tax: CurrencyValue,
    pub(crate) gross: CurrencyValue,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct CurrencyValue {
    pub(crate) value: Decimal,
    currency: Currency,
    formatted: String,
    formatted_value: String,
}

impl CurrencyValue {
    #[cfg(test)]
    pub fn new(num: i64) -> Self {
        let value = Decimal::new(num, SCALE);
        let currency = default_currency();
        Self {
            value,
            currency,
            formatted: format!("{} {}", value, currency.to_str()),
            formatted_value: value.to_string(),
        }
    }

    pub fn new_from_decimal(value: Decimal) -> Self {
        let currency = default_currency();
        let mut scaled_value = value;
        scaled_value.rescale(SCALE);
        Self {
            value,
            currency,
            formatted: format!("{} {}", scaled_value, currency.to_str(),),
            formatted_value: scaled_value.to_string(),
        }
    }

    pub fn calculate_vat(&self, vat: Vat) -> VatCalculationResult {
        let tax = Self::new_from_decimal(
            self.value
                .checked_mul(vat.value())
                .unwrap_or_else(default_currency_value),
        );
        let gross = Self::new_from_decimal(
            self.value
                .checked_add(tax.value)
                .unwrap_or_else(default_currency_value),
        );

        VatCalculationResult { tax, gross }
    }

    pub fn to_str(&self) -> &str {
        &self.formatted
    }

    pub fn to_euro_str(&self) -> String {
        format!(
            "{} {}",
            default_currency().to_str(),
            format_euro_string(&self.value)
        )
    }

    pub fn _to_value_str(&self) -> &str {
        &self.formatted_value
    }

    pub fn to_value_string(&self) -> String {
        self.formatted_value.clone()
    }
}

impl PartialOrd for CurrencyValue {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CurrencyValue {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

fn format_euro_string(num: &Decimal) -> String {
    let mut scaled_value = num.to_owned();
    scaled_value.rescale(SCALE);
    let input = scaled_value.to_string();
    let parts: Vec<&str> = input.split('.').collect();
    let with_minus = input.starts_with('-');
    let cut_off = if with_minus { 4 } else { 3 };

    let int_part = parts[0];
    let dec_part = parts[1];
    let int_formatted = if int_part.len() > cut_off {
        let mut result = String::new();
        let chars: Vec<char> = int_part.chars().rev().collect();

        for (i, c) in chars.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                result.push('.');
            }
            result.push(*c);
        }
        result.chars().rev().collect()
    } else {
        int_part.to_string()
    };

    if with_minus {
        format!("- {},{}", &int_formatted[1..], dec_part)
    } else {
        format!("{},{}", int_formatted, dec_part)
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) enum Currency {
    Euro,
}

impl Currency {
    fn to_str(self) -> &'static str {
        match self {
            Currency::Euro => "€",
        }
    }
}

impl From<CurrencyValue> for WidgetText {
    fn from(val: CurrencyValue) -> Self {
        WidgetText::from(val.to_str())
    }
}

impl From<&CurrencyValue> for WidgetText {
    fn from(val: &CurrencyValue) -> Self {
        WidgetText::from(val.to_str())
    }
}

impl From<CurrencyValue> for RichText {
    fn from(val: CurrencyValue) -> Self {
        RichText::from(val.to_str())
    }
}

impl From<&CurrencyValue> for RichText {
    fn from(val: &CurrencyValue) -> Self {
        RichText::from(val.to_str())
    }
}

impl From<CurrencyValue> for String {
    fn from(val: CurrencyValue) -> Self {
        val.to_string()
    }
}

impl std::fmt::Display for CurrencyValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_str())
    }
}
