use crate::{
    data::{currency::CurrencyValue, Address, Invoice, InvoiceItem, ServicePeriod, Unit, Vat},
    db::DB,
    messages::Messages,
    ui,
    util::{
        self,
        export::invoice::{create_invoice_pdf, CreatePDFResult, MAX_ITEMS},
        files::build_invoice_file_name,
        validation::{Field, ValidationResult},
    },
    AppContext, Colors, Event, GuiEvent, State, DATE_FORMAT,
};
use chrono::NaiveDate;
use eframe::egui::{Context, Grid, RichText, ScrollArea, SelectableLabel, TextEdit, Ui};
use egui_extras::{Size, StripBuilder};
use egui_extras_datepicker_fork::DatePickerButton;
use egui_file::FileDialog;
use rust_decimal::Decimal;
use std::{
    path::{Path, PathBuf},
    str::FromStr,
};
use uuid::Uuid;

mod items_table;

fn render_field_errors(field: &Field, validation_result: &ValidationResult, ui: &mut Ui) {
    if let Some(errors) = validation_result.get_errors(field) {
        errors.iter().for_each(|e| {
            ui.end_row();
            ui.label(""); // workaround because we can't span columns in a grid
            ui.colored_label(Colors::Error.col(), format!("❎ {}", e));
        });
    }
}

