use super::{AccountingState, Item, Mode};
use crate::config::Config;
use crate::data::currency::{CurrencyValue, VatCalculationResult};
use crate::data::{InvoiceType, Vat};
use crate::db::get_date_range_for_settings;
use crate::messages::Messages;
use crate::ui::dialog::{self, Dialog, DialogResponse};
use crate::util::files::{PATH_FOR_FILES, copy_file_and_rename};
use crate::util::validation::Field;
use crate::util::{self, Colors, VALID_FILETYPES};
use crate::{AppContext, DATE_FORMAT, Event, GuiEvent, State, ui};
use eframe::egui::{Align, Context, Grid, Id, RichText, SelectableLabel, TextEdit, Ui};
use egui_extras_datepicker_fork::DatePickerButton;
use log::info;
use rust_decimal::Decimal;
use std::path::Path;
use std::str::FromStr;

fn render_field_errors(field: &Field, state: &AccountingState, ui: &mut Ui) {
    if let Some(errors) = state.item.validation.get_errors(field) {
        errors.iter().for_each(|e| {
            ui.end_row();
            ui.label(""); // workaround because we can't span columns in a grid
            ui.colored_label(Colors::Error.col(), format!("❎ {}", e));
        });
    }
}

fn render_field_warnings(field: &Field, state: &AccountingState, ui: &mut Ui) {
    if let Some(warnings) = state.item.validation.get_warnings(field) {
        warnings.iter().for_each(|w| {
            ui.end_row();
            ui.label(""); // workaround because we can't span columns in a grid
            ui.colored_label(Colors::Warning.col(), format!("⚠ {}", w));
        });
    }
}

