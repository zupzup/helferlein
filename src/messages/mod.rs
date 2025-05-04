use eframe::egui::{RichText, WidgetText};

use crate::get_language;

#[derive(Debug, Clone, Eq, PartialEq, Copy)]
pub(crate) enum Language {
    EN,
    DE,
}

impl Language {
    pub(crate) fn name(&self) -> &'static str {
        match self {
            Language::EN => "en",
            Language::DE => "de",
        }
    }
}

impl From<String> for Language {
    fn from(value: String) -> Self {
        match value.as_str() {
            "de" => Language::DE,
            _ => Language::EN,
        }
    }
}

impl From<&str> for Language {
    fn from(value: &str) -> Self {
        match value {
            "de" => Language::DE,
            _ => Language::EN,
        }
    }
}

#[derive(Debug)]
pub(crate) enum Messages {
    // General
    Title,

    // Settings
    DataFolder,
    Language,
    FileOpenProgram,
    SuccessFullyChangedDataFolder,
    ErrorChangingDataFolder,
    SuccessFullyChangedProgramToOpen,

    // Invoice
    General,
    Invoice,
    InvoiceShort,
    ServicePeriod,
    CreateNewInvoice,
    From,
    To,
    Items,
    PostalAddress,
    Zip,
    City,
    Country,
    VatNr,
    Misc,
    Nr,
    Pos,
    Description,
    Unit,
    UnitShort,
    Amount,
    Qty,
    PricePerUnit,
    SaveAsTemplate,
    Templates,
    PreText,
    PostText,
    BankData,

    // Accounting
    Accounting,
    Year,
    Quarter,
    Month,
    Ingoing,
    Outgoing,
    AccountingSummary,
    CategoriesSummary,
    Sum,

    // Accounting Items
    InvoiceType,
    InvoiceNumber,
    InvoiceNumberText,
    Date,
    Name,
    Company,
    Category,
    Net,
    Vat,
    Tax,
    Gross,
    Total,
    File,
    ChooseFile,
    SaveFile,
    SelectFolder,
    FileTitle,
    Link,
    AddItem,
    NewItem,
    EditItem,
    Edit,
    Delete,

    // Navigation
    Home,
    Settings,
    Welcome,

    // Buttons / Ui
    Select,
    Fill,
    SaveItem,
    Save,
    Rename,
    Refresh,
    NewFolder,
    ParentFolder,
    ShowHidden,
    Change,
    Cancel,
    Done,
    Reset,
    Open,
    ThereAreWarnings,
    ReallySave,
    ReallyChangeDataFolder,
    Export,

    // Months
    January,
    February,
    March,
    April,
    May,
    June,
    July,
    August,
    September,
    October,
    November,
    December,

    // Months short
    Jan,
    Feb,
    Mar,
    Apr,
    // no may, because it's 3 letters anyway
    Jun,
    Jul,
    Aug,
    Sep,
    Oct,
    Nov,
    Dec,

    // Suggestions
    NoDataFolder,

    // Infos
    FileCopied,
    PDFCreated,
    ItemDeleted,
    ItemCreated,
    InvoiceTemplateCreated,
    InvoiceTemplateFilled,
    ItemsFetched,

    // Warnings
    DateNotInSelectedDateRange,

    // Errors
    PDFFilesCopyFailed,
    DateNotValid,
    CanNotBeEmpty,
    NotANumber,
    FilesFolderNotCreated,
    FileCouldNotBeDeleted,
    FolderCouldNotBeDeleted,
    ItemCopyFailed,
    PDFNotCreated,
    CouldNotFetchData,
    CouldNotDeleteItem,
    CouldNotFetchNames,
    CouldNotFetchCategories,
    CouldNotFetchCompanies,
    CouldNotCreateItem,
    CouldNotCreateInvoiceTemplate,
    CouldNotOpenFile,
    TooManyItemsForPDFExport,
}

impl From<Messages> for &str {
    fn from(val: Messages) -> Self {
        val.msg()
    }
}

impl From<Messages> for WidgetText {
    fn from(val: Messages) -> Self {
        WidgetText::from(val.msg())
    }
}

impl From<Messages> for RichText {
    fn from(val: Messages) -> Self {
        RichText::from(val.msg())
    }
}

impl From<&Messages> for &str {
    fn from(val: &Messages) -> Self {
        val.msg()
    }
}

