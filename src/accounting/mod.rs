use crate::{
    AppContext, DATE_FORMAT, Event, GuiEvent, State,
    config::Config,
    data::{
        AccountingItem, AccountingSheet, Category, Company, InvoiceType, Vat,
        currency::CurrencyValue,
    },
    db::get_date_range_for_settings,
    messages::Messages,
    ui::{self, autosuggest::AutoSuggest, dialog::Dialog},
    util::{
        self, MONTHS, Month, QUARTERS, Quarter,
        export::accounting::{CreatePDFResult, create_accounting_pdf},
        files::{build_file_name_suggestion, copy_file_and_rename, delete_file_and_folder},
        validation::{Field, ValidationResult, is_date_in_selected_time_span},
    },
};
use chrono::{Datelike, NaiveDate};
use eframe::egui::{ComboBox, Context, Grid, RichText, SelectableLabel, Ui};
use egui_file::FileDialog;
use log::info;
use rust_decimal::Decimal;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use uuid::Uuid;

mod add_edit;
mod items_table;

#[derive(Debug, PartialEq)]
pub(crate) enum Mode {
    Add,
    Edit,
}

#[derive(Debug)]
pub(crate) struct AccountingState {
    pub(crate) selected_year: i32,
    pub(crate) selected_quarter: Option<Quarter>,
    pub(crate) selected_month: Option<Month>,
    pub(crate) selected_accounting_sheet: Option<AccountingSheet>,
    quarter_selector_selected: Option<Quarter>,
    month_selector_selected: Option<Month>,
    year_selector_selected: i32,
    item: Item,
    mode: Mode,
    export_state: ExportState,
    pub(crate) names: Vec<String>,
    pub(crate) companies: Vec<String>,
    pub(crate) categories: Vec<String>,
}

impl AccountingState {
    pub(crate) fn new() -> Self {
        let now = chrono::Local::now();
        let month = now.month();

        Self {
            selected_year: now.year(),
            selected_quarter: None,
            selected_month: None,
            selected_accounting_sheet: None,
            quarter_selector_selected: Some(Quarter::from_month(month)),
            month_selector_selected: None,
            year_selector_selected: now.year(),
            item: Item::new().hidden(),
            mode: Mode::Add,
            export_state: ExportState::new(),
            names: vec![],
            companies: vec![],
            categories: vec![],
        }
    }
}

#[derive(Debug)]
struct ExportState {
    open_file_dialog: Option<FileDialog>,
    selected_path: Option<PathBuf>,
}

impl ExportState {
    pub fn new() -> Self {
        Self {
            open_file_dialog: None,
            selected_path: None,
        }
    }
}

#[derive(Debug)]
struct Item {
    id: Uuid,
    show: bool,
    focus_first_element: bool,
    invoice_type: InvoiceType,
    date: NaiveDate,
    date_field: String,
    name: String,
    name_autosuggest: AutoSuggest,
    company: String,
    company_autosuggest: AutoSuggest,
    category: String,
    category_autosuggest: AutoSuggest,
    net: String,
    vat: Vat,
    file: PathBuf,
    open_file_dialog: Option<FileDialog>,
    validation: ValidationResult,
    save_dialog: Option<Dialog>,
}

impl From<&AccountingItem> for Item {
    fn from(item: &AccountingItem) -> Self {
        Self {
            id: item.id,
            show: true,
            focus_first_element: true,
            invoice_type: item.invoice_type,
            date: item.date,
            date_field: item.date.format(DATE_FORMAT).to_string(),
            name: item.name.to_owned(),
            name_autosuggest: AutoSuggest::new(),
            company: item.company.0.to_owned(),
            company_autosuggest: AutoSuggest::new(),
            category: item.category.0.to_owned(),
            category_autosuggest: AutoSuggest::new(),
            net: item.net.to_value_string(),
            vat: item.vat,
            file: item.file.to_path_buf(),
            open_file_dialog: None,
            validation: ValidationResult::new(),
            save_dialog: None,
        }
    }
}

impl From<&Item> for AccountingItem {
    fn from(val: &Item) -> Self {
        AccountingItem {
            invoice_type: val.invoice_type,
            id: val.id,
            date: NaiveDate::parse_from_str(&val.date_field, DATE_FORMAT).expect("was validated"),
            name: val.name.to_owned(),
            company: Company(val.company.to_owned()),
            category: Category(val.category.to_owned()),
            net: CurrencyValue::new_from_decimal(
                Decimal::from_str(&val.net).expect("is a valid number"),
            ),
            vat: val.vat,
            file: val.file.to_owned(),
        }
    }
}