pub(super) fn build(
    ctx: &Context,
    state: &mut State,
    config: &Config,
    app_context: &AppContext,
    ui: &mut Ui,
) {
    let accounting_state = &mut state.accounting;
    if accounting_state.item.show {
        ui.separator();
        match accounting_state.mode {
            Mode::Add => {
                ui.label(RichText::new(Messages::AddItem).heading());
            }
            Mode::Edit => {
                ui.label(RichText::new(Messages::EditItem).heading());
            }
        }
        Grid::new("item_add_grid").num_columns(2).show(ui, |ui| {
            ui.label(Messages::InvoiceType);
            let mut first_invoice_type_id = None;
            ui.horizontal(|ui| {
                [InvoiceType::In, InvoiceType::Out]
                    .iter()
                    .for_each(|invoice_type| {
                        let resp = ui.add(SelectableLabel::new(
                            accounting_state.item.invoice_type == *invoice_type,
                            invoice_type.name(),
                        ));
                        if first_invoice_type_id.is_none() {
                            first_invoice_type_id = Some(resp.id);
                        }

                        if resp.clicked() {
                            accounting_state.item.invoice_type = *invoice_type;
                        }
                    });
            });
            ui.end_row();
            if accounting_state.item.focus_first_element {
                accounting_state.item.focus_first_element = false;
                if let Some(first_resp_id) = first_invoice_type_id {
                    ui.memory_mut(|m| m.request_focus(first_resp_id));
                }
            }

            ui.label(Messages::Date);
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut accounting_state.item.date_field);
                let date_response = ui.add(
                    DatePickerButton::new(&mut accounting_state.item.date)
                        .calendar_week(false)
                        .save_button_text(Messages::Save.msg())
                        .cancel_button_text(Messages::Cancel.msg())
                        .show_icon(true)
                        .day_names(Messages::days())
                        .month_names(Messages::months())
                        .highlight_weekends(false),
                );
                if date_response.changed() {
                    accounting_state.item.date_field =
                        accounting_state.item.date.format(DATE_FORMAT).to_string();
                    accounting_state
                        .item
                        .validation
                        .clear_for_field(&Field::Date);
                }
            });
            render_field_warnings(&Field::Date, accounting_state, ui);
            render_field_errors(&Field::Date, accounting_state, ui);
            ui.end_row();

            ui.label(Messages::Name);
            let name_response = accounting_state.item.name_autosuggest.ui(
                ui,
                &mut accounting_state.item.name,
                &accounting_state.names,
            );

            if name_response.changed() {
                accounting_state
                    .item
                    .validation
                    .clear_for_field(&Field::Name);
            }
            render_field_warnings(&Field::Name, accounting_state, ui);
            render_field_errors(&Field::Name, accounting_state, ui);
            ui.end_row();

            ui.label(Messages::Company);
            let comp_response = accounting_state.item.company_autosuggest.ui(
                ui,
                &mut accounting_state.item.company,
                &accounting_state.companies,
            );

            if comp_response.changed() {
                accounting_state
                    .item
                    .validation
                    .clear_for_field(&Field::Company);
            }
            render_field_warnings(&Field::Company, accounting_state, ui);
            render_field_errors(&Field::Company, accounting_state, ui);
            ui.end_row();

            ui.label(Messages::Category);
            let cat_response = accounting_state.item.category_autosuggest.ui(
                ui,
                &mut accounting_state.item.category,
                &accounting_state.categories,
            );
            if cat_response.changed() {
                accounting_state
                    .item
                    .validation
                    .clear_for_field(&Field::Category);
            }
            render_field_warnings(&Field::Category, accounting_state, ui);
            render_field_errors(&Field::Category, accounting_state, ui);
            ui.end_row();

            ui.label(Messages::Net);
            let net_id = Id::new("net field").with("fld");
            ui.horizontal(|ui| {
                if ui
                    .add(
                        TextEdit::singleline(&mut accounting_state.item.net)
                            .id(net_id)
                            .cursor_at_end(false)
                            .horizontal_align(Align::Max),
                    )
                    .changed()
                {
                    accounting_state
                        .item
                        .validation
                        .clear_for_field(&Field::Net);
                }
                ui.label("€");
            });
            render_field_warnings(&Field::Net, accounting_state, ui);
            render_field_errors(&Field::Net, accounting_state, ui);
            ui.end_row();

            ui.label(Messages::Vat);
            ui.horizontal(|ui| {
                [Vat::Zero, Vat::Ten, Vat::Twenty].iter().for_each(|vat| {
                    if ui
                        .add(SelectableLabel::new(
                            accounting_state.item.vat == *vat,
                            vat.name(),
                        ))
                        .clicked()
                    {
                        accounting_state.item.vat = *vat;
                    }
                });
            });
            ui.end_row();

            let (mut tax, mut gross) =
                if let Ok(net) = Decimal::from_str(&accounting_state.item.net) {
                    let VatCalculationResult { tax, gross } = CurrencyValue::new_from_decimal(net)
                        .calculate_vat(accounting_state.item.vat);
                    (tax.to_value_string(), gross.to_value_string())
                } else {
                    (String::from("0.00"), String::from("0.00"))
                };

            ui.label(Messages::Tax);
            ui.horizontal(|ui| {
                ui.add_enabled(
                    false,
                    TextEdit::singleline(&mut tax).horizontal_align(Align::Max),
                );
                ui.label("€");
            });
            ui.end_row();

            ui.label(Messages::Gross);
            ui.horizontal(|ui| {
                ui.add_enabled(
                    false,
                    TextEdit::singleline(&mut gross).horizontal_align(Align::Max),
                );
                ui.label("€");
            });
            ui.end_row();

            ui.label(Messages::File);
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut accounting_state.item.file.to_str().map_or("", |v| v));
                let file_button_response = ui.button(Messages::Open);
                if file_button_response.clicked() {
                    let filter = Box::new({
                        move |path: &Path| -> bool {
                            if let Some(ext) = path.extension() {
                                for t in VALID_FILETYPES.iter() {
                                    if *t == ext {
                                        return true;
                                    }
                                }
                            }
                            false
                        }
                    });
                    let mut dialog = ui::get_localized_open_file_dialog(
                        if accounting_state.item.file.as_os_str().is_empty() {
                            state.file_picker_startpoint.clone()
                        } else {
                            Some(accounting_state.item.file.to_owned())
                        },
                        Messages::ChooseFile.msg(),
                    )
                    .show_files_filter(filter);
                    dialog.open();
                    accounting_state.item.open_file_dialog = Some(dialog);
                }

                if let Some(dialog) = &mut accounting_state.item.open_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(file) = dialog.path() {
                            state.file_picker_startpoint = Some(file.to_path_buf());
                            accounting_state.item.file = file.to_path_buf();
                        }
                        accounting_state
                            .item
                            .validation
                            .clear_for_field(&Field::File);
                    }
                }
            });
            render_field_warnings(&Field::File, accounting_state, ui);
            render_field_errors(&Field::File, accounting_state, ui);
            ui.end_row();
        });

        ui.horizontal(|ui| {
            let reset_button_response = ui.button(Messages::Reset);
            if reset_button_response.clicked() {
                accounting_state.item = Item::new();
                accounting_state.mode = Mode::Add;
            }
            ui.separator();
            let save_button_response = ui.button(Messages::SaveItem);
            if save_button_response.clicked() {
                accounting_state.item.validation = accounting_state.item.validate(accounting_state);

                if accounting_state.item.validation.is_ok() {
                    save_item(accounting_state, app_context, ctx, config);
                    accounting_state.item = Item::new();
                } else if accounting_state.item.validation.has_warnings()
                    && !accounting_state.item.validation.has_errors()
                {
                    accounting_state.item.save_dialog = Some(Dialog::new(
                        format!(
                            "{} {}",
                            Messages::ThereAreWarnings.msg(),
                            Messages::ReallySave.msg()
                        ),
                        Messages::SaveItem.msg(),
                        Messages::Cancel.msg(),
                    ));
                }
            }
        });
        if let Some(ref dialog) = accounting_state.item.save_dialog {
            match dialog::render_dialog(ctx, dialog) {
                DialogResponse::Ok => {
                    save_item(accounting_state, app_context, ctx, config);
                    accounting_state.item.save_dialog = None;
                    accounting_state.item = Item::new();
                    info!("save item pressed")
                }
                DialogResponse::Cancel => {
                    accounting_state.item.save_dialog = None;
                    info!("canceled")
                }
                _ => (),
            }
        }
    }
}

fn save_item(
    accounting_state: &mut AccountingState,
    app_context: &AppContext,
    ctx: &Context,
    config: &Config,
) {
    let id = accounting_state.item.id;
    match copy_file_and_rename(
        &id.to_string(),
        config
            .data_folder
            .as_ref()
            .expect("data folder is set")
            .join(PATH_FOR_FILES)
            .as_path(),
        &accounting_state.item.file,
    ) {
        Ok(new_path) => {
            accounting_state.item.file = new_path;
            util::send_gui_event(
                &app_context.gui_event_sender,
                GuiEvent::ShowInfoNotification(String::from(Messages::FileCopied.msg())),
            );
            util::send_event_and_request_repaint(
                ctx,
                &app_context.background_event_sender,
                Event::SaveItem(
                    (&accounting_state.item).into(),
                    get_date_range_for_settings(
                        accounting_state.selected_year,
                        accounting_state.selected_quarter,
                        accounting_state.selected_month,
                    ),
                ),
            )
        }
        Err(e) => {
            util::send_gui_event(
                &app_context.gui_event_sender,
                GuiEvent::ShowErrorNotification(e.to_string()),
            );
        }
    }
}
