use crate::{State, util::Colors};
use chrono::{DateTime, Duration, Local};
use eframe::egui::{
    Align, Align2, Color32, Context, CursorIcon, Id, Label, Layout, RichText, Sense, Window,
};
use egui_extras::{Size, StripBuilder};

const TIMEOUT_MS: i64 = 5000;
const MAX_SHOW_TEXT_LEN: usize = 100;

pub(crate) fn render_notifications(ctx: &Context, state: &mut State) {
    state
        .notifications
        .iter_mut()
        .enumerate()
        .for_each(|(i, notification)| {
            let now = chrono::Local::now();
            match notification {
                Notification::Error(inner) => {
                    if is_within_timeout(&inner.ts, &now) {
                        if render_notification(ctx, i, &inner.text, "❎", Colors::Error.col())
                            == HiddenState::Hide
                        {
                            inner.hidden = true;
                        }
                    } else {
                        inner.hidden = true
                    }
                }
                Notification::Info(inner) => {
                    if is_within_timeout(&inner.ts, &now) {
                        if render_notification(ctx, i, &inner.text, "ℹ", Colors::Info.col())
                            == HiddenState::Hide
                        {
                            inner.hidden = true;
                        };
                    } else {
                        inner.hidden = true
                    }
                }
            };
        });

    state.notifications = state
        .notifications
        .clone()
        .into_iter()
        .filter(|n| match n {
            Notification::Info(inner) | Notification::Error(inner) => !inner.hidden,
        })
        .collect();
}

fn is_within_timeout(ts: &DateTime<Local>, now: &DateTime<Local>) -> bool {
    let to = *ts + Duration::milliseconds(TIMEOUT_MS);
    to.ge(now)
}

#[derive(Debug, PartialEq)]
enum HiddenState {
    Hide,
    Show,
}

fn render_notification(
    ctx: &Context,
    idx: usize,
    text: &str,
    icon: &str,
    color: Color32,
) -> HiddenState {
    let mut hidden = HiddenState::Show;
    let window_height = 50.0;
    let offset_top: f32 = idx as f32 * window_height + (10.0 + idx as f32 * 20.0);
    Window::new(idx.to_string())
        .movable(false)
        .resizable(false)
        .collapsible(false)
        .title_bar(false)
        .fade_in(false)
        .fade_out(false)
        .anchor(Align2::RIGHT_TOP, [-10.0, offset_top])
        .drag_to_scroll(false)
        .fixed_size([200.0, window_height])
        .show(ctx, |ui| {
            if ui
                .interact(
                    ui.max_rect(),
                    Id::new(format!("window{idx}clicked")),
                    Sense::click(),
                )
                .clicked()
            {
                hidden = HiddenState::Hide
            }

            ui.set_width(ui.available_width());
            ui.set_height(ui.available_height());

            StripBuilder::new(ui)
                .size(Size::exact(20.0))
                .size(Size::remainder())
                .size(Size::exact(15.0))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                            if ui
                                .label(RichText::new(icon).color(color).size(15.0))
                                .on_hover_and_drag_cursor(CursorIcon::Default)
                                .clicked()
                            {
                                hidden = HiddenState::Hide
                            };
                        });
                    });
                    strip.cell(|ui| {
                        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                            let mut text_to_show = text.to_owned();
                            if text_to_show.len() > MAX_SHOW_TEXT_LEN {
                                text_to_show.truncate(MAX_SHOW_TEXT_LEN);
                                text_to_show.push_str("...");
                            }
                            if ui
                                .add(Label::new(text_to_show).wrap())
                                .on_hover_text(text)
                                .clicked()
                            {
                                hidden = HiddenState::Hide
                            };
                        });
                    });
                    strip.cell(|ui| {
                        if ui
                            .label("✖")
                            .on_hover_and_drag_cursor(CursorIcon::Default)
                            .clicked()
                        {
                            hidden = HiddenState::Hide
                        }
                    });
                });
        });
    hidden
}

#[derive(Debug, Clone)]
pub(crate) enum Notification {
    Error(InnerNotification),
    Info(InnerNotification),
}

#[derive(Debug, Clone)]
pub(crate) struct InnerNotification {
    ts: DateTime<Local>,
    text: String,
    hidden: bool,
}

impl InnerNotification {
    pub(crate) fn new(text: String) -> Self {
        Self {
            ts: Local::now(),
            hidden: false,
            text,
        }
    }
}