impl Item {
    fn new() -> Self {
        let now = chrono::Local::now().date_naive();
        Self {
            id: Uuid::now_v7(),
            show: true,
            focus_first_element: true,
            invoice_type: InvoiceType::In,
            date: now,
            date_field: now.format(DATE_FORMAT).to_string(),
            name: String::default(),
            name_autosuggest: AutoSuggest::new(),
            company: String::default(),
            company_autosuggest: AutoSuggest::new(),
            category: String::default(),
            category_autosuggest: AutoSuggest::new(),
            net: String::from("0.00"),
            vat: Vat::Zero,
            file: PathBuf::default(),
            open_file_dialog: None,
            validation: ValidationResult::new(),
            save_dialog: None,
        }
    }

    fn hidden(mut self) -> Self {
        self.show = false;
        self
    }

    fn validate(&self, state: &AccountingState) -> ValidationResult {
        let mut validation_result = ValidationResult::new();
        if let Ok(date) = NaiveDate::parse_from_str(&self.date_field, DATE_FORMAT) {
            if !is_date_in_selected_time_span(
                date,
                state.selected_year,
                state.selected_quarter,
                state.selected_month,
            ) {
                validation_result.add_warning(
                    Field::Date,
                    Messages::DateNotInSelectedDateRange.msg().to_owned(),
                );
            }
        } else {
            validation_result.add_error(Field::Date, Messages::DateNotValid.msg().to_owned());
        }

        if self.name.is_empty() {
            validation_result.add_error(
                Field::Name,
                format!("{} {}", Messages::Name, Messages::CanNotBeEmpty),
            );
        }

        if self.company.is_empty() {
            validation_result.add_error(
                Field::Company,
                format!("{} {}", Messages::Company, Messages::CanNotBeEmpty),
            );
        }

        if self.category.is_empty() {
            validation_result.add_error(
                Field::Category,
                format!("{} {}", Messages::Category, Messages::CanNotBeEmpty),
            );
        }

        if let Err(_e) = Decimal::from_str(&self.net) {
            validation_result.add_error(
                Field::Net,
                format!("{} {}", Messages::Net, Messages::NotANumber),
            );
        }
        if self.file.as_os_str().is_empty() {
            validation_result.add_error(
                Field::File,
                format!("{} {}", Messages::File, Messages::CanNotBeEmpty),
            );
        }
        validation_result
    }
}

pub(crate) fn build(
    ctx: &Context,
    state: &mut State,
    config: &Config,
    app_context: &AppContext,
    ui: &mut Ui,
) {
    ui.label(RichText::new(Messages::Accounting).strong());
    ui.separator();
    ui.vertical(|ui| {
        Grid::new("date_selection_grid")
            .num_columns(3)
            .show(ui, |ui| {
                ui.label(Messages::Year);
                ComboBox::from_id_salt("year_selector")
                    .selected_text(format!("{}", state.accounting.year_selector_selected))
                    .show_ui(ui, |ui| {
                        ((state.accounting.year_selector_selected - 100)
                            ..=chrono::Local::now().year())
                            .rev()
                            .for_each(|year| {
                                if ui
                                    .add(SelectableLabel::new(
                                        state.accounting.year_selector_selected == year,
                                        format!("{}", year),
                                    ))
                                    .clicked()
                                {
                                    state.accounting.year_selector_selected = year;
                                    state.accounting.quarter_selector_selected = None;
                                    state.accounting.month_selector_selected = None;
                                }
                            });
                    });
                ui.end_row();

                ui.label(Messages::Quarter);
                ui.horizontal(|ui| {
                    QUARTERS.iter().for_each(|quarter| {
                        if ui
                            .add(SelectableLabel::new(
                                state.accounting.quarter_selector_selected
                                    == Some(quarter.to_owned()),
                                quarter.name(),
                            ))
                            .clicked()
                        {
                            state.accounting.quarter_selector_selected = Some(quarter.to_owned());
                            state.accounting.month_selector_selected = None;
                        }
                    });
                });
                ui.end_row();

                ui.label(Messages::Month);
                ui.horizontal(|ui| {
                    MONTHS.iter().for_each(|month| {
                        if ui
                            .add(SelectableLabel::new(
                                state.accounting.month_selector_selected == Some(month.to_owned()),
                                month.short(),
                            ))
                            .clicked()
                        {
                            state.accounting.month_selector_selected = Some(month.to_owned());
                            state.accounting.quarter_selector_selected = None;
                        }
                    });
                });
                if ui.button(Messages::Select).clicked() {
                    state.accounting.selected_year = state.accounting.year_selector_selected;
                    state.accounting.selected_month = state.accounting.month_selector_selected;
                    state.accounting.selected_quarter = state.accounting.quarter_selector_selected;
                    select_date_range(state, app_context, ctx);
                }
                ui.end_row();
            });
        ui.separator();
        ui.horizontal(|ui| {
            ui.label(format!(
                "{}: {}",
                Messages::Year,
                state.accounting.selected_year
            ));
            if let Some(quarter) = state.accounting.selected_quarter {
                ui.label(format!("{}: {}", Messages::Quarter, quarter.name()));
            }
            if let Some(month) = state.accounting.selected_month {
                ui.label(format!("{}: {}", Messages::Month, month.name()));
            }
        });

        add_button(ui, state);
        items_table::build(ctx, &mut state.accounting, app_context, ui);

        add_edit::build(ctx, state, config, app_context, ui);
        if ui.button(Messages::Export.msg()).clicked() {
            let name_suggestion = build_file_name_suggestion(&state.accounting);
            let mut dialog = ui::get_localized_save_file_dialog(
                state.file_picker_startpoint.clone(),
                Messages::SaveFile.msg(),
            )
            .default_filename(name_suggestion.unwrap_or_default());
            dialog.open();
            state.accounting.export_state.open_file_dialog = Some(dialog);
        }
        if let Some(dialog) = &mut state.accounting.export_state.open_file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    let path_buf;
                    match file.extension() {
                        None => {
                            path_buf = file.with_extension("pdf");
                        }
                        Some(ext) => {
                            if ext != "pdf" {
                                path_buf = file.with_extension("pdf");
                            } else {
                                path_buf = file.to_path_buf();
                            }
                        }
                    }
                    state.file_picker_startpoint = Some(path_buf.clone());
                    state.accounting.export_state.selected_path = Some(path_buf);
                }
            }
        }

        if let Some(ref path_buf) = state.accounting.export_state.selected_path {
            if let Some(ref accounting_sheet) = state.accounting.selected_accounting_sheet {
                create_pdf(path_buf, accounting_sheet, app_context);
                state.accounting.export_state.selected_path = None;
            }
        }
    });
}

