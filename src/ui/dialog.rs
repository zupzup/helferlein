use eframe::egui::{Align, Align2, Context, Layout, Window};
use egui_extras::{Size, StripBuilder};

#[derive(Debug, Clone)]
pub(crate) struct Dialog {
    text: String,
    ok_text: &'static str,
    cancel_text: &'static str,
}

impl Dialog {
    pub(crate) fn new(text: String, ok_text: &'static str, cancel_text: &'static str) -> Self {
        Self {
            text,
            ok_text,
            cancel_text,
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum DialogResponse {
    Cancel,
    Ok,
    None,
}

pub(crate) fn render_dialog(ctx: &Context, dialog: &Dialog) -> DialogResponse {
    let mut result = DialogResponse::None;
    Window::new("dialog")
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .fade_in(false)
        .fade_out(false)
        .anchor(Align2::CENTER_CENTER, [0.0, 0.0])
        .drag_to_scroll(false)
        .fixed_size([400.0, 100.0])
        .show(ctx, |ui| {
            StripBuilder::new(ui)
                .size(Size::remainder())
                .size(Size::remainder())
                .size(Size::remainder())
                .size(Size::remainder())
                .vertical(|mut strip| {
                    strip.empty();
                    strip.cell(|ui| {
                        ui.vertical_centered(|ui| {
                            ui.label(&dialog.text);
                        });
                    });
                    strip.strip(|builder| {
                        builder
                            .size(Size::relative(0.5))
                            .size(Size::relative(0.5))
                            .horizontal(|mut strip| {
                                strip.cell(|ui| {
                                    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                                        if ui.button(dialog.cancel_text).clicked() {
                                            result = DialogResponse::Cancel;
                                        }
                                    });
                                });

                                strip.cell(|ui| {
                                    ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                                        if ui.button(dialog.ok_text).clicked() {
                                            result = DialogResponse::Ok;
                                        }
                                    });
                                });
                            });
                    });
                    strip.empty();
                });
        });
    result
}
