use crate::data::Invoice;
use crate::util::{self, Month, Quarter};
use crate::{GuiError, data::AccountingItem};
use chrono::{Datelike, NaiveDate};
use redb::{Database, ReadableTable, TableDefinition, TypeName, Value, WriteTransaction};
use serde::{Deserialize, Serialize};
use std::any::type_name;
use std::fmt::Debug;
use std::path::Path;

const DB_FILE: &str = "helferlein.redb";

const ACCOUNTING_ITEMS_TABLE: TableDefinition<&str, Bincode<AccountingItem>> =
    TableDefinition::new("accounting_items");
const NAMES_TABLE: TableDefinition<&str, Bincode<Vec<String>>> = TableDefinition::new("names");
const COMPANIES_TABLE: TableDefinition<&str, Bincode<Vec<String>>> =
    TableDefinition::new("companies");
const CATEGORIES_TABLE: TableDefinition<&str, Bincode<Vec<String>>> =
    TableDefinition::new("categories");
const INVOICES_TABLE: TableDefinition<&str, Bincode<Invoice>> = TableDefinition::new("invoices");

/// This can only be called once
fn get_db(data_folder: &Path) -> Database {
    let db_file = DB_FILE;
    let path = data_folder.join(db_file);

    let db = Database::create(path).expect("can create/open db file");
    if let Ok(write_txn) = db.begin_write() {
        let _ = write_txn.open_table(NAMES_TABLE);
        let _ = write_txn.open_table(COMPANIES_TABLE);
        let _ = write_txn.open_table(CATEGORIES_TABLE);
        let _ = write_txn.open_table(INVOICES_TABLE);
        let _ = write_txn.open_table(ACCOUNTING_ITEMS_TABLE);
        let _ = write_txn.commit();
    }

    db
}
#[derive(Debug, Clone)]
pub struct DateRange {
    pub from: String,
    pub to: String,
}

pub fn get_date_range_for_settings(
    year: i32,
    quarter: Option<Quarter>,
    month: Option<Month>,
) -> DateRange {
    let range_from = match quarter {
        None => match month {
            None => {
                format!("{year}-01-01")
            }
            Some(m) => {
                let month_num: u32 = m.into();
                let date_from = NaiveDate::from_ymd_opt(year, month_num, 1);
                match date_from {
                    None => {
                        format!("{year}-01-01")
                    }
                    Some(date) => date.format(KEY_DATE_FORMAT).to_string(),
                }
            }
        },
        Some(q) => {
            let (from, _) = q.start_and_end_months();
            let date_from = NaiveDate::from_ymd_opt(year, from, 1);
            match date_from {
                None => {
                    format!("{year}-01-01")
                }
                Some(date) => date.format(KEY_DATE_FORMAT).to_string(),
            }
        }
    };
    let range_to = match quarter {
        None => match month {
            None => {
                format!("{year}-12-31")
            }
            Some(m) => {
                let month_num: u32 = m.into();
                let date_from = NaiveDate::from_ymd_opt(year, month_num, 1);
                match date_from {
                    None => {
                        format!("{year}-12-31")
                    }
                    Some(date) => {
                        let last_day = util::last_day_of_month(date.year(), date.month());
                        last_day.format(KEY_DATE_FORMAT).to_string()
                    }
                }
            }
        },
        Some(q) => {
            let (_, to) = q.start_and_end_months();
            let date_to = NaiveDate::from_ymd_opt(year, to, 1);
            match date_to {
                None => {
                    format!("{year}-12-31")
                }
                Some(date) => {
                    let last_day = util::last_day_of_month(date.year(), date.month());
                    last_day.format(KEY_DATE_FORMAT).to_string()
                }
            }
        }
    };
    DateRange {
        from: range_from,
        to: range_to,
    }
}

pub(crate) const KEY_DATE_FORMAT: &str = "%Y-%m-%d";

#[derive(Debug)]
pub(crate) struct DB {
    db: Database,
}

impl DB {
    /// This can only be called once
    pub(crate) fn new(data_folder: &Path) -> Self {
        Self {
            db: get_db(data_folder),
        }
    }

    pub(crate) fn get_key_for_item(item: &AccountingItem) -> String {
        format!("{}_{}", item.date.format(KEY_DATE_FORMAT), item.id)
    }

    pub(crate) fn get_key_for_invoice(invoice: &Invoice) -> String {
        format!("{}_{}", invoice.date.format(KEY_DATE_FORMAT), invoice.id)
    }