fn create_pdf(path_buf: &Path, accounting_sheet: &AccountingSheet, app_context: &AppContext) {
    match create_accounting_pdf(path_buf, accounting_sheet) {
        Ok(CreatePDFResult { file, files_folder }) => {
            info!("created pdf!");
            let mut results = accounting_sheet
                .items
                .iter()
                .enumerate()
                .map(|(idx, item)| {
                    let invoce_number = idx + 1;

                    copy_file_and_rename(
                        &invoce_number.to_string(),
                        files_folder.as_path(),
                        &item.file,
                    )
                    .map(|_| ())
                });
            if results.any(|r| r.is_err()) {
                let error_count = results.filter(|x| x.is_err()).count();
                info!(
                    "Errors while copying invoices for PDF creation: {error_count} - rolling back pdf and files folder creation"
                );
                // rollback pdf and files folder creation
                delete_file_and_folder(file.as_path(), files_folder.as_path());

                util::send_gui_event(
                    &app_context.gui_event_sender,
                    GuiEvent::ShowErrorNotification(format!(
                        "{} {}",
                        error_count,
                        Messages::PDFFilesCopyFailed.msg(),
                    )),
                );
            } else {
                util::send_gui_event(
                    &app_context.gui_event_sender,
                    GuiEvent::ShowInfoNotification(String::from(Messages::PDFCreated.msg())),
                );
            }
        }
        Err(e) => {
            log::error!("PDF was not created: {}", e);
            util::send_gui_event(
                &app_context.gui_event_sender,
                GuiEvent::ShowErrorNotification(String::from(Messages::PDFNotCreated.msg())),
            );
        }
    }
}

fn select_date_range(state: &mut State, app_context: &AppContext, ctx: &Context) {
    let date_range = get_date_range_for_settings(
        state.accounting.selected_year,
        state.accounting.selected_quarter,
        state.accounting.selected_month,
    );

    state.accounting.selected_accounting_sheet = Some(AccountingSheet {
        year: state.accounting.selected_year,
        quarter: state.accounting.selected_quarter,
        month: state.accounting.selected_month,
        items: vec![],
    });

    util::send_event_and_request_repaint(
        ctx,
        &app_context.background_event_sender,
        Event::FetchItems(date_range),
    );
}

fn add_button(ui: &mut Ui, state: &mut State) {
    if ui.button(Messages::AddItem).clicked() {
        state.accounting.item.focus_first_element = true;
        if state.accounting.item.show && state.accounting.mode == Mode::Edit {
            state.accounting.item = Item::new();
        } else {
            state.accounting.item.show = !state.accounting.item.show;
        }
        state.accounting.mode = Mode::Add;
    }
}
