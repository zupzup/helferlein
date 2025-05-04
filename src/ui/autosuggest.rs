use eframe::egui::{
    Key, Modifiers, PopupCloseBehavior, Response, ScrollArea, TextBuffer, Ui,
    popup::popup_below_widget,
};
use fuzzy_matcher::FuzzyMatcher;
use fuzzy_matcher::skim::SkimMatcherV2;
use std::cmp::Reverse;

#[derive(Debug)]
pub(crate) struct AutoSuggest {
    selected_index: Option<usize>,
    focused_last_frame: bool,
}

#[derive(Debug)]
enum SelectionMove {
    Up,
    Down,
}

impl AutoSuggest {
    pub(crate) fn new() -> Self {
        Self {
            selected_index: None,
            focused_last_frame: false,
        }
    }

    pub(crate) fn ui(&mut self, ui: &mut Ui, input: &mut String, values: &[String]) -> Response {
        let data = filter(values, input.as_str());

        let mut tab_pressed = false;
        let mut enter_pressed = false;

        if self.focused_last_frame {
            ui.input_mut(|i| {
                if i.consume_key(Modifiers::default(), Key::Enter) {
                    enter_pressed = true;
                }

                if i.consume_key(Modifiers::default(), Key::ArrowDown) {
                    self.update_index(SelectionMove::Down, data.len());
                }

                if i.consume_key(Modifiers::default(), Key::ArrowUp) {
                    self.update_index(SelectionMove::Up, data.len());
                }

                if i.consume_key(Modifiers::default(), Key::Tab) {
                    tab_pressed = true;
                }
            });
        }

        let text_field = ui.text_edit_singleline(input);
        let field_id = ui.make_persistent_id(text_field.id);

        let popup_id = field_id.with("popup");

        if tab_pressed {
            if let Some(idx) = self.selected_index {
                let text = data[idx];
                input.replace_with(text);
                self.selected_index = None;
            }
            ui.memory_mut(|m| {
                m.surrender_focus(field_id);
                m.close_popup();
            });
        }

        popup_below_widget(
            ui,
            popup_id,
            &text_field,
            PopupCloseBehavior::IgnoreClicks,
            |ui| {
                ScrollArea::vertical().max_height(100.0).show(ui, |ui| {
                    for (row, text) in data.iter().enumerate() {
                        let mut selected = if let Some(idx) = self.selected_index {
                            idx == row
                        } else {
                            false
                        };
                        let resp = ui.toggle_value(&mut selected, text.to_string());
                        if resp.hovered() {
                            self.selected_index = Some(row);
                        }
                        if selected {
                            resp.scroll_to_me(None);
                        }
                    }
                });
            },
        );

        self.focused_last_frame = text_field.has_focus();
        if text_field.changed() {
            self.selected_index = None;
        }

        if let Some(idx) = self.selected_index {
            if idx >= data.len() {
                self.selected_index = None;
            }
        }

        if let (Some(idx), true) = (
            self.selected_index,
            self.focused_last_frame && (enter_pressed || tab_pressed)
                || !ui.memory(|m| m.is_popup_open(popup_id)),
        ) {
            let text = data[idx];
            input.replace_with(text);
            self.selected_index = None;
            ui.memory_mut(|m| {
                if m.is_popup_open(popup_id) {
                    m.close_popup();
                    if enter_pressed {
                        m.surrender_focus(field_id);
                    }
                }
            });
        }

        if text_field.has_focus() {
            ui.memory_mut(|m| m.open_popup(popup_id));
        } else {
            ui.memory_mut(|m| {
                if m.is_popup_open(popup_id) {
                    m.close_popup();
                }
            });
        }

        text_field
    }

    fn update_index(&mut self, selection_move: SelectionMove, results_len: usize) {
        if results_len == 0 {
            self.selected_index = None;
            return;
        }

        match self.selected_index {
            None => match selection_move {
                SelectionMove::Up => {
                    self.selected_index = Some(results_len - 1);
                }
                SelectionMove::Down => {
                    self.selected_index = Some(0);
                }
            },
            Some(idx) => match selection_move {
                SelectionMove::Up => {
                    if idx == 0 {
                        self.selected_index = None;
                    } else {
                        self.selected_index = Some(idx - 1);
                    }
                }
                SelectionMove::Down => {
                    if idx >= results_len - 1 {
                        self.selected_index = None;
                    } else {
                        self.selected_index = Some(idx + 1);
                    }
                }
            },
        };
    }
}

fn filter<'a>(data: &'a [String], input: &str) -> Vec<&'a String> {
    let matcher = SkimMatcherV2::default();
    let mut res = data
        .iter()
        .filter_map(|s| {
            let score = matcher.fuzzy_match(s, input);
            score.map(|score| (s, score))
        })
        .collect::<Vec<(&String, i64)>>();
    res.sort_by_key(|k| Reverse(k.1));
    res.into_iter().map(|(s, _)| s).collect()
}
