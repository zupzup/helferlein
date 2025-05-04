use super::InvoiceState;
use crate::messages::Messages;
use eframe::egui::{Align, Layout, Ui};
use egui_extras::{Column, TableBuilder};
use log::info;

const ROW_HEIGHT: f32 = 30.0;

pub(super) fn build(state: &mut InvoiceState, ui: &mut Ui) {
    let mut item_to_remove: Option<usize> = None;
    let table = TableBuilder::new(ui)
        .striped(true)
        .max_scroll_height(200.0)
        .min_scrolled_height(100.0)
        .auto_shrink(true)
        .cell_layout(Layout::left_to_right(Align::Center))
        .column(Column::exact(30.0))
        .column(Column::remainder().clip(true))
        .column(Column::exact(30.0))
        .column(Column::exact(80.0))
        .column(Column::exact(100.0))
        .column(Column::exact(30.0))
        .column(Column::auto())
        .column(Column::auto());

    table
        .header(ROW_HEIGHT, |mut header| {
            header.col(|ui| {
                ui.strong(Messages::Nr);
            });
            header.col(|ui| {
                ui.strong(Messages::Description);
            });
            header.col(|ui| {
                ui.strong(Messages::Unit);
            });
            header.col(|ui| {
                ui.strong(Messages::Amount);
            });
            header.col(|ui| {
                ui.strong(Messages::PricePerUnit);
            });
            header.col(|ui| {
                ui.strong(Messages::Vat);
            });
        })
        .body(|body| {
            body.rows(ROW_HEIGHT, state.items.len(), |mut row| {
                let row_index = row.index();
                let item = &state.items[row_index].clone();
                row.col(|ui| {
                    ui.label(&item.nr);
                });
                row.col(|ui| {
                    ui.label(&item.decription);
                });
                row.col(|ui| {
                    ui.label(item.unit.name());
                });
                row.col(|ui| {
                    ui.label(&item.amount);
                });
                row.col(|ui| {
                    ui.label(&item.price_per_unit);
                });
                row.col(|ui| {
                    ui.label(item.vat.name());
                });
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button(Messages::Edit.msg()).clicked() {
                            info!("edit clicked on {}", item.id);
                            state.item_to_add = item.clone();
                        }
                    });
                });
                row.col(|ui| {
                    ui.horizontal(|ui| {
                        if ui.button(Messages::Delete.msg()).clicked() {
                            info!("delete clicked on {}", &item.id);
                            item_to_remove = Some(row_index);
                        }
                    });
                });
            });
        });
    if let Some(index) = item_to_remove {
        state.items.remove(index);
    }
}
