use anyhow::{anyhow, Result};
use config::Config;
use data::{AccountingItem, Invoice};
use db::{DateRange, DB};
use eframe::{
    egui::{
        self, Align2, Color32, Grid, RichText, ScrollArea, SelectableLabel, Shadow, TextEdit,
        Window,
    },
    App,
};
use egui_extras::{Size, StripBuilder};
use egui_file::FileDialog;
use log::{error, info};
use messages::{Language, Messages};
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::{
    path::PathBuf,
    sync::mpsc::{channel, Receiver, Sender},
};
use ui::{
    dialog::{self, Dialog, DialogResponse},
    notification::{self, InnerNotification, Notification},
};
use util::Colors;

mod accounting;
mod config;
mod data;
mod db;
mod invoice;
mod messages;
mod ui;
mod util;

static LANGUAGE: Lazy<Mutex<Language>> = Lazy::new(|| Mutex::new(Language::EN));

fn update_language(new_val: &str) {
    let mut config = LANGUAGE.lock().expect("failed to get LANGUAGE lock");
    *config = Language::from(new_val);
}

fn get_language() -> Language {
    let config = LANGUAGE.lock().expect("failed to get LANGUAGE lock");
    *config
}

const DATE_FORMAT: &str = "%d.%m.%Y";

fn main() -> Result<(), anyhow::Error> {
    env_logger::init();

    let (background_event_sender, background_event_receiver) = channel::<Event>();
    let (gui_event_sender, gui_event_receiver) = channel::<GuiEvent>();
    let config = config::load_config()?;

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_app_id(Messages::Title)
            .with_always_on_top()
            .with_inner_size([1024.0, 1024.0]),
        ..Default::default()
    };

    info!("Starting background thread...");
    let gui_event_sender_clone = gui_event_sender.clone();
    std::thread::spawn(move || {
        let mut db: Option<DB> = None;
        while let Ok(event) = background_event_receiver.recv() {
            if let Event::SetDB(ref data_folder) = event {
                if db.is_none() {
                    db = Some(DB::new(data_folder.as_path()));
                    if let Some(ref db) = db {
                        handle_background_events(
                            Event::FetchInvoiceTemplates(),
                            gui_event_sender_clone.clone(),
                            db,
                        );
                        handle_background_events(
                            Event::FetchNames(),
                            gui_event_sender_clone.clone(),
                            db,
                        );
                        handle_background_events(
                            Event::FetchCategories(),
                            gui_event_sender_clone.clone(),
                            db,
                        );
                        handle_background_events(
                            Event::FetchCompanies(),
                            gui_event_sender_clone.clone(),
                            db,
                        );
                    }
                }
            }
            if let Some(ref db) = db {
                handle_background_events(event, gui_event_sender_clone.clone(), db);
            }
        }
    });

    info!("Starting helferlein...");

    eframe::run_native(
        Messages::Title.into(),
        options,
        Box::new(|context| {
            context.egui_ctx.style_mut(|style| {
                // remove window shadow
                style.visuals.window_shadow = Shadow {
                    offset: [0, 0],
                    blur: 0,
                    spread: 0,
                    color: Color32::BLACK,
                };
            });
            Ok(Helferlein::new(
                background_event_sender,
                gui_event_receiver,
                gui_event_sender,
                config,
            ))
        }),
    )
    .map_err(|e| anyhow!("eframe error: {}", e))
}

