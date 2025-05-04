use crate::messages::Messages;
use crate::{Event, GuiEvent};
use chrono::{Duration, NaiveDate};
use eframe::egui::Color32;
use eframe::egui::Context;
use log::error;
use std::sync::mpsc::Sender;

pub(crate) mod export;
pub(crate) mod files;
pub(crate) mod validation;

#[derive(Debug)]
pub(crate) enum Colors {
    Error,
    Warning,
    Info,
    ButtonDefault,
    ButtonActive,
}

impl Colors {
    pub(crate) fn col(&self) -> Color32 {
        match self {
            Colors::Error => Color32::LIGHT_RED,
            Colors::Warning => Color32::LIGHT_YELLOW,
            Colors::Info => Color32::LIGHT_GREEN,
            Colors::ButtonDefault => Color32::LIGHT_GRAY,
            Colors::ButtonActive => Color32::LIGHT_BLUE,
        }
    }
}

pub(crate) const VALID_FILETYPES: &[&str] = &["pdf", "png", "jpg", "jpeg", "gif"];

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Quarter {
    Q1,
    Q2,
    Q3,
    Q4,
}

impl Quarter {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Quarter::Q1 => "Q1",
            Quarter::Q2 => "Q2",
            Quarter::Q3 => "Q3",
            Quarter::Q4 => "Q4",
        }
    }

    pub(crate) fn start_and_end_months(&self) -> (u32, u32) {
        match self {
            Quarter::Q1 => (1, 3),
            Quarter::Q2 => (4, 6),
            Quarter::Q3 => (7, 9),
            Quarter::Q4 => (10, 12),
        }
    }

    pub(crate) fn from_month(month: u32) -> Self {
        match month {
            1..=3 => Quarter::Q1,
            4..=6 => Quarter::Q2,
            7..=9 => Quarter::Q3,
            10..=12 => Quarter::Q4,
            _ => Quarter::Q1,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub(crate) enum Month {
    January = 1,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,
}

impl Month {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Month::January => Messages::January.msg(),
            Month::February => Messages::February.msg(),
            Month::March => Messages::March.msg(),
            Month::April => Messages::April.msg(),
            Month::May => Messages::May.msg(),
            Month::June => Messages::June.msg(),
            Month::July => Messages::July.msg(),
            Month::August => Messages::August.msg(),
            Month::September => Messages::September.msg(),
            Month::October => Messages::October.msg(),
            Month::November => Messages::November.msg(),
            Month::December => Messages::December.msg(),
        }
    }

    pub(crate) fn short(&self) -> &'static str {
        match self {
            Month::January => Messages::Jan.msg(),
            Month::February => Messages::Feb.msg(),
            Month::March => Messages::Mar.msg(),
            Month::April => Messages::Apr.msg(),
            Month::May => Messages::May.msg(),
            Month::June => Messages::Jun.msg(),
            Month::July => Messages::Jul.msg(),
            Month::August => Messages::Aug.msg(),
            Month::September => Messages::Sep.msg(),
            Month::October => Messages::Oct.msg(),
            Month::November => Messages::Nov.msg(),
            Month::December => Messages::Dec.msg(),
        }
    }
}

impl From<u32> for Month {
    fn from(value: u32) -> Self {
        match value {
            1 => Month::January,
            2 => Month::February,
            3 => Month::March,
            4 => Month::April,
            5 => Month::May,
            6 => Month::June,
            7 => Month::July,
            8 => Month::August,
            9 => Month::September,
            10 => Month::October,
            11 => Month::November,
            12 => Month::December,
            _ => Month::January,
        }
    }
}

impl From<Month> for u32 {
    fn from(value: Month) -> Self {
        match value {
            Month::January => 1,
            Month::February => 2,
            Month::March => 3,
            Month::April => 4,
            Month::May => 5,
            Month::June => 6,
            Month::July => 7,
            Month::August => 8,
            Month::September => 9,
            Month::October => 10,
            Month::November => 11,
            Month::December => 12,
        }
    }
}

pub(crate) const QUARTERS: &[Quarter] = &[Quarter::Q1, Quarter::Q2, Quarter::Q3, Quarter::Q4];
pub(crate) const MONTHS: &[Month] = &[
    Month::January,
    Month::February,
    Month::March,
    Month::April,
    Month::May,
    Month::June,
    Month::July,
    Month::August,
    Month::September,
    Month::October,
    Month::November,
    Month::December,
];

pub(crate) fn send_event_and_request_repaint(ctx: &Context, sender: &Sender<Event>, event: Event) {
    match sender.send(event) {
        Ok(_) => {
            ctx.request_repaint();
        }
        Err(err) => {
            error!("Could not send event, {}", err);
        }
    }
}

pub(crate) fn send_gui_event(sender: &Sender<GuiEvent>, event: GuiEvent) {
    if let Err(err) = sender.send(event) {
        error!("Could not send gui event, {err}");
    }
}

pub(crate) fn last_day_of_month(year: i32, month: u32) -> NaiveDate {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };

    let first_day_next_month =
        NaiveDate::from_ymd_opt(next_year, next_month, 1).expect("is a valid date");

    first_day_next_month - Duration::days(1)
}