    // ACCOUNTING ITEMS
    pub(crate) fn get_accounting_items_for_range(
        &self,
        date_range: &DateRange,
    ) -> Result<Vec<AccountingItem>, GuiError> {
        let table = self
            .db
            .begin_read()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?
            .open_table(ACCOUNTING_ITEMS_TABLE)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        // add \x7f, because it compares bit-wise, so date{something} doesn't match date_a324
        let iter = table
            .range(date_range.from.as_str()..=format!("{}\x7f", date_range.to.as_str()).as_str())
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        Ok(iter.filter_map(|r| r.map(|v| v.1.value()).ok()).collect())
    }

    fn fetch_invoice_templates(
        &self,
        write_txn: &WriteTransaction,
    ) -> Result<Vec<Invoice>, GuiError> {
        let table = write_txn
            .open_table(INVOICES_TABLE)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        let iter = table
            .iter()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        Ok(iter.filter_map(|r| r.map(|v| v.1.value()).ok()).collect())
    }

    pub(crate) fn get_invoice_templates(&self) -> Result<Vec<Invoice>, GuiError> {
        let table = self
            .db
            .begin_read()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?
            .open_table(INVOICES_TABLE)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        let iter = table
            .iter()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        Ok(iter.filter_map(|r| r.map(|v| v.1.value()).ok()).collect())
    }

    pub(crate) fn create_invoice_template_and_refetch(
        &self,
        invoice: &Invoice,
    ) -> Result<Vec<Invoice>, GuiError> {
        let key = DB::get_key_for_invoice(invoice);
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INVOICES_TABLE)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