fn handle_background_events(event: Event, sender: Sender<GuiEvent>, db: &db::DB) {
    match event {
        Event::OpenFile(file) => {
            if let Err(e) = open::with(&file, "firefox") {
                error!("Could not open file {file}: {e}");
                util::send_gui_event(
                    &sender,
                    GuiEvent::ShowErrorNotification(String::from(Messages::CouldNotOpenFile.msg())),
                );
            };
        }
        Event::SaveItem(item, date_range) => {
            match db.create_or_update_accounting_item_and_refetch(&item, &date_range) {
                Ok(items) => {
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowInfoNotification(String::from(Messages::ItemCreated.msg())),
                    );
                    util::send_gui_event(&sender, GuiEvent::SetAccountingItems(items));
                    handle_background_events(Event::FetchNames(), sender.clone(), db);
                    handle_background_events(Event::FetchCompanies(), sender.clone(), db);
                    handle_background_events(Event::FetchCategories(), sender.clone(), db);
                }
                Err(e) => {
                    error!(
                        "Could not create item with id {:?} and re-fetch items: {e}",
                        &item.id
                    );
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotCreateItem.msg(),
                        )),
                    );
                }
            };
        }
        Event::RemoveItem(item_id, date_range) => {
            match db.delete_accounting_item_and_refetch(&item_id, &date_range) {
                Ok(items) => {
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowInfoNotification(String::from(Messages::ItemDeleted.msg())),
                    );
                    util::send_gui_event(&sender, GuiEvent::SetAccountingItems(items));
                }
                Err(e) => {
                    error!("Could not delete item {item_id} and re-fetch items: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotDeleteItem.msg(),
                        )),
                    );
                }
            };
        }
        Event::FetchItems(date_range) => {
            match db.get_accounting_items_for_range(&date_range) {
                Ok(items) => {
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowInfoNotification(String::from(Messages::ItemsFetched.msg())),
                    );
                    util::send_gui_event(&sender, GuiEvent::SetAccountingItems(items));
                }
                Err(e) => {
                    error!("Could not fetch items: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotFetchData.msg(),
                        )),
                    );
                }
            };
        }
        Event::SetDB(_) => (),
        Event::RemoveInvoiceTemplate(invoice_id) => {
            match db.delete_invoice_template_and_refetch(&invoice_id) {
                Ok(items) => {
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowInfoNotification(String::from(Messages::ItemDeleted.msg())),
                    );
                    util::send_gui_event(&sender, GuiEvent::SetInvoiceTemplates(items));
                }
                Err(e) => {
                    error!(
                        "Could not delete invoice template {invoice_id} and re-fetch items: {e}"
                    );
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotDeleteItem.msg(),
                        )),
                    );
                }
            };
        }
        Event::SaveInvoiceTemplate(invoice) => {
            match db.create_invoice_template_and_refetch(&invoice) {
                Ok(items) => {
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowInfoNotification(String::from(
                            Messages::InvoiceTemplateCreated.msg(),
                        )),
                    );
                    util::send_gui_event(&sender, GuiEvent::SetInvoiceTemplates(items));
                }
                Err(e) => {
                    error!(
                        "Could not create invoice template with id {:?} and re-fetch items: {e}",
                        &invoice.id
                    );
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotCreateInvoiceTemplate.msg(),
                        )),
                    );
                }
            };
        }
        Event::FetchInvoiceTemplates() => {
            match db.get_invoice_templates() {
                Ok(items) => {
                    util::send_gui_event(&sender, GuiEvent::SetInvoiceTemplates(items));
                }
                Err(e) => {
                    error!("Could not fetch invoice templates: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotFetchNames.msg(),
                        )),
                    );
                }
            };
        }
        Event::FetchNames() => {
            match db.get_all_names() {
                Ok(items) => {
                    util::send_gui_event(&sender, GuiEvent::SetNames(items));
                }
                Err(e) => {
                    error!("Could not fetch names: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotFetchNames.msg(),
                        )),
                    );
                }
            };
        }
        Event::FetchCompanies() => {
            match db.get_all_companies() {
                Ok(items) => {
                    util::send_gui_event(&sender, GuiEvent::SetCompanies(items));
                }
                Err(e) => {
                    error!("Could not fetch companies: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotFetchCompanies.msg(),
                        )),
                    );
                }
            };
        }
        Event::FetchCategories() => {
            match db.get_all_categories() {
                Ok(items) => {
                    util::send_gui_event(&sender, GuiEvent::SetCategories(items));
                }
                Err(e) => {
                    error!("Could not fetch categories: {e}");
                    util::send_gui_event(
                        &sender,
                        GuiEvent::ShowErrorNotification(String::from(
                            Messages::CouldNotFetchCategories.msg(),
                        )),
                    );
                }
            };
        }
    }
}

