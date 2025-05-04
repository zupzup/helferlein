use super::AccountingState;
use crate::{
    accounting::{Item, Mode},
    data::currency::VatCalculationResult,
    db::{get_date_range_for_settings, DB},
    messages::Messages,
    util, AppContext, Event, DATE_FORMAT,
};
use eframe::egui::{Align, Context, Layout, Ui};
use egui_extras::{Column, TableBuilder};
use log::info;

const ROW_HEIGHT: f32 = 30.0;

pub(super) fn build(
    ctx: &Context,
    state: &mut AccountingState,
    app_context: &AppContext,
    ui: &mut Ui,
) {
    if let Some(accounting_sheet) = &mut state.selected_accounting_sheet {
        let table = TableBuilder::new(ui)
            .striped(true)
            .max_scroll_height(200.0)
            .min_scrolled_height(100.0)
            .auto_shrink(true)
            .cell_layout(Layout::left_to_right(Align::Center))
            .column(Column::exact(60.0))
            .column(Column::exact(30.0))
            .column(Column::auto())
            .column(Column::remainder().clip(true))
            .column(Column::remainder().clip(true))
            .column(Column::remainder().clip(true))
            .column(Column::exact(80.0))
            .column(Column::exact(30.0))
            .column(Column::exact(80.0))
            .column(Column::exact(80.0))
            .column(Column::exact(25.0))
            .column(Column::auto())
            .column(Column::auto());

        table
            .header(ROW_HEIGHT, |mut header| {
                header.col(|ui| {
                    ui.strong(Messages::InvoiceType);
                });
                header.col(|ui| {
                    ui.strong(Messages::InvoiceNumber);
                });
                header.col(|ui| {
                    ui.strong(Messages::Date);
                });
                header.col(|ui| {
                    ui.strong(Messages::Name);
                });
                header.col(|ui| {
                    ui.strong(Messages::Company);
                });
                header.col(|ui| {
                    ui.strong(Messages::Category);
                });
                header.col(|ui| {
                    ui.strong(Messages::Net);
                });
                header.col(|ui| {
                    ui.strong(Messages::Vat);
                });
                header.col(|ui| {
                    ui.strong(Messages::Tax);
                });
                header.col(|ui| {
                    ui.strong(Messages::Gross);
                });
                header.col(|ui| {
                    ui.strong(Messages::File);
                });
                header.col(|ui| {
                    ui.strong(Messages::Edit);
                });
                header.col(|ui| {
                    ui.strong(Messages::Delete);
                });
            })
            .body(|body| {
                body.rows(ROW_HEIGHT, accounting_sheet.items.len(), |mut row| {
                    let row_index = row.index();
                    let invoice_number = row_index + 1;
                    let item = &accounting_sheet.items[row_index];
                    row.col(|ui| {
                        let text = item.invoice_type.name();
                        ui.label(text);
                    });
                    row.col(|ui| {
                        let text = invoice_number.to_string();
                        ui.label(&text);
                    });
                    row.col(|ui| {
                        let text = item.date.format(DATE_FORMAT).to_string();
                        ui.label(&text);
                    });
                    row.col(|ui| {
                        ui.label(&item.name);
                    });
                    row.col(|ui| {
                        ui.label(&item.company.0);
                    });
                    row.col(|ui| {
                        ui.label(&item.category.0);
                    });
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            let text = item.net.to_str();
                            ui.label(text);
                        });
                    });
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(item.vat);
                        });
                    });
                    let VatCalculationResult { tax, gross } = &item.net.calculate_vat(item.vat);
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(tax);
                        });
                    });
                    row.col(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            ui.label(gross);
                        });
                    });
                    row.col(|ui| {
                        let file = &item.file;
                        let text = file.to_str().unwrap_or_default();
                        if ui.link(Messages::Link).on_hover_text(text).clicked() {
                            info!("clicked link: {}", text);
                            util::send_event_and_request_repaint(
                                ctx,
                                &app_context.background_event_sender,
                                Event::OpenFile(text.to_owned()),
                            );
                        }
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            let text = item.id.to_string();
                            if ui.button(Messages::Edit.msg()).clicked() {
                                state.mode = Mode::Edit;
                                state.item = Item::from(item);
                                info!("edit pressed on {}", text)
                            }
                        });
                    });
                    row.col(|ui| {
                        ui.horizontal(|ui| {
                            if ui.button(Messages::Delete.msg()).clicked() {
                                util::send_event_and_request_repaint(
                                    ctx,
                                    &app_context.background_event_sender,
                                    Event::RemoveItem(
                                        DB::get_key_for_item(item),
                                        get_date_range_for_settings(
                                            state.selected_year,
                                            state.selected_quarter,
                                            state.selected_month,
                                        ),
                                    ),
                                );
                            }
                        });
                    });
                });
            });
    }
}
