use crate::messages::Messages;
use crate::util::{Month, Quarter};
use chrono::NaiveDate;
use currency::{CurrencyValue, SCALE};
use eframe::egui::{RichText, WidgetText};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::path::PathBuf;
use uuid::Uuid;

pub(crate) mod currency;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Invoice {
    pub(crate) id: Uuid,
    pub(crate) date: NaiveDate,
    pub(crate) city: String,
    pub(crate) name: String,
    pub(crate) from: Address,
    pub(crate) to: Address,
    pub(crate) service_period: ServicePeriod,
    pub(crate) invoice_number: String,
    pub(crate) pre_text: String,
    pub(crate) post_text: String,
    pub(crate) bank_data: String,
    pub(crate) items: Vec<InvoiceItem>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct InvoiceItem {
    pub(crate) nr: u64,
    pub(crate) description: String,
    pub(crate) unit: Unit,
    pub(crate) amount: Decimal,
    pub(crate) price_per_unit: CurrencyValue,
    pub(crate) vat: Vat,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct Address {
    pub(crate) name: String,
    pub(crate) postal_address: String,
    pub(crate) zip: String,
    pub(crate) city: String,
    pub(crate) country: String,
    pub(crate) vat: String,
    pub(crate) misc: String,
}

impl Address {
    pub fn new() -> Self {
        Self {
            name: String::default(),
            postal_address: String::default(),
            zip: String::default(),
            city: String::default(),
            country: String::default(),
            vat: String::default(),
            misc: String::default(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct ServicePeriod {
    pub(crate) from: NaiveDate,
    pub(crate) from_field: String,
    pub(crate) to: NaiveDate,
    pub(crate) to_field: String,
}

#[derive(Debug, Clone, Eq, PartialEq, Copy, Serialize, Deserialize)]
pub(crate) enum Unit {
    Hour,
    Day,
    None,
}

impl Unit {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Unit::Hour => "h",
            Unit::Day => "d",
            Unit::None => "-",
        }
    }
}

#[derive(Debug)]
pub(crate) struct AccountingSheet {
    pub(crate) year: i32,
    pub(crate) quarter: Option<Quarter>,
    pub(crate) month: Option<Month>,
    pub(crate) items: Vec<AccountingItem>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub(crate) struct AccountingItem {
    pub(crate) invoice_type: InvoiceType,
    pub(crate) id: Uuid,
    pub(crate) date: NaiveDate,
    pub(crate) name: String,
    pub(crate) company: Company,
    pub(crate) category: Category,
    pub(crate) net: CurrencyValue,
    pub(crate) vat: Vat,
    pub(crate) file: PathBuf,
}

impl PartialOrd for AccountingItem {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AccountingItem {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.date.cmp(&other.date) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.name.cmp(&other.name) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.net.cmp(&other.net) {
            Ordering::Equal => {}
            ord => return ord,
        }
        match self.id.cmp(&other.id) {
            Ordering::Equal => {}
            ord => return ord,
        }
        Ordering::Equal
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Serialize, Deserialize)]
pub(crate) struct Company(pub(crate) String);

impl std::ops::Deref for Company {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Default, Hash, Serialize, Deserialize)]
pub(crate) struct Category(pub(crate) String);

impl std::ops::Deref for Category {
    type Target = str;

    fn deref(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Serialize, Deserialize)]
pub(crate) enum Vat {
    Zero,
    Ten,
    Twenty,
}

impl std::fmt::Display for Vat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

impl From<Vat> for WidgetText {
    fn from(val: Vat) -> Self {
        WidgetText::from(val.name())
    }
}

impl From<Vat> for RichText {
    fn from(val: Vat) -> Self {
        RichText::from(val.name())
    }
}

impl From<&Vat> for WidgetText {
    fn from(val: &Vat) -> Self {
        WidgetText::from(val.name())
    }
}

impl From<&Vat> for RichText {
    fn from(val: &Vat) -> Self {
        RichText::from(val.name())
    }
}

impl Vat {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Vat::Zero => "0 %",
            Vat::Ten => "10 %",
            Vat::Twenty => "20 %",
        }
    }

    pub(crate) fn value(&self) -> Decimal {
        match self {
            Vat::Zero => Decimal::new(0, SCALE),
            Vat::Ten => Decimal::new(10, SCALE),
            Vat::Twenty => Decimal::new(20, SCALE),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash, Serialize, Deserialize)]
pub(crate) enum InvoiceType {
    In,
    Out,
}

impl InvoiceType {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            InvoiceType::In => Messages::Ingoing.msg(),
            InvoiceType::Out => Messages::Outgoing.msg(),
        }
    }
}