#[derive(Debug)]
struct Helferlein {
    state: State,
    context: AppContext,
    config: Config,
}

#[derive(Debug)]
struct AppContext {
    background_event_sender: Sender<Event>,
    gui_event_receiver: Receiver<GuiEvent>,
    gui_event_sender: Sender<GuiEvent>,
    db_set: bool,
}

#[derive(Debug)]
struct State {
    navigation: NavigationState,
    accounting: accounting::AccountingState,
    invoice: invoice::InvoiceState,
    notifications: Vec<Notification>,
    config_state: ConfigState,
    file_picker_startpoint: Option<PathBuf>,
}

impl State {
    fn new() -> Self {
        Self {
            navigation: NavigationState::new(),
            accounting: accounting::AccountingState::new(),
            invoice: invoice::InvoiceState::new(),
            notifications: vec![],
            config_state: ConfigState::new(),
            file_picker_startpoint: None,
        }
    }
}

#[derive(Debug)]
struct ConfigState {
    open_file_dialog: Option<FileDialog>,
    selected_folder: Option<PathBuf>,
    change_data_folder_dialog: Option<Dialog>,
    file_open_command: String,
    file_open_command_change: bool,
    language: Language,
}

impl ConfigState {
    fn new() -> Self {
        Self {
            open_file_dialog: None,
            selected_folder: None,
            change_data_folder_dialog: None,
            file_open_command: String::default(),
            file_open_command_change: false,
            language: Language::EN,
        }
    }
}

#[derive(Debug)]
struct NavigationState {
    current_screen: Screen,
}

impl NavigationState {
    fn new() -> Self {
        Self {
            current_screen: Screen::Home,
        }
    }
}

impl Helferlein {
    fn new(
        background_event_sender: Sender<Event>,
        gui_event_receiver: Receiver<GuiEvent>,
        gui_event_sender: Sender<GuiEvent>,
        config: Config,
    ) -> Box<Self> {
        Box::new(Self {
            config,
            state: State::new(),
            context: AppContext {
                background_event_sender,
                gui_event_receiver,
                gui_event_sender,
                db_set: false,
            },
        })
    }

