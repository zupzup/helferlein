use crate::accounting::AccountingState;
use crate::invoice::InvoiceState;
use crate::messages::Messages;
use crate::GuiError;
use chrono::Datelike;
use log::{error, info};
use std::fs::{copy, create_dir_all, read_dir, remove_dir_all, remove_file};
use std::io;
use std::path::{Path, PathBuf};

pub(crate) const PATH_FOR_FILES: &str = "files";
pub(crate) const SUFFIX_FOR_FILES: &str = "_files";

// returns the path of the copied file at it's new destination
pub(crate) fn copy_file_and_rename(
    new_name: &str,
    destination_folder: &Path,
    file_path: &PathBuf,
) -> Result<PathBuf, GuiError> {
    info!("file path {:?}", file_path);
    let mut files_path = destination_folder.to_path_buf();
    if !files_path.exists() {
        create_dir_all(&files_path).map_err(|e| {
            GuiError::FileAccessError(format!(
                "{}: {:?}, {}",
                Messages::FilesFolderNotCreated.msg(),
                files_path,
                e,
            ))
        })?;
    }

    files_path.push(new_name);
    if let Some(ext) = file_path.extension() {
        files_path.set_extension(ext);
    }
    // only copy, if it's not the same file to avoid deleting the file
    if file_path != &files_path {
        copy(file_path, &files_path).map_err(|e| {
            error!("Copy, from {file_path:?} to {files_path:?} failed: {e}");
            GuiError::CopyItemFileFailed(format!("{}, {}", Messages::ItemCopyFailed.msg(), e,))
        })?;
    }

    Ok(files_path)
}

pub(crate) fn move_folder_recursively(source: &Path, target: &Path) -> Result<(), GuiError> {
    if target.starts_with(source) {
        return Err(GuiError::FileAccessError(String::from(
            "target folder can't be inside source folder",
        )));
    }
    if !source.exists() {
        return Err(GuiError::FileAccessError(format!(
            "source folder does not exist: {:?}",
            source
        )));
    }
    if !target.exists() {
        return Err(GuiError::FileAccessError(format!(
            "target folder does not exist: {:?}",
            target
        )));
    }

    copy_dir_all(source, target).map_err(|e| GuiError::FileAccessError(e.to_string()))?;

    if let Err(e) = remove_dir_all(source) {
        log::error!("error while removing source data folder: {e}");
    }
    Ok(())
}

fn copy_dir_all(source: impl AsRef<Path>, target: impl AsRef<Path>) -> io::Result<()> {
    if !target.as_ref().exists() {
        create_dir_all(&target)?;
    }
    for entry in read_dir(source)? {
        let entry = entry?;
        let t = entry.file_type()?;
        if t.is_dir() {
            copy_dir_all(entry.path(), target.as_ref().join(entry.file_name()))?;
        } else {
            copy(entry.path(), target.as_ref().join(entry.file_name()))?;
        }
    }
    Ok(())
}

// logs errors
pub(crate) fn delete_file_and_folder(file: &Path, folder: &Path) {
    let _ = remove_file(file).map_err(|e| {
        log::error!(
            "{}: {:?}, {}",
            Messages::FileCouldNotBeDeleted.msg(),
            file,
            e,
        )
    });
    let _ = remove_dir_all(folder).map_err(|e| {
        log::error!(
            "{}: {:?}, {}",
            Messages::FolderCouldNotBeDeleted.msg(),
            folder,
            e,
        )
    });
}

// creates a file name suggestion based on the data folder and "year-month/quarter"
pub(crate) fn build_file_name_suggestion(accounting_state: &AccountingState) -> Option<String> {
    let mut file_name = String::default();
    let year = accounting_state.selected_year;
    file_name.push_str(&year.to_string());
    if let Some(quarter) = accounting_state.selected_quarter {
        file_name.push('-');
        file_name.push_str(quarter.name());
    } else if let Some(month) = accounting_state.selected_month {
        file_name.push('-');
        file_name.push_str(month.name());
    }
    file_name.push_str(".pdf");
    Some(file_name)
}

pub(crate) fn build_invoice_file_name(invoice_state: &InvoiceState) -> String {
    let now = chrono::Local::now().date_naive();
    let mut file_name = format!(
        "{}-{}_{}_{}_{}",
        Messages::InvoiceShort.msg(),
        now.year(),
        now.month(),
        now.day(),
        invoice_state.metadata.name
    );
    file_name.push_str(".pdf");
    file_name
}
