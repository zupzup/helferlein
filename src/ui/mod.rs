use crate::messages::Messages;
use egui_file::FileDialog;
use std::path::PathBuf;

pub(crate) mod autosuggest;
pub(crate) mod dialog;
pub(crate) mod notification;

fn get_localized_file_dialog(dialog: FileDialog, title: &str) -> FileDialog {
    dialog
        .title(title)
        .open_button_text(Messages::Open.msg().into())
        .save_button_text(Messages::Save.msg().into())
        .cancel_button_text(Messages::Cancel.msg().into())
        .rename_button_text(Messages::Rename.msg().into())
        .refresh_button_hover_text(Messages::Refresh.msg().into())
        .new_folder_name_text(Messages::NewFolder.msg().into())
        .new_folder_button_text(Messages::NewFolder.msg().into())
        .file_label_text(Messages::FileTitle.msg().into())
        .parent_folder_button_hover_text(Messages::ParentFolder.msg().into())
        .show_hidden_checkbox_text(Messages::ShowHidden.msg().into())
}

pub(crate) fn get_localized_open_file_dialog(path: Option<PathBuf>, title: &str) -> FileDialog {
    let dialog = FileDialog::open_file(path);
    get_localized_file_dialog(dialog, title)
}

pub(crate) fn get_localized_save_file_dialog(path: Option<PathBuf>, title: &str) -> FileDialog {
    let dialog = FileDialog::save_file(path);
    get_localized_file_dialog(dialog, title)
}

pub(crate) fn get_localized_select_folder_dialog(path: Option<PathBuf>, title: &str) -> FileDialog {
    let dialog = FileDialog::select_folder(path);
    get_localized_file_dialog(dialog, title)
}