    fn handle_config_init(&mut self, ctx: &egui::Context) {
        match self.config.data_folder {
            None => {
                Window::new("config_missing")
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
                            .size(Size::remainder())
                            .size(Size::remainder())
                            .size(Size::remainder())
                            .vertical(|mut strip| {
                                strip.empty();
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.label(Messages::NoDataFolder.msg());
                                    });
                                });
                                strip.empty();
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.text_edit_singleline(
                                                &mut self
                                                    .state
                                                    .config_state
                                                    .selected_folder
                                                    .as_ref()
                                                    .map_or_else(
                                                        || "",
                                                        |path| path.to_str().unwrap_or(""),
                                                    ),
                                            );
                                            if (ui.button(Messages::Open)).clicked() {
                                                let mut dialog =
                                                    ui::get_localized_select_folder_dialog(
                                                        self.state
                                                            .config_state
                                                            .selected_folder
                                                            .clone(),
                                                        Messages::SelectFolder.msg(),
                                                    );
                                                dialog.open();
                                                self.state.config_state.open_file_dialog =
                                                    Some(dialog);
                                            }

                                            if let Some(dialog) =
                                                &mut self.state.config_state.open_file_dialog
                                            {
                                                if dialog.show(ctx).selected() {
                                                    if let Some(folder) = dialog.path() {
                                                        self.state.file_picker_startpoint =
                                                            Some(folder.to_path_buf());
                                                        self.state.config_state.selected_folder =
                                                            Some(folder.to_path_buf());
                                                    }
                                                }
                                            }
                                        });
                                    });
                                });
                                strip.empty();
                                strip.cell(|ui| {
                                    ui.vertical_centered(|ui| {
                                        if ui.button(Messages::Done.msg()).clicked() {
                                            if let Some(ref data_folder) =
                                                self.state.config_state.selected_folder
                                            {
                                                let cfg = Config {
                                                    data_folder: Some(data_folder.clone()),
                                                    file_open_command: self
                                                        .config
                                                        .file_open_command
                                                        .clone(),
                                                    language: self
                                                        .state
                                                        .config_state
                                                        .language
                                                        .name()
                                                        .into(),
                                                };
                                                if let Err(e) = config::save_config(&cfg) {
                                                    error!("Could not save config: {e}");
                                                } else {
                                                    self.config = cfg;
                                                }
                                            }
                                        }
                                    });
                                });
                                strip.empty();
                            });
                    });
            }
            Some(ref data_folder) => {
                if !self.context.db_set {
                    self.context.db_set = true;
                    util::send_event_and_request_repaint(
                        ctx,
                        &self.context.background_event_sender,
                        Event::SetDB(data_folder.clone()),
                    );
                }
            }
        }
    }

    fn handle_gui_events(&mut self) {
        while let Ok(event) = self.context.gui_event_receiver.try_recv() {
            match event {
                GuiEvent::SetInvoiceTemplates(items) => {
                    self.state.invoice.templates = items;
                }
                GuiEvent::ShowInfoNotification(text) => self
                    .state
                    .notifications
                    .push(Notification::Info(InnerNotification::new(text))),

                GuiEvent::ShowErrorNotification(text) => {
                    self.state
                        .notifications
                        .push(Notification::Error(InnerNotification::new(text)));
                }
                GuiEvent::SetAccountingItems(items) => {
                    if let Some(ref mut sheet) = self.state.accounting.selected_accounting_sheet {
                        sheet.items = items;
                    }
                }
                GuiEvent::SetNames(items) => {
                    self.state.accounting.names = items;
                }
                GuiEvent::SetCategories(items) => {
                    self.state.accounting.categories = items;
                }
                GuiEvent::SetCompanies(items) => {
                    self.state.accounting.companies = items;
                }
            }
        }
    }

    fn build_navigation(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let current_screen = self.state.navigation.current_screen;
            if ui
                .button(
                    RichText::new(Messages::Home).color(if current_screen == Screen::Home {
                        Colors::ButtonActive.col()
                    } else {
                        Colors::ButtonDefault.col()
                    }),
                )
                .clicked()
            {
                self.state.navigation.current_screen = Screen::Home;
            }
            if ui
                .button(RichText::new(Messages::Accounting).color(
                    if current_screen == Screen::Accounting {
                        Colors::ButtonActive.col()
                    } else {
                        Colors::ButtonDefault.col()
                    },
                ))
                .clicked()
            {
                self.state.navigation.current_screen = Screen::Accounting;
            }
            if ui
                .button(RichText::new(Messages::Invoice).color(
                    if current_screen == Screen::Invoice {
                        Colors::ButtonActive.col()
                    } else {
                        Colors::ButtonDefault.col()
                    },
                ))
                .clicked()
            {
                self.state.navigation.current_screen = Screen::Invoice;
            }
            if ui
                .button(RichText::new(Messages::Settings).color(
                    if current_screen == Screen::Settings {
                        Colors::ButtonActive.col()
                    } else {
                        Colors::ButtonDefault.col()
                    },
                ))
                .clicked()
            {
                self.state.navigation.current_screen = Screen::Settings;
            }
        });
    }

    fn build_home(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(Messages::Welcome).strong());
    }

    fn build_settings(&mut self, ui: &mut egui::Ui) {
        ui.label(RichText::new(Messages::Settings).strong());
        Grid::new("settings_grid").num_columns(3).show(ui, |ui| {
            ui.label(Messages::Language);
            ui.horizontal(|ui| {
                let current_lang = Language::from(self.config.language.clone());
                [Language::EN, Language::DE].iter().for_each(|lang| {
                    if ui
                        .add(SelectableLabel::new(current_lang == *lang, lang.name()))
                        .clicked()
                    {
                        self.state.config_state.language = *lang;
                        let cfg = Config {
                            data_folder: self.config.data_folder.clone(),
                            file_open_command: self.config.file_open_command.clone(),
                            language: self.state.config_state.language.name().into(),
                        };
                        if let Err(e) = config::save_config(&cfg) {
                            error!("Could not save config: {e}");
                        } else {
                            self.config = cfg;
                        }
                    }
                });
            });
            ui.end_row();
            ui.label(Messages::FileOpenProgram);
            let file_open_command = self.config.file_open_command.clone();
            if ui.button(Messages::Change.msg()).clicked() {
                self.state.config_state.file_open_command_change =
                    !self.state.config_state.file_open_command_change;
            }
            ui.add(
                TextEdit::singleline(
                    &mut file_open_command
                        .as_ref()
                        .map_or_else(|| "", |path| path.as_str()),
                )
                .desired_width(250.0),
            );

            if self.state.config_state.file_open_command_change {
                ui.end_row();
                ui.text_edit_singleline(&mut self.state.config_state.file_open_command);
                if ui.button(Messages::Save.msg()).clicked() {
                    self.config.file_open_command =
                        Some(self.state.config_state.file_open_command.clone());
                    if let Err(e) = config::save_config(&self.config) {
                        error!("Could not save config: {e}");
                    } else {
                        util::send_gui_event(
                            &self.context.gui_event_sender,
                            GuiEvent::ShowInfoNotification(
                                Messages::SuccessFullyChangedProgramToOpen.msg().to_owned(),
                            ),
                        );
                    }
                }
            }
            ui.end_row();

            ui.label(Messages::DataFolder);
            let data_folder = self.config.data_folder.clone();
            if ui.button(Messages::Open.msg()).clicked() {
                let mut dialog =
                    ui::get_localized_select_folder_dialog(None, Messages::SelectFolder.msg());
                dialog.open();
                self.state.config_state.open_file_dialog = Some(dialog);
            }
            ui.add(
                TextEdit::singleline(
                    &mut data_folder
                        .as_ref()
                        .map_or_else(|| "", |path| path.to_str().unwrap_or("")),
                )
                .desired_width(250.0),
            );
            ui.end_row();

            if let Some(dialog) = &mut self.state.config_state.open_file_dialog {
                if dialog.show(ui.ctx()).selected() {
                    if let Some(folder) = dialog.path() {
                        self.state.config_state.selected_folder = Some(folder.to_path_buf());
                        self.state.config_state.change_data_folder_dialog = Some(Dialog::new(
                            Messages::ReallyChangeDataFolder.msg().to_string(),
                            Messages::Save.msg(),
                            Messages::Cancel.msg(),
                        ));
                    }
                }
            }

            if let Some(ref dialog) = self.state.config_state.change_data_folder_dialog {
                match dialog::render_dialog(ui.ctx(), dialog) {
                    DialogResponse::Ok => {
                        self.state.config_state.change_data_folder_dialog = None;
                        if let Some(ref source) = self.config.data_folder {
                            if let Some(ref target) = self.state.config_state.selected_folder {
                                match util::files::move_folder_recursively(
                                    source.as_path(),
                                    target.as_path(),
                                ) {
                                    Err(e) => {
                                        util::send_gui_event(
                                            &self.context.gui_event_sender,
                                            GuiEvent::ShowErrorNotification(
                                                Messages::ErrorChangingDataFolder.msg().to_owned(),
                                            ),
                                        );
                                        log::error!("error while changing data folder: {e}")
                                    }
                                    Ok(_) => {
                                        self.config.data_folder = Some(target.to_path_buf());
                                        if let Err(e) = config::save_config(&self.config) {
                                            error!("Could not save config: {e}");
                                        } else {
                                            util::send_gui_event(
                                                &self.context.gui_event_sender,
                                                GuiEvent::ShowInfoNotification(
                                                    Messages::SuccessFullyChangedDataFolder
                                                        .msg()
                                                        .to_owned(),
                                                ),
                                            );
                                            util::send_event_and_request_repaint(
                                                ui.ctx(),
                                                &self.context.background_event_sender,
                                                Event::SetDB(target.to_owned()),
                                            );
                                        }
                                    }
                                }
                            }
                        }
                        self.state.config_state.selected_folder = None;
                    }
                    DialogResponse::Cancel => {
                        self.state.config_state.change_data_folder_dialog = None;
                        self.state.config_state.selected_folder = None;
                        info!("canceled")
                    }
                    _ => (),
                }
            }
        });
    }
}