fn export_pdf(path_buf: &Path, app_context: &AppContext, invoice: &Invoice) {
    match create_invoice_pdf(path_buf, invoice) {
        Ok(CreatePDFResult { .. }) => {
            util::send_gui_event(
                &app_context.gui_event_sender,
                GuiEvent::ShowInfoNotification(String::from(Messages::PDFCreated.msg())),
            );
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

#[derive(Debug)]
pub(crate) struct InvoiceState {
    pub(crate) metadata: Metadata,
    items: Vec<Item>,
    item_to_add: Item,
    validation: ValidationResult,
    item_validation: ValidationResult,
    export_state: ExportState,
    pub(crate) templates: Vec<Invoice>,
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

impl InvoiceState {
    pub fn new() -> Self {
        let now = chrono::Local::now().date_naive();
        Self {
            metadata: Metadata {
                name: String::default(),
                from: Address::new(),
                to: Address::new(),
                date: now,
                date_field: now.format(DATE_FORMAT).to_string(),
                city: String::default(),
                invoice_number: String::default(),
                service_period: ServicePeriod {
                    from: now,
                    from_field: now.format(DATE_FORMAT).to_string(),
                    to: now,
                    to_field: now.format(DATE_FORMAT).to_string(),
                },
                pretext: String::default(),
                posttext: String::default(),
                bank_data: String::default(),
            },
            items: vec![],
            item_to_add: Item::default(),
            validation: ValidationResult::new(),
            item_validation: ValidationResult::new(),
            export_state: ExportState::new(),
            templates: vec![],
        }
    }

    pub fn validate(&self) -> ValidationResult {
        let mut validation_result = ValidationResult::new();
        if self.metadata.from.name.is_empty() {
            validation_result.add_error(
                Field::FromName,
                format!("{} {}", Messages::Name, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.from.postal_address.is_empty() {
            validation_result.add_error(
                Field::FromAddress,
                format!("{} {}", Messages::PostalAddress, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.from.zip.is_empty() {
            validation_result.add_error(
                Field::FromZip,
                format!("{} {}", Messages::Zip, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.from.city.is_empty() {
            validation_result.add_error(
                Field::FromCity,
                format!("{} {}", Messages::City, Messages::CanNotBeEmpty),
            );
        }

        if self.metadata.to.name.is_empty() {
            validation_result.add_error(
                Field::ToName,
                format!("{} {}", Messages::Name, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.to.postal_address.is_empty() {
            validation_result.add_error(
                Field::ToAddress,
                format!("{} {}", Messages::PostalAddress, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.to.zip.is_empty() {
            validation_result.add_error(
                Field::ToZip,
                format!("{} {}", Messages::Zip, Messages::CanNotBeEmpty),
            );
        }
        if self.metadata.to.city.is_empty() {
            validation_result.add_error(
                Field::ToCity,
                format!("{} {}", Messages::City, Messages::CanNotBeEmpty),
            );
        }

        if NaiveDate::parse_from_str(&self.metadata.date_field, DATE_FORMAT).is_err() {
            validation_result.add_error(Field::Date, Messages::DateNotValid.msg().to_owned());
        }

        if self.metadata.name.is_empty() {
            validation_result.add_error(
                Field::Name,
                format!("{} {}", Messages::Name, Messages::CanNotBeEmpty),
            );
        }

        if self.metadata.city.is_empty() {
            validation_result.add_error(
                Field::City,
                format!("{} {}", Messages::City, Messages::CanNotBeEmpty),
            );
        }

        if self.metadata.invoice_number.is_empty() {
            validation_result.add_error(
                Field::Nr,
                format!("{} {}", Messages::Nr, Messages::CanNotBeEmpty),
            );
        }

        if NaiveDate::parse_from_str(&self.metadata.service_period.from_field, DATE_FORMAT).is_err()
        {
            validation_result.add_error(
                Field::ServicePeriodFrom,
                Messages::DateNotValid.msg().to_owned(),
            );
        }

        if NaiveDate::parse_from_str(&self.metadata.service_period.to_field, DATE_FORMAT).is_err() {
            validation_result.add_error(
                Field::ServicePeriodTo,
                Messages::DateNotValid.msg().to_owned(),
            );
        }

        validation_result
    }
}

impl From<&InvoiceState> for Invoice {
    fn from(value: &InvoiceState) -> Self {
        Invoice {
            id: Uuid::now_v7(),
            date: value.metadata.date.to_owned(),
            city: value.metadata.city.to_owned(),
            name: value.metadata.name.to_owned(),
            from: value.metadata.from.to_owned(),
            to: value.metadata.to.to_owned(),
            service_period: value.metadata.service_period.to_owned(),
            invoice_number: value.metadata.invoice_number.to_owned(),
            pre_text: value.metadata.pretext.to_owned(),
            post_text: value.metadata.posttext.to_owned(),
            bank_data: value.metadata.bank_data.to_owned(),
            items: value
                .items
                .iter()
                .cloned()
                .map(|i| InvoiceItem {
                    nr: i.nr.parse::<u64>().expect("is a valid number"),
                    description: i.decription,
                    unit: i.unit,
                    amount: Decimal::from_str(&i.amount).expect("is a valid number"),
                    price_per_unit: CurrencyValue::new_from_decimal(
                        Decimal::from_str(&i.price_per_unit).expect("is a valid number"),
                    ),
                    vat: i.vat,
                })
                .collect(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Metadata {
    pub(crate) name: String,
    from: Address,
    to: Address,
    date: NaiveDate,
    date_field: String,
    city: String,
    invoice_number: String,
    service_period: ServicePeriod,
    pretext: String,
    posttext: String,
    bank_data: String,
}

#[derive(Debug, Clone)]
pub(crate) struct Item {
    id: Uuid,
    nr: String,
    decription: String,
    unit: Unit,
    amount: String,
    price_per_unit: String,
    vat: Vat,
}

impl Default for Item {
    fn default() -> Self {
        Self {
            id: Uuid::now_v7(),
            nr: Default::default(),
            decription: Default::default(),
            unit: Unit::Hour,
            amount: Default::default(),
            price_per_unit: Default::default(),
            vat: Vat::Twenty,
        }
    }
}

impl Item {
    pub fn validate(&self) -> ValidationResult {
        let mut validation_result = ValidationResult::new();

        if self.nr.parse::<u64>().is_err() {
            validation_result.add_error(
                Field::Nr,
                format!("{} {}", Messages::Nr, Messages::NotANumber),
            );
        }

        if self.decription.is_empty() {
            validation_result.add_error(
                Field::Description,
                format!("{} {}", Messages::Description, Messages::CanNotBeEmpty),
            );
        }
        if Decimal::from_str(&self.amount).is_err() {
            validation_result.add_error(
                Field::Amount,
                format!("{} {}", Messages::Amount, Messages::NotANumber),
            );
        }

        if Decimal::from_str(&self.price_per_unit).is_err() {
            validation_result.add_error(
                Field::PricePerUnit,
                format!("{} {}", Messages::PricePerUnit, Messages::NotANumber),
            );
        }

        validation_result
    }
}

pub(crate) fn build(ctx: &Context, state: &mut State, app_context: &AppContext, ui: &mut Ui) {
    ui.label(RichText::new(Messages::Invoice).strong());
    ui.separator();
    StripBuilder::new(ui)
        .size(Size::relative(0.7))
        .size(Size::remainder())
        .horizontal(|mut strip| {
            strip.cell(|ui| {
                ui.label(RichText::new(Messages::CreateNewInvoice).strong());
                ui.separator();
                Grid::new("invoice_add_grid_from_to")
                    .num_columns(2)
                    .show(ui, |ui| {
                        Grid::new("invoice_add_grid_from")
                            .num_columns(2)
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(Messages::From).strong());
                                ui.end_row();
                                ui.label(Messages::Name);
                                ui.text_edit_singleline(&mut state.invoice.metadata.from.name);
                                render_field_errors(
                                    &Field::FromName,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::PostalAddress);
                                ui.text_edit_singleline(
                                    &mut state.invoice.metadata.from.postal_address,
                                );
                                render_field_errors(
                                    &Field::FromAddress,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::Zip);
                                ui.text_edit_singleline(&mut state.invoice.metadata.from.zip);
                                render_field_errors(&Field::FromZip, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::City);
                                ui.text_edit_singleline(&mut state.invoice.metadata.from.city);
                                render_field_errors(
                                    &Field::FromCity,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::Country);
                                ui.text_edit_singleline(&mut state.invoice.metadata.from.country);
                                render_field_errors(
                                    &Field::FromCountry,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::VatNr);
                                ui.text_edit_singleline(&mut state.invoice.metadata.from.vat);
                                render_field_errors(&Field::FromVat, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::Misc);
                                ui.text_edit_multiline(&mut state.invoice.metadata.from.misc);
                                render_field_errors(
                                    &Field::FromMisc,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                            });
                        Grid::new("invoice_add_grid_to")
                            .num_columns(2)
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(Messages::To).strong());
                                ui.end_row();
                                ui.label(Messages::Name);
                                ui.text_edit_singleline(&mut state.invoice.metadata.to.name);
                                render_field_errors(&Field::ToName, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::PostalAddress);
                                ui.text_edit_singleline(
                                    &mut state.invoice.metadata.to.postal_address,
                                );
                                render_field_errors(
                                    &Field::ToAddress,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::Zip);
                                ui.text_edit_singleline(&mut state.invoice.metadata.to.zip);
                                render_field_errors(&Field::ToZip, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::City);
                                ui.text_edit_singleline(&mut state.invoice.metadata.to.city);
                                render_field_errors(&Field::ToCity, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::Country);
                                ui.text_edit_singleline(&mut state.invoice.metadata.to.country);
                                render_field_errors(
                                    &Field::ToCountry,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::VatNr);
                                ui.text_edit_singleline(&mut state.invoice.metadata.to.vat);
                                render_field_errors(&Field::ToVat, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::Misc);
                                ui.text_edit_multiline(&mut state.invoice.metadata.to.misc);
                                render_field_errors(&Field::ToMisc, &state.invoice.validation, ui);
                                ui.end_row();
                            });
                    });
                ui.separator();
                Grid::new("invoice_add_grid_pre_items_service_period")
                    .num_columns(2)
                    .show(ui, |ui| {
                        Grid::new("invoice_add_grid_pre_items")
                            .num_columns(2)
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(Messages::General).strong());
                                ui.end_row();
                                ui.label(Messages::Name);
                                ui.text_edit_singleline(&mut state.invoice.metadata.name);
                                render_field_errors(&Field::Name, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::Date);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(
                                            &mut state.invoice.metadata.date_field,
                                        )
                                        .desired_width(65.0),
                                    );
                                    let date_response = ui.add(
                                        DatePickerButton::new(&mut state.invoice.metadata.date)
                                            .id_salt("metadata_date")
                                            .calendar_week(false)
                                            .save_button_text(Messages::Save.msg())
                                            .cancel_button_text(Messages::Cancel.msg())
                                            .show_icon(true)
                                            .day_names(Messages::days())
                                            .month_names(Messages::months())
                                            .highlight_weekends(false),
                                    );
                                    if date_response.changed() {
                                        state.invoice.metadata.date_field = state
                                            .invoice
                                            .metadata
                                            .date
                                            .format(DATE_FORMAT)
                                            .to_string();
                                        state.invoice.validation.clear_for_field(&Field::Date);
                                    }
                                });
                                render_field_errors(&Field::Date, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::City);
                                ui.text_edit_singleline(&mut state.invoice.metadata.city);
                                render_field_errors(&Field::City, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.label(Messages::Nr);
                                ui.text_edit_singleline(&mut state.invoice.metadata.invoice_number);
                                render_field_errors(&Field::Nr, &state.invoice.validation, ui);
                                ui.end_row();
                                ui.end_row();
                                ui.label(RichText::new(Messages::Misc).strong());
                                ui.end_row();
                                ui.label(Messages::PreText);
                                ui.text_edit_multiline(&mut state.invoice.metadata.pretext);
                                ui.end_row();
                                ui.label(Messages::PostText);
                                ui.text_edit_multiline(&mut state.invoice.metadata.posttext);
                                ui.end_row();
                                ui.label(Messages::BankData);
                                ui.text_edit_multiline(&mut state.invoice.metadata.bank_data);
                                ui.end_row();
                            });
                        Grid::new("invoice_add_grid_service_period")
                            .num_columns(2)
                            .min_col_width(70.0)
                            .show(ui, |ui| {
                                ui.label(RichText::new(Messages::ServicePeriod).strong());
                                ui.end_row();
                                ui.label(Messages::From);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(
                                            &mut state.invoice.metadata.service_period.from_field,
                                        )
                                        .desired_width(65.0),
                                    );
                                    let date_response_from = ui.add(
                                        DatePickerButton::new(
                                            &mut state.invoice.metadata.service_period.from,
                                        )
                                        .id_salt("metadata_sp_from")
                                        .calendar_week(false)
                                        .save_button_text(Messages::Save.msg())
                                        .cancel_button_text(Messages::Cancel.msg())
                                        .show_icon(true)
                                        .day_names(Messages::days())
                                        .month_names(Messages::months())
                                        .highlight_weekends(false),
                                    );
                                    if date_response_from.changed() {
                                        state.invoice.metadata.service_period.from_field = state
                                            .invoice
                                            .metadata
                                            .service_period
                                            .from
                                            .format(DATE_FORMAT)
                                            .to_string();
                                        state
                                            .invoice
                                            .validation
                                            .clear_for_field(&Field::ServicePeriodFrom);
                                    }
                                });
                                render_field_errors(
                                    &Field::ServicePeriodFrom,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::To);
                                ui.horizontal(|ui| {
                                    ui.add(
                                        TextEdit::singleline(
                                            &mut state.invoice.metadata.service_period.to_field,
                                        )
                                        .desired_width(65.0),
                                    );
                                    let date_response_to = ui.add(
                                        DatePickerButton::new(
                                            &mut state.invoice.metadata.service_period.to,
                                        )
                                        .id_salt("metadata_sp_to")
                                        .calendar_week(false)
                                        .save_button_text(Messages::Save.msg())
                                        .cancel_button_text(Messages::Cancel.msg())
                                        .show_icon(true)
                                        .day_names(Messages::days())
                                        .month_names(Messages::months())
                                        .highlight_weekends(false),
                                    );
                                    if date_response_to.changed() {
                                        state.invoice.metadata.service_period.to_field = state
                                            .invoice
                                            .metadata
                                            .service_period
                                            .to
                                            .format(DATE_FORMAT)
                                            .to_string();
                                        state
                                            .invoice
                                            .validation
                                            .clear_for_field(&Field::ServicePeriodTo);
                                    }
                                });
                                render_field_errors(
                                    &Field::ServicePeriodTo,
                                    &state.invoice.validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.end_row();
                                ui.label(RichText::new(Messages::NewItem).strong());
                                ui.end_row();
                                ui.label(Messages::Nr);
                                if ui
                                    .text_edit_singleline(&mut state.invoice.item_to_add.nr)
                                    .changed()
                                {
                                    state.invoice.validation.clear_for_field(&Field::Nr);
                                }
                                render_field_errors(&Field::Nr, &state.invoice.item_validation, ui);
                                ui.end_row();
                                ui.label(Messages::Description);
                                if ui
                                    .text_edit_multiline(&mut state.invoice.item_to_add.decription)
                                    .changed()
                                {
                                    state
                                        .invoice
                                        .validation
                                        .clear_for_field(&Field::Description);
                                }
                                render_field_errors(
                                    &Field::Description,
                                    &state.invoice.item_validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::Unit);
                                ui.horizontal(|ui| {
                                    [Unit::Hour, Unit::Day, Unit::None].iter().for_each(|unit| {
                                        if ui
                                            .add(SelectableLabel::new(
                                                state.invoice.item_to_add.unit == *unit,
                                                unit.name(),
                                            ))
                                            .clicked()
                                        {
                                            state.invoice.item_to_add.unit = *unit;
                                        }
                                    });
                                });
                                ui.end_row();
                                ui.label(Messages::Amount);
                                if ui
                                    .text_edit_singleline(&mut state.invoice.item_to_add.amount)
                                    .changed()
                                {
                                    state.invoice.validation.clear_for_field(&Field::Amount);
                                }
                                render_field_errors(
                                    &Field::Amount,
                                    &state.invoice.item_validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::PricePerUnit);
                                if ui
                                    .text_edit_singleline(
                                        &mut state.invoice.item_to_add.price_per_unit,
                                    )
                                    .changed()
                                {
                                    state
                                        .invoice
                                        .validation
                                        .clear_for_field(&Field::PricePerUnit);
                                }
                                render_field_errors(
                                    &Field::PricePerUnit,
                                    &state.invoice.item_validation,
                                    ui,
                                );
                                ui.end_row();
                                ui.label(Messages::Vat);
                                ui.horizontal(|ui| {
                                    [Vat::Zero, Vat::Ten, Vat::Twenty].iter().for_each(|vat| {
                                        if ui
                                            .add(SelectableLabel::new(
                                                state.invoice.item_to_add.vat == *vat,
                                                vat.name(),
                                            ))
                                            .clicked()
                                        {
                                            state.invoice.item_to_add.vat = *vat;
                                        }
                                    });
                                });
                                ui.end_row();
                                if ui.button(Messages::Save).clicked() {
                                    state.invoice.item_validation =
                                        state.invoice.item_to_add.validate();
                                    if state.invoice.item_validation.is_ok() {
                                        match state
                                            .invoice
                                            .items
                                            .iter_mut()
                                            .find(|i| i.id == state.invoice.item_to_add.id)
                                        {
                                            Some(item) => {
                                                *item = state.invoice.item_to_add.clone();
                                            }
                                            None => state
                                                .invoice
                                                .items
                                                .push(state.invoice.item_to_add.clone()),
                                        }
                                        state.invoice.item_to_add = Item::default();
                                    }
                                }
                            });
                    });
                ui.separator();
                ui.label(Messages::Items);
                items_table::build(&mut state.invoice, ui);
                ui.separator();
                ui.horizontal(|ui| {
                    if ui.button(Messages::Export).clicked() {
                        state.invoice.validation = state.invoice.validate();
                        if state.invoice.items.len() > MAX_ITEMS {
                            util::send_gui_event(
                                &app_context.gui_event_sender,
                                GuiEvent::ShowErrorNotification(format!(
                                    "{} {}/{}",
                                    Messages::TooManyItemsForPDFExport.msg(),
                                    state.invoice.items.len(),
                                    MAX_ITEMS
                                )),
                            );
                        } else if state.invoice.validation.is_ok() {
                            let mut dialog = ui::get_localized_save_file_dialog(
                                state.file_picker_startpoint.clone(),
                                Messages::SaveFile.msg(),
                            )
                            .default_filename(build_invoice_file_name(&state.invoice));
                            dialog.open();
                            state.invoice.export_state.open_file_dialog = Some(dialog);
                        }
                    }
                    if let Some(dialog) = &mut state.invoice.export_state.open_file_dialog {
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
                                state.invoice.export_state.selected_path = Some(path_buf);
                            }
                        }
                        if let Some(ref path_buf) = state.invoice.export_state.selected_path {
                            let invoice: Invoice = Invoice::from(&state.invoice);
                            export_pdf(path_buf, app_context, &invoice);
                            state.invoice.export_state.selected_path = None;
                        }
                    }
                    if ui.button(Messages::SaveAsTemplate).clicked() {
                        state.invoice.validation = state.invoice.validate();
                        if state.invoice.validation.is_ok() {
                            let invoice: Invoice = Invoice::from(&state.invoice);
                            util::send_event_and_request_repaint(
                                ctx,
                                &app_context.background_event_sender,
                                Event::SaveInvoiceTemplate(Box::new(invoice)),
                            )
                        }
                    }
                });
            });
            strip.cell(|ui| {
                ui.label(Messages::Templates);
                ui.separator();
                ScrollArea::vertical()
                    .max_height(200.0)
                    .auto_shrink(false)
                    .show(ui, |ui| {
                        Grid::new("invoice_templates")
                            .num_columns(3)
                            .show(ui, |ui| {
                                state.invoice.templates.iter().for_each(|t| {
                                    ui.label(t.name.chars().take(25).collect::<String>());
                                    ui.label(t.date.format(DATE_FORMAT).to_string());
                                    ui.horizontal(|ui| {
                                        if ui.button(Messages::Fill.msg()).clicked() {
                                            state.invoice.metadata = Metadata {
                                                name: t.name.clone(),
                                                from: t.from.clone(),
                                                to: t.to.clone(),
                                                date: t.date,
                                                date_field: t.date.format(DATE_FORMAT).to_string(),
                                                city: t.city.clone(),
                                                invoice_number: t.invoice_number.clone(),
                                                service_period: t.service_period.clone(),
                                                pretext: t.pre_text.clone(),
                                                posttext: t.post_text.clone(),
                                                bank_data: t.bank_data.clone(),
                                            };
                                            state.invoice.items = t
                                                .items
                                                .iter()
                                                .map(|i| Item {
                                                    id: Uuid::now_v7(),
                                                    nr: i.nr.to_string(),
                                                    decription: i.description.clone(),
                                                    unit: i.unit,
                                                    amount: i.amount.to_string(),
                                                    price_per_unit: i
                                                        .price_per_unit
                                                        .to_value_string(),
                                                    vat: i.vat,
                                                })
                                                .collect();
                                            util::send_gui_event(
                                                &app_context.gui_event_sender,
                                                GuiEvent::ShowInfoNotification(String::from(
                                                    Messages::InvoiceTemplateFilled.msg(),
                                                )),
                                            );
                                        }
                                        if ui.button(Messages::Delete.msg()).clicked() {
                                            util::send_event_and_request_repaint(
                                                ctx,
                                                &app_context.background_event_sender,
                                                Event::RemoveInvoiceTemplate(
                                                    DB::get_key_for_invoice(t),
                                                ),
                                            );
                                        }
                                    });
                                    ui.end_row();
                                });
                            });
                    });
            });
        });
}
