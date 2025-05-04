use crate::util::{Month, Quarter, last_day_of_month};
use chrono::{Datelike, NaiveDate};
use std::collections::HashMap;

#[derive(Debug, Eq, Hash, PartialEq)]
pub(crate) enum Field {
    Date,
    ServicePeriodFrom,
    ServicePeriodTo,
    City,
    Name,
    ToName,
    ToAddress,
    ToZip,
    ToCity,
    ToCountry,
    ToVat,
    ToMisc,
    FromName,
    FromAddress,
    FromZip,
    FromCity,
    FromCountry,
    FromVat,
    FromMisc,
    Description,
    Nr,
    Company,
    Category,
    Net,
    File,
    Amount,
    PricePerUnit,
}

#[derive(Debug)]
pub(crate) struct ValidationResult {
    warnings: HashMap<Field, Vec<String>>,
    errors: HashMap<Field, Vec<String>>,
}

impl ValidationResult {
    pub(crate) fn new() -> Self {
        Self {
            warnings: HashMap::new(),
            errors: HashMap::new(),
        }
    }

    pub(crate) fn clear_for_field(&mut self, field: &Field) {
        self.warnings.remove(field);
        self.errors.remove(field);
    }

    pub(crate) fn is_ok(&self) -> bool {
        !self.has_errors() && !self.has_warnings()
    }

    pub(crate) fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
    }

    pub(crate) fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    pub(crate) fn get_warnings(&self, field: &Field) -> Option<&Vec<String>> {
        self.warnings.get(field)
    }

    pub(crate) fn get_errors(&self, field: &Field) -> Option<&Vec<String>> {
        self.errors.get(field)
    }

    pub(crate) fn add_warning(&mut self, field: Field, msg: String) {
        match self.warnings.get_mut(&field) {
            None => {
                self.warnings.insert(field, vec![msg]);
            }
            Some(warnings) => {
                warnings.push(msg);
            }
        };
    }

    pub(crate) fn add_error(&mut self, field: Field, msg: String) {
        match self.errors.get_mut(&field) {
            None => {
                self.errors.insert(field, vec![msg]);
            }
            Some(errors) => {
                errors.push(msg);
            }
        };
    }
}

pub(crate) fn is_date_in_selected_time_span(
    selected_date: NaiveDate,
    year: i32,
    selected_quarter: Option<Quarter>,
    selected_month: Option<Month>,
) -> bool {
    if let Some(quarter) = selected_quarter {
        let (start, end) = quarter.start_and_end_months();
        let start_of_quarter = NaiveDate::from_ymd_opt(year, start, 1).expect("is a valid date");
        let end_of_quarter = last_day_of_month(year, end);
        return selected_date.ge(&start_of_quarter) && selected_date.le(&end_of_quarter);
    }

    if let Some(month) = selected_month {
        let start_of_month =
            NaiveDate::from_ymd_opt(year, month.into(), 1).expect("is a valid date");
        let end_of_month = last_day_of_month(year, month.into());
        return selected_date.ge(&start_of_month) && selected_date.le(&end_of_month);
    }
    selected_date.year() == year
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_year() {
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2022,
            None,
            None,
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2022,
            Some(Quarter::Q2),
            None,
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2022,
            None,
            Some(Month::March),
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2022,
            Some(Quarter::Q2),
            Some(Month::March),
        ));
    }
    #[test]
    fn no_quarter_month() {
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            None,
            None,
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2016,
            None,
            None,
        ));
    }

    #[test]
    fn quarter() {
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 31).unwrap(),
            2015,
            Some(Quarter::Q1),
            None,
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            Some(Quarter::Q1),
            None,
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 5, 1).unwrap(),
            2015,
            Some(Quarter::Q2),
            None,
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 9, 1).unwrap(),
            2015,
            Some(Quarter::Q3),
            None,
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 11, 1).unwrap(),
            2015,
            Some(Quarter::Q4),
            None,
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            Some(Quarter::Q2),
            None,
        ));
    }

    #[test]
    fn month() {
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            None,
            Some(Month::March),
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 12, 31).unwrap(),
            2015,
            None,
            Some(Month::December),
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            None,
            Some(Month::May),
        ));
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2024, 2, 29).unwrap(),
            2024,
            None,
            Some(Month::February),
        ));
    }

    #[test]
    fn quarter_over_month() {
        assert!(is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            Some(Quarter::Q1),
            Some(Month::May),
        ));
        assert!(!is_date_in_selected_time_span(
            NaiveDate::from_ymd_opt(2015, 3, 1).unwrap(),
            2015,
            Some(Quarter::Q2),
            Some(Month::May),
        ));
    }
}