impl App for Helferlein {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.handle_config_init(ctx);
        self.handle_gui_events();

        egui::CentralPanel::default().show(ctx, |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    notification::render_notifications(ctx, &mut self.state);
                    ui.label(RichText::new(Messages::Title).strong());
                    ui.separator();
                    self.build_navigation(ui);
                    ui.separator();
                    match self.state.navigation.current_screen {
                        Screen::Home => {
                            self.build_home(ui);
                        }
                        Screen::Invoice => {
                            invoice::build(ctx, &mut self.state, &self.context, ui);
                        }
                        Screen::Accounting => {
                            accounting::build(
                                ctx,
                                &mut self.state,
                                &self.config,
                                &self.context,
                                ui,
                            );
                        }
                        Screen::Settings => {
                            self.build_settings(ui);
                        }
                    };
                    ui.separator();
                });
            });
        });
    }
}

#[derive(Debug)]
enum GuiError {
    CopyItemFileFailed(String),
    FileAccessError(String),
    ExportFailed(String),
    DatabaseError(String),
}

impl From<&GuiError> for String {
    fn from(val: &GuiError) -> Self {
        match val {
            GuiError::CopyItemFileFailed(msg) => msg.to_owned(),
            GuiError::FileAccessError(msg) => msg.to_owned(),
            GuiError::ExportFailed(msg) => msg.to_owned(),
            GuiError::DatabaseError(msg) => msg.to_owned(),
        }
    }
}

impl std::fmt::Display for GuiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GuiError::CopyItemFileFailed(msg) => {
                write!(f, "{}", msg)
            }
            GuiError::FileAccessError(msg) => {
                write!(f, "{}", msg)
            }
            GuiError::ExportFailed(msg) => {
                write!(f, "{}", msg)
            }
            GuiError::DatabaseError(msg) => {
                write!(f, "{}", msg)
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum Screen {
    Home,
    Accounting,
    Invoice,
    Settings,
}

enum Event {
    RemoveItem(String, DateRange),
    FetchItems(DateRange),
    FetchNames(),
    FetchCompanies(),
    FetchCategories(),
    SaveItem(AccountingItem, DateRange),
    SetDB(PathBuf),
    OpenFile(String),
    FetchInvoiceTemplates(),
    SaveInvoiceTemplate(Box<Invoice>),
    RemoveInvoiceTemplate(String),
}

#[derive(Debug)]
enum GuiEvent {
    ShowInfoNotification(String),
    ShowErrorNotification(String),
    SetAccountingItems(Vec<AccountingItem>),
    SetNames(Vec<String>),
    SetCompanies(Vec<String>),
    SetCategories(Vec<String>),
    SetInvoiceTemplates(Vec<Invoice>),
}