            table
                .insert(key.as_str(), invoice)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        }
        let res = self
            .fetch_invoice_templates(&write_txn)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        write_txn
            .commit()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        Ok(res)
    }

    pub(crate) fn delete_invoice_template_and_refetch(
        &self,
        key: &str,
    ) -> Result<Vec<Invoice>, GuiError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(INVOICES_TABLE)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

            table
                .remove(key)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        }
        let res = self
            .fetch_invoice_templates(&write_txn)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        write_txn
            .commit()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        Ok(res)
    }

    pub(crate) fn create_or_update_accounting_item_and_refetch(
        &self,
        item: &AccountingItem,
        date_range: &DateRange,
    ) -> Result<Vec<AccountingItem>, GuiError> {
        let key = DB::get_key_for_item(item);
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        {
            let mut table = write_txn
                .open_table(ACCOUNTING_ITEMS_TABLE)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

            self.create_or_update_name(&item.name, key.clone(), &write_txn)?;
            self.create_or_update_category(&item.category, key.clone(), &write_txn)?;
            self.create_or_update_company(&item.company, key.clone(), &write_txn)?;

            table
                .insert(key.as_str(), item)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        }

        let res = self
            .fetch_accounting_items_by_range(&write_txn, date_range)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        write_txn
            .commit()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        Ok(res)
    }

    fn fetch_accounting_items_by_range(
        &self,
        write_txn: &WriteTransaction,
        date_range: &DateRange,
    ) -> Result<Vec<AccountingItem>, GuiError> {
        let table = write_txn
            .open_table(ACCOUNTING_ITEMS_TABLE)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        // add \x7f, because it compares bit-wise, so date{something} doesn't match date_a324
        let iter = table
            .range(date_range.from.as_str()..=format!("{}\x7f", date_range.to.as_str()).as_str())
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        Ok(iter.filter_map(|r| r.map(|v| v.1.value()).ok()).collect())
    }

    pub(crate) fn delete_accounting_item_and_refetch(
        &self,
        key: &str,
        date_range: &DateRange,
    ) -> Result<Vec<AccountingItem>, GuiError> {
        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        {
            let mut table = write_txn
                .open_table(ACCOUNTING_ITEMS_TABLE)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

            let res = table
                .get(key)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?
                .map(|v| v.value());

            let value = match res {
                None => {
                    return Err(GuiError::DatabaseError(format!(
                        "Item {key} does not exist and can't be deleted."
                    )));
                }
                Some(v) => v,
            };

            self.remove_name(&value.name, key, &write_txn)?;
            self.remove_category(&value.name, key, &write_txn)?;
            self.remove_company(&value.name, key, &write_txn)?;

            table
                .remove(key)
                .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        }

        let res = self
            .fetch_accounting_items_by_range(&write_txn, date_range)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        write_txn
            .commit()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        Ok(res)
    }

    // NAMES / CATEGORIES / COMPANIES
    pub(crate) fn get_all_names(&self) -> Result<Vec<String>, GuiError> {
        self.get_all(NAMES_TABLE)
    }

    pub(crate) fn get_all_companies(&self) -> Result<Vec<String>, GuiError> {
        self.get_all(COMPANIES_TABLE)
    }

    pub(crate) fn get_all_categories(&self) -> Result<Vec<String>, GuiError> {
        self.get_all(CATEGORIES_TABLE)
    }

    fn get_all(
        &self,
        table: TableDefinition<&str, Bincode<Vec<String>>>,
    ) -> Result<Vec<String>, GuiError> {
        let table = self
            .db
            .begin_read()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?
            .open_table(table)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        let iter = table
            .iter()
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        Ok(iter
            .filter_map(|r| r.map(|v| v.0.value().to_owned()).ok())
            .collect())
    }

    fn create_or_update_name(
        &self,
        key: &str,
        accounting_item_key: String,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.create_or_update(key, accounting_item_key, write_txn, NAMES_TABLE)
    }

    fn create_or_update_category(
        &self,
        key: &str,
        accounting_item_key: String,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.create_or_update(key, accounting_item_key, write_txn, CATEGORIES_TABLE)
    }

    fn create_or_update_company(
        &self,
        key: &str,
        accounting_item_key: String,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.create_or_update(key, accounting_item_key, write_txn, COMPANIES_TABLE)
    }

    fn create_or_update(
        &self,
        key: &str,
        accounting_item_key: String,
        write_txn: &WriteTransaction,
        table: TableDefinition<&str, Bincode<Vec<String>>>,
    ) -> Result<(), GuiError> {
        let mut table = write_txn
            .open_table(table)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
        let item = table
            .get(key)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?
            .map(|v| v.value());
        match item {
            None => {
                let accounting_item_keys = vec![accounting_item_key];
                table
                    .insert(key, accounting_item_keys)
                    .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
            }
            Some(mut v) => {
                if !v.contains(&accounting_item_key) {
                    v.push(accounting_item_key);
                    table
                        .insert(key, v)
                        .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
                }
            }
        };
        Ok(())
    }

    fn remove_name(
        &self,
        key: &str,
        accounting_item_key: &str,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.remove(key, accounting_item_key, write_txn, NAMES_TABLE)
    }

    fn remove_category(
        &self,
        key: &str,
        accounting_item_key: &str,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.remove(key, accounting_item_key, write_txn, CATEGORIES_TABLE)
    }

    fn remove_company(
        &self,
        key: &str,
        accounting_item_key: &str,
        write_txn: &WriteTransaction,
    ) -> Result<(), GuiError> {
        self.remove(key, accounting_item_key, write_txn, COMPANIES_TABLE)
    }

    fn remove(
        &self,
        key: &str,
        accounting_item_key: &str,
        write_txn: &WriteTransaction,
        table: TableDefinition<&str, Bincode<Vec<String>>>,
    ) -> Result<(), GuiError> {
        let mut table = write_txn
            .open_table(table)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;

        let res = table
            .get(key)
            .map_err(|e| GuiError::DatabaseError(e.to_string()))?
            .map(|v| v.value());
        match res {
            None => Ok(()),
            Some(mut v) => match v.iter().position(|v| *v == accounting_item_key) {
                None => Ok(()),
                Some(found) => {
                    v.remove(found);
                    // if it's the last item, remove the entire entry
                    if v.is_empty() {
                        table
                            .remove(key)
                            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
                    } else {
                        table
                            .insert(key, v)
                            .map_err(|e| GuiError::DatabaseError(e.to_string()))?;
                    }
                    Ok(())
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct Bincode<T>(pub T);

impl<T> Value for Bincode<T>
where
    T: Debug + Serialize + for<'a> Deserialize<'a>,
{
    type SelfType<'a>
        = T
    where
        Self: 'a;
    type AsBytes<'a>
        = Vec<u8>
    where
        Self: 'a;

    fn as_bytes<'a, 'b: 'a>(value: &'a Self::SelfType<'b>) -> Self::AsBytes<'a>
    where
        Self: 'a,
        Self: 'b,
    {
        bincode::serialize(value).expect("can serialize with bincode")
    }

    fn fixed_width() -> Option<usize> {
        None
    }

    fn from_bytes<'a>(data: &'a [u8]) -> Self::SelfType<'a>
    where
        Self: 'a,
    {
        bincode::deserialize(data).expect("can deserialize using bincode")
    }

    fn type_name() -> redb::TypeName {
        TypeName::new(&format!("Bincode<{}>", type_name::<T>()))
    }
}