impl From<Messages> for String {
    fn from(val: Messages) -> Self {
        val.msg().to_owned()
    }
}

impl std::fmt::Display for Messages {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.msg())
    }
}

impl Messages {
    pub(crate) fn months() -> &'static [&'static str] {
        match get_language() {
            Language::EN => &[
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ],
            Language::DE => &[
                "Jänner",
                "Februar",
                "März",
                "April",
                "Mai",
                "Juni",
                "Juli",
                "August",
                "September",
                "Oktober",
                "November",
                "Dezember",
            ],
        }
    }

    pub(crate) fn days() -> &'static [&'static str] {
        match get_language() {
            Language::EN => &["Mo", "Tu", "We", "Th", "Fr", "Sa", "Su"],
            Language::DE => &["Mo", "Di", "Mi", "Do", "Fr", "Sa", "So"],
        }
    }

    pub(crate) fn msg(&self) -> &'static str {
        match get_language() {
            Language::EN => {
                match self {
                    // General
                    Messages::Title => "Helferlein",

                    // Settings
                    Messages::DataFolder => "Data Folder",
                    Messages::Language => "Language",
                    Messages::FileOpenProgram => "Program to open Files",
                    Messages::SuccessFullyChangedDataFolder => "Data folder changed successfully!",
                    Messages::ErrorChangingDataFolder => {
                        "There was an error changing the data folder."
                    }
                    Messages::SuccessFullyChangedProgramToOpen => {
                        "Program to open files changed successfully!"
                    }
                    // Invoice
                    Messages::Invoice => "Invoice",
                    Messages::InvoiceShort => "inv",
                    Messages::General => "General",
                    Messages::ServicePeriod => "Service Period",
                    Messages::CreateNewInvoice => "Create new Invoice",
                    Messages::From => "From",
                    Messages::To => "To",
                    Messages::Items => "Items",
                    Messages::PostalAddress => "Address",
                    Messages::Zip => "Zip",
                    Messages::City => "City",
                    Messages::Country => "Country",
                    Messages::VatNr => "Vat Nr.",
                    Messages::Misc => "Misc",
                    Messages::Nr => "Nr.",
                    Messages::Pos => "Pos",
                    Messages::Description => "Description",
                    Messages::Unit => "Unit",
                    Messages::UnitShort => "Unit",
                    Messages::Qty => "Qty",
                    Messages::Amount => "Amount",
                    Messages::PricePerUnit => "Price per unit",
                    Messages::SaveAsTemplate => "Save as Template",
                    Messages::Templates => "Templates",
                    Messages::PreText => "Pre Text",
                    Messages::PostText => "Post Text",
                    Messages::BankData => "Bank Data",

                    // Accounting
                    Messages::Accounting => "Accounting",
                    Messages::Year => "Year",
                    Messages::Quarter => "Quarter",
                    Messages::Month => "Month",
                    Messages::Ingoing => "Ingoing",
                    Messages::Outgoing => "Outgoing",
                    Messages::AccountingSummary => "Accounting Summary",
                    Messages::CategoriesSummary => "Categories Summary",
                    Messages::Sum => "Sum",

                    // Accounting Items
                    Messages::InvoiceType => "Inv. Type",
                    Messages::InvoiceNumber => "#",
                    Messages::InvoiceNumberText => "Invoice Number",
                    Messages::Date => "Date",
                    Messages::Name => "Name",
                    Messages::Company => "Company",
                    Messages::Category => "Category",
                    Messages::Net => "Net",
                    Messages::Vat => "VAT",
                    Messages::Tax => "Tax",
                    Messages::Gross => "Gross",
                    Messages::Total => "Total",
                    Messages::File => "File",
                    Messages::ChooseFile => "Choose File",
                    Messages::SaveFile => "Save File",
                    Messages::SelectFolder => "Select Folder",
                    Messages::FileTitle => "File:",
                    Messages::Link => "Link",
                    Messages::AddItem => "Add New Item",
                    Messages::NewItem => "New Item",
                    Messages::EditItem => "Edit Item",
                    Messages::Edit => "Edit",
                    Messages::Delete => "Delete",

                    // Navigation
                    Messages::Home => "Home",
                    Messages::Welcome => "Welcome",
                    Messages::Settings => "Settings",

                    // Buttons / Ui
                    Messages::Select => "Select",
                    Messages::Fill => "Fill",
                    Messages::Done => "Done",
                    Messages::SaveItem => "Save Item",
                    Messages::Save => "Save",
                    Messages::Rename => "Rename",
                    Messages::Refresh => "Refresh",
                    Messages::NewFolder => "New Folder",
                    Messages::ParentFolder => "Parent Folder",
                    Messages::ShowHidden => "Show Hidden",
                    Messages::Change => "Change",
                    Messages::Cancel => "Cancel",
                    Messages::Reset => "Reset",
                    Messages::Open => "Open",
                    Messages::ThereAreWarnings => "⚠ There are warnings!",
                    Messages::ReallySave => "Do you really want to save?",
                    Messages::ReallyChangeDataFolder => {
                        "Do you really want to save? If there are files at the new location, they might be overridden."
                    }
                    Messages::Export => "Export",

                    //Months
                    Messages::January => "January",
                    Messages::February => "February",
                    Messages::March => "March",
                    Messages::April => "April",
                    Messages::May => "May",
                    Messages::June => "June",
                    Messages::July => "July",
                    Messages::August => "August",
                    Messages::September => "September",
                    Messages::October => "October",
                    Messages::November => "November",
                    Messages::December => "December",

                    //Months short
                    Messages::Jan => "Jan",
                    Messages::Feb => "Feb",
                    Messages::Mar => "Mar",
                    Messages::Apr => "Apr",
                    Messages::Jun => "Jun",
                    Messages::Jul => "Jul",
                    Messages::Aug => "Aug",
                    Messages::Sep => "Sep",
                    Messages::Oct => "Oct",
                    Messages::Nov => "Nov",
                    Messages::Dec => "Dec",

                    // Suggestions
                    Messages::NoDataFolder => {
                        "Please set a folder to store your accounting data. Make sure the data is safe there and is backed up regularly."
                    }
                    // Infos
                    Messages::FileCopied => "Item file was copied to data folder.",
                    Messages::PDFCreated => {
                        "The PDF report was created and all invoice files were put in a \"_files\" folder beside it."
                    }
                    Messages::ItemDeleted => "Item successfully deleted.",
                    Messages::ItemCreated => "Item successfully created.",
                    Messages::InvoiceTemplateCreated => "Invoice Template successfully created.",
                    Messages::InvoiceTemplateFilled => "Invoice Template filled.",
                    Messages::ItemsFetched => "Items successfully fetched.",

                    // Warnings
                    Messages::DateNotInSelectedDateRange => {
                        "The selected date is not within the selected date range."
                    }

                    // Errors
                    Messages::DateNotValid => "Not a valid date.",
                    Messages::PDFFilesCopyFailed => {
                        "files could not be copied. PDF report was not created. Please check the files in the sheet."
                    }
                    Messages::CanNotBeEmpty => "can not be empty.",
                    Messages::NotANumber => "is not a number.",
                    Messages::FilesFolderNotCreated => {
                        "Couldn't create files folder in the data folder"
                    }

                    Messages::FileCouldNotBeDeleted => "Couldn't delete file",
                    Messages::FolderCouldNotBeDeleted => "Couldn't delete folder",
                    Messages::ItemCopyFailed => "Couldn't copy file to data folder",
                    Messages::PDFNotCreated => "The PDF report could not be created.",
                    Messages::CouldNotFetchData => "Could not fetch data.",
                    Messages::CouldNotDeleteItem => "Could not delete item.",
                    Messages::CouldNotFetchNames => "Could not fetch names.",
                    Messages::CouldNotFetchCategories => "Could not fetch categories.",
                    Messages::CouldNotFetchCompanies => "Could not fetch companies",
                    Messages::CouldNotCreateItem => "Could not create item.",
                    Messages::CouldNotOpenFile => "Could not open file.",
                    Messages::CouldNotCreateInvoiceTemplate => "Could not create invoice template.",
                    Messages::TooManyItemsForPDFExport => "Too many items for PDF export.",
                }
            }
            Language::DE => {
                match self {
                    // General
                    Messages::Title => "Helferlein",

                    // Settings
                    Messages::DataFolder => "Datenverzeichnis",
                    Messages::Language => "Sprache",
                    Messages::FileOpenProgram => "Programm um Dateien zu öffnen",
                    Messages::SuccessFullyChangedDataFolder => {
                        "Datenverzeichnis erfolgreich geändert!"
                    }
                    Messages::ErrorChangingDataFolder => {
                        "Es ist ein Fehler aufgetreten beim Ändern des Datenverzeichnisses."
                    }
                    Messages::SuccessFullyChangedProgramToOpen => {
                        "Programm um Dateien zu öffnen erfolgreich geändert!"
                    }

                    // Rechnung
                    Messages::Invoice => "Rechnung",
                    Messages::InvoiceShort => "re",
                    Messages::General => "Allgemein",
                    Messages::ServicePeriod => "Leistungszeitraum",
                    Messages::CreateNewInvoice => "Neue Rechnung erstellen",
                    Messages::From => "Von",
                    Messages::To => "An",
                    Messages::Items => "Posten",
                    Messages::PostalAddress => "Adresse",
                    Messages::Zip => "PLZ",
                    Messages::City => "Stadt",
                    Messages::Country => "Land",
                    Messages::VatNr => "USt-IdNr.",
                    Messages::Misc => "Div.",
                    Messages::Nr => "Nr.",
                    Messages::Pos => "Pos",
                    Messages::Description => "Beschreibung",
                    Messages::Unit => "Einheit",
                    Messages::UnitShort => "Einh.",
                    Messages::Qty => "Anz.",
                    Messages::Amount => "Menge",
                    Messages::PricePerUnit => "Preis/Einheit",
                    Messages::SaveAsTemplate => "Als Vorlage speichern",
                    Messages::Templates => "Vorlagen",
                    Messages::PreText => "Textzeilen Bevor",
                    Messages::PostText => "Textzeilen Danach",
                    Messages::BankData => "Bankdaten",

                    // Accounting
                    Messages::Accounting => "Buchhaltung",
                    Messages::Year => "Jahr",
                    Messages::Quarter => "Quartal",
                    Messages::Month => "Monat",
                    Messages::Ingoing => "Eingang",
                    Messages::Outgoing => "Ausgang",
                    Messages::AccountingSummary => "Buchhaltungsübersicht",
                    Messages::CategoriesSummary => "Kategorienübersicht",
                    Messages::Sum => "Summe",

                    // Accounting Items
                    Messages::InvoiceType => "Typ",
                    Messages::InvoiceNumber => "#",
                    Messages::InvoiceNumberText => "Rechnungsnummer",
                    Messages::Date => "Datum",
                    Messages::Name => "Name",
                    Messages::Company => "Firma",
                    Messages::Category => "Kategorie",
                    Messages::Net => "Netto",
                    Messages::Vat => "USt",
                    Messages::Tax => "Steuer",
                    Messages::Gross => "Brutto",
                    Messages::Total => "Gesamt",
                    Messages::File => "Datei",
                    Messages::ChooseFile => "Datei auswählen",
                    Messages::SaveFile => "Datei speichern",
                    Messages::SelectFolder => "Ordner auswählen",
                    Messages::FileTitle => "Datei:",
                    Messages::Link => "Link",
                    Messages::AddItem => "Neuen Eintrag hinzufügen",
                    Messages::NewItem => "Neuer Eintrag",
                    Messages::EditItem => "Eintrag ändern",
                    Messages::Edit => "Ändern",
                    Messages::Delete => "Löschen",

                    // Navigation
                    Messages::Home => "Übersicht",
                    Messages::Welcome => "Willkommen",
                    Messages::Settings => "Einstellungen",

                    // Buttons / Ui
                    Messages::Select => "Auswählen",
                    Messages::Fill => "Einfüllen",
                    Messages::Done => "Erledigt",
                    Messages::SaveItem => "Eintrag Speichern",
                    Messages::Save => "Speichern",
                    Messages::Rename => "Rename",
                    Messages::Refresh => "Aktualisieren",
                    Messages::NewFolder => "Neuer Ordner",
                    Messages::ParentFolder => "Übergeordneter Ordner",
                    Messages::ShowHidden => "Versteckte Anzeigen",
                    Messages::Change => "Ändern",
                    Messages::Cancel => "Abbrechen",
                    Messages::Reset => "Zurücksetzen",
                    Messages::Open => "Öffnen",
                    Messages::ThereAreWarnings => "⚠ Es gibt Warnungen!",
                    Messages::ReallySave => "Willst du wirklich speichern?",
                    Messages::ReallyChangeDataFolder => {
                        "Willst du wirklich speichern? Wenn es Dateien am ausgewählten Ort gibt, werden diese überschrieben."
                    }
                    Messages::Export => "Exportieren",

                    //Months
                    Messages::January => "Jänner",
                    Messages::February => "Februar",
                    Messages::March => "März",
                    Messages::April => "April",
                    Messages::May => "Mai",
                    Messages::June => "Juni",
                    Messages::July => "Juli",
                    Messages::August => "August",
                    Messages::September => "September",
                    Messages::October => "Oktober",
                    Messages::November => "November",
                    Messages::December => "Dezember",

                    //Months short
                    Messages::Jan => "Jän",
                    Messages::Feb => "Feb",
                    Messages::Mar => "Mär",
                    Messages::Apr => "Apr",
                    Messages::Jun => "Jun",
                    Messages::Jul => "Jul",
                    Messages::Aug => "Aug",
                    Messages::Sep => "Sep",
                    Messages::Oct => "Okt",
                    Messages::Nov => "Nov",
                    Messages::Dec => "Dez",

                    // Suggestions
                    Messages::NoDataFolder => {
                        "Bitte setz einen Ordner um deine Buchhaltungsdaten zu speichern. Stell sicher, dass der Ordner sicher ist und regelmäßig gebackuppt wird.."
                    }
                    // Infos
                    Messages::FileCopied => {
                        "Eintragsdatei wurde in das Dateienverzeichnis kopiert."
                    }
                    Messages::PDFCreated => {
                        "Der PDF Report wurde erstellt und alle Rechnungsdateien wurden in den \"_files\" im gleichen Ordner erstellt."
                    }
                    Messages::ItemDeleted => "Eintrag erfolgreich gelöscht.",
                    Messages::ItemCreated => "Eintrag erfolgreich erstellt.",
                    Messages::InvoiceTemplateCreated => "Rechnungsvorlage erfolgreich erstellt.",
                    Messages::InvoiceTemplateFilled => "Rechnungsvorlage eingefüllt",
                    Messages::ItemsFetched => "Einträge gefunden.",

                    // Warnings
                    Messages::DateNotInSelectedDateRange => {
                        "Das augewählte Datum ist nicht innerhalb des ausgewählten Bereichs."
                    }

                    // Errors
                    Messages::DateNotValid => "Kein gültiges Datum.",
                    Messages::PDFFilesCopyFailed => {
                        "dateien konnten nicht kopiert werden. Der PDF Report wurde nicht erstellt. Bitte überprüfe die Dateien der ausgewählten Einträge."
                    }
                    Messages::CanNotBeEmpty => "kann nicht leer sein.",
                    Messages::NotANumber => "ist keine Zahl.",
                    Messages::FilesFolderNotCreated => {
                        "Dateien im Datenverzeichnis konnten nicht angelegt werden."
                    }

                    Messages::FileCouldNotBeDeleted => "Datei konnte nicht gelöscht werden.",
                    Messages::FolderCouldNotBeDeleted => "Ordner konnte nicht gelöscht werden.",
                    Messages::ItemCopyFailed => {
                        "Konnte Dateien nicht in das Datenverzeichnis kopieren.."
                    }
                    Messages::PDFNotCreated => "Der PDF Report wurde nicht erstellt.",
                    Messages::CouldNotFetchData => "Daten konnten nicht gefunden werden.",
                    Messages::CouldNotDeleteItem => "Eintrag konnte nicht gelöscht werden.",
                    Messages::CouldNotFetchNames => "Namen konnten nicht gefunden werden.",
                    Messages::CouldNotFetchCategories => {
                        "Kategorien konnten nicht gefunden werden."
                    }
                    Messages::CouldNotFetchCompanies => "Firen konnten nicht gefunden werden.",
                    Messages::CouldNotCreateItem => "Eintrag konnte nicht erstellt werden.",
                    Messages::CouldNotOpenFile => "Datei konnte nicht geöffnet werden.",
                    Messages::CouldNotCreateInvoiceTemplate => {
                        "Rechnungsvorlage konnte nicht erstellt werden."
                    }
                    Messages::TooManyItemsForPDFExport => "Zu viele Posten für PDF Export.",
                }
            }
        }
    }
}
