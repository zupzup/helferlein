use crate::{
    DATE_FORMAT, GuiError,
    data::{
        AccountingItem, AccountingSheet, Category, InvoiceType,
        currency::{CurrencyValue, VatCalculationResult, default_currency_value},
    },
    messages::Messages,
    util::{
        export::{FONT_SIZE, LINE_WIDTH, PADDING, ROW_HEIGHT},
        files::SUFFIX_FOR_FILES,
    },
};
use log::info;
use printpdf::{Color, IndirectFontRef, Line, Mm, PdfDocument, PdfLayerReference, Point, Rgb};
use rust_decimal::Decimal;
use std::{
    collections::HashMap,
    fs::{File, create_dir_all, remove_dir_all},
    io::BufWriter,
    path::{Path, PathBuf},
};

use super::{MARGIN, MAX_CHARS_CURRENCY, MAX_CHARS_VAT, TABLE_LINE_HEIGHT};

const ITEMS_PER_PAGE: usize = 22;
const SUMMARY_CUTOFF: usize = 8;
const MAX_DIGITS_NR: i32 = 3;
const CATEGORIES_SUMMARY_COLS: usize = 4;
const CATEGORIES_SUMMARY_ITEMS_PER_COL: usize = 6;

const WIDTH: Mm = Mm(297.0);
const HEIGHT: Mm = Mm(210.0);
const LEFT: Mm = Mm(MARGIN);
const RIGHT: Mm = Mm(WIDTH.0 - MARGIN);
const TOP: Mm = Mm(HEIGHT.0 - MARGIN);
const BOTTOM: Mm = Mm(MARGIN);

// COL WIDTHS
const INVOICE_TYPE_WIDTH: Mm = Mm(18.0);
const NR_WIDTH: Mm = Mm(10.0);
const DATE_WIDTH: Mm = Mm(22.0);
const COMPANY_NAME_WIDTH: Mm = Mm(80.0);
const COMPANY_NAME_CUTOFF_CHARS: usize = 40;
const CATEGORY_WIDTH: Mm = Mm(36.0);
const CATEGORY_CUTOFF_CHARS: usize = 18;
const NET_WIDTH: Mm = Mm(26.0);
const VAT_WIDTH: Mm = Mm(12.0);
const TAX_WIDTH: Mm = Mm(26.0);

// SUMMARY WIDTHS
const SUMMARY_INGOING_OUTGOING_WIDTH: Mm = Mm(20.0);
const SUMMARY_NET_WIDTH: Mm = Mm(30.0);
const SUMMARY_TAX_WIDTH: Mm = Mm(30.0);
const SUMMARY_CATEGORY_WIDTH: Mm = Mm(34.0);

#[derive(Debug, Clone)]
struct Summary {
    categories: HashMap<Category, CurrencyValue>,
    accounting: HashMap<InvoiceType, AccountingSummary>,
}

#[derive(Debug, Clone)]
struct AccountingSummary {
    net: CurrencyValue,
    tax: CurrencyValue,
    gross: CurrencyValue,
}

#[derive(Debug, Clone)]
pub(crate) struct CreatePDFResult {
    pub(crate) file: PathBuf,
    pub(crate) files_folder: PathBuf,
}

// returns the "_files" folder created for the PDF, as well as the file of the pdf
pub(crate) fn create_accounting_pdf(
    file_name: &Path,
    sheet: &AccountingSheet,
) -> Result<CreatePDFResult, GuiError> {
    // SETUP
    let title = create_title(sheet);
    let num_items = sheet.items.len();
    let pages = (num_items / ITEMS_PER_PAGE) + 1;
    info!("items: {num_items}, pages: {pages}");

    let (doc, page1, layer) = PdfDocument::new(&title, WIDTH, HEIGHT, "layer");
    let font = doc
        .add_builtin_font(printpdf::BuiltinFont::Helvetica)
        .expect("font is available");
    let bold_font = doc
        .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
        .expect("font is available");

    let current_layer = doc.get_page(page1).get_layer(layer);
    current_layer.set_outline_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    current_layer.set_outline_thickness(LINE_WIDTH);

    // TITLE
    current_layer.use_text(&title, FONT_SIZE.0, LEFT, TOP, &bold_font);
    let line = Line {
        points: vec![
            (Point::new(LEFT, Mm(TOP.0 - PADDING)), false),
            (Point::new(RIGHT, Mm(TOP.0 - PADDING)), false),
        ],
        is_closed: true,
    };
    current_layer.add_line(line);

    // Page 1
    build_items_table(sheet, &current_layer, &font, &bold_font, 0);

    current_layer.use_text(
        "1",
        FONT_SIZE.0,
        Mm(LEFT.0 + (RIGHT.0 - LEFT.0) / 2.0),
        BOTTOM,
        &font,
    );

    let mut last_page_idx = page1;
    let mut last_layer_idx = layer;

    // Pages 2 - N
    for i in 1..pages {
        let (page_idx, layer_idx) = doc.add_page(WIDTH, HEIGHT, format!("layer{i}"));
        let layer = doc.get_page(page_idx).get_layer(layer_idx);
        layer.set_outline_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
        layer.set_outline_thickness(LINE_WIDTH);

        build_items_table(sheet, &layer, &font, &bold_font, i * ITEMS_PER_PAGE);

        layer.use_text(
            format!("{}", i + 1),
            FONT_SIZE.0,
            Mm(LEFT.0 + (RIGHT.0 - LEFT.0) / 2.0),
            BOTTOM,
            &font,
        );

        last_page_idx = page_idx;
        last_layer_idx = layer_idx;
    }

    // SUMMARY
    let rest = num_items % ITEMS_PER_PAGE;
    let summary_needs_new_page = rest > SUMMARY_CUTOFF;
    info!("new page: {summary_needs_new_page}, {rest}");
    let (layer, top) = if summary_needs_new_page {
        let (page_idx, layer_idx) = doc.add_page(WIDTH, HEIGHT, format!("layer{}", pages));
        (doc.get_page(page_idx).get_layer(layer_idx), TOP)
    } else {
        // use last page, right after items + 1 ROW HEIGHT
        (
            doc.get_page(last_page_idx).get_layer(last_layer_idx),
            Mm(TOP.0 - ((rest + 3) as f32 * ROW_HEIGHT)),
        )
    };
    layer.set_outline_color(Color::Rgb(Rgb::new(0.7, 0.7, 0.7, None)));
    layer.set_outline_thickness(LINE_WIDTH);
    let summary = calculate_summary(sheet);
    build_summary(&summary, top, &layer, &font, &bold_font);

    // SAVE (overwrites the file)
    doc.save(&mut BufWriter::new(
        File::create(file_name).map_err(|e| GuiError::ExportFailed(e.to_string()))?,
    ))
    .map_err(|e| GuiError::ExportFailed(e.to_string()))?;

    // Create files folder, if it exists, remove the old one first
    let folder_name = file_name.with_extension("");
    let files_folder = PathBuf::from(format!(
        "{}{}",
        folder_name.to_str().expect("path is valid utf-8"),
        SUFFIX_FOR_FILES
    ));

    if files_folder.exists() {
        remove_dir_all(&files_folder).map_err(|e| GuiError::ExportFailed(e.to_string()))?;
    }

    create_dir_all(&files_folder).map_err(|e| GuiError::ExportFailed(e.to_string()))?;
    Ok(CreatePDFResult {
        file: file_name.to_path_buf(),
        files_folder,
    })
}

fn calculate_summary(sheet: &AccountingSheet) -> Summary {
    let mut categories: HashMap<Category, Decimal> = HashMap::new();
    let mut accounting = HashMap::new();
    let mut out_net_sum = default_currency_value();
    let mut out_tax_sum = default_currency_value();
    let mut out_gross_sum = default_currency_value();
    let mut in_net_sum = default_currency_value();
    let mut in_tax_sum = default_currency_value();
    let mut in_gross_sum = default_currency_value();

    sheet.items.iter().for_each(|item| match item.invoice_type {
        InvoiceType::Out => {
            let net = &item.net;
            out_net_sum = out_net_sum
                .checked_add(net.value)
                .unwrap_or_else(default_currency_value);
            let VatCalculationResult { tax, gross } = net.calculate_vat(item.vat);
            out_tax_sum = out_tax_sum
                .checked_add(tax.value)
                .unwrap_or_else(default_currency_value);
            out_gross_sum = out_gross_sum
                .checked_add(gross.value)
                .unwrap_or_else(default_currency_value);
        }
        InvoiceType::In => {
            let net = &item.net;
            in_net_sum = in_net_sum
                .checked_add(net.value)
                .unwrap_or_else(default_currency_value);
            let VatCalculationResult { tax, gross } = net.calculate_vat(item.vat);
            in_tax_sum = in_tax_sum
                .checked_add(tax.value)
                .unwrap_or_else(default_currency_value);
            in_gross_sum = in_gross_sum
                .checked_add(gross.value)
                .unwrap_or_else(default_currency_value);

            let category = &item.category;
            categories
                .entry(category.to_owned())
                .and_modify(|v| {
                    *v = v
                        .checked_add(net.value)
                        .unwrap_or_else(default_currency_value)
                })
                .or_insert(net.value);
        }
    });

    accounting.insert(
        InvoiceType::In,
        AccountingSummary {
            net: CurrencyValue::new_from_decimal(in_net_sum),
            tax: CurrencyValue::new_from_decimal(in_tax_sum),
            gross: CurrencyValue::new_from_decimal(in_gross_sum),
        },
    );
    accounting.insert(
        InvoiceType::Out,
        AccountingSummary {
            net: CurrencyValue::new_from_decimal(out_net_sum),
            tax: CurrencyValue::new_from_decimal(out_tax_sum),
            gross: CurrencyValue::new_from_decimal(out_gross_sum),
        },
    );

    Summary {
        categories: categories
            .into_iter()
            .map(|(k, v)| (k, CurrencyValue::new_from_decimal(v)))
            .collect(),
        accounting,
    }
}

fn create_title(sheet: &AccountingSheet) -> String {
    let mut title = format!("{} - {} ", Messages::Accounting.msg(), sheet.year);
    match sheet.quarter {
        None => {
            match sheet.month {
                None => {
                    // do nothing
                }
                Some(month) => {
                    title.push_str(month.name());
                }
            }
        }
        Some(quarter) => {
            title.push_str(quarter.name());
        }
    };
    title
}

// TABLE

// -------------------------------------------------------------------------------------------------
// | Date | Nr | Company + Text | Gross | Tax % | Tax | Net | Gross | Tax % | Tax | Net | Category |
// -------------------------------------------------------------------------------------------------
// |      |    |                |       |       |     |     |       |       |     |     |          |
// -------------------------------------------------------------------------------------------------
// |      |    |                |       |       |     |     |       |       |     |     |          |
// -------------------------------------------------------------------------------------------------
// |      |    |                |       |       |     |     |       |       |     |     |          |
// -------------------------------------------------------------------------------------------------
// |      |    |                |       |       |     |     |       |       |     |     |          |
// -------------------------------------------------------------------------------------------------
fn build_items_table(
    sheet: &AccountingSheet,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
    from_item: usize,
) {
    let top = match from_item {
        0 => Mm(TOP.0 - 5.0 * PADDING),
        _ => Mm(TOP.0 - PADDING),
    };
    render_table_header(top, layer, bold_font);
    for (idx, item) in sheet
        .items
        .iter()
        .skip(from_item)
        .take(ITEMS_PER_PAGE)
        .enumerate()
    {
        render_row(
            from_item + idx + 1,
            item,
            Mm(top.0 - ROW_HEIGHT - (idx as f32 * ROW_HEIGHT)),
            layer,
            font,
        );
    }
}

fn render_table_header(top: Mm, layer: &PdfLayerReference, font: &IndirectFontRef) {
    let mut col_line_x = 0.0;
    // START OF ROW
    render_row_line(top, layer);
    render_col_line(LEFT, top, layer);
    // Invoice Type
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::InvoiceType.msg(),
        layer,
        font,
    );
    col_line_x += INVOICE_TYPE_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Number
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::InvoiceNumber.msg(),
        layer,
        font,
    );
    col_line_x += NR_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Date
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Date.msg(),
        layer,
        font,
    );
    col_line_x += DATE_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // COMPANY + NAME
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &format!("{} - {}", Messages::Company.msg(), Messages::Name.msg()),
        layer,
        font,
    );
    col_line_x += COMPANY_NAME_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // CATEGORY
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Category.msg(),
        layer,
        font,
    );
    col_line_x += CATEGORY_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // NET
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Net.msg(),
        layer,
        font,
    );
    col_line_x += NET_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // VAT
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Vat.msg(),
        layer,
        font,
    );
    col_line_x += VAT_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Tax
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Tax.msg(),
        layer,
        font,
    );
    col_line_x += TAX_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Gross
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Gross.msg(),
        layer,
        font,
    );
    // Omit last col_line, since it's the row's col line

    // END OF ROW
    render_col_line(RIGHT, top, layer);
    render_row_line(Mm(top.0 - ROW_HEIGHT), layer);
}

fn render_row(
    idx: usize,
    item: &AccountingItem,
    top: Mm,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
) {
    let mut col_line_x = 0.0;
    // START OF ROW
    render_row_line(top, layer);
    render_col_line(LEFT, top, layer);
    // Invoice Type
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        item.invoice_type.name(),
        layer,
        font,
    );
    col_line_x += INVOICE_TYPE_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Number
    let nr_str = idx.to_string();
    render_col_text(
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_DIGITS_NR - nr_str.chars().count() as i32) as f32 * PADDING)), // right-align for max. 3
        // numbers
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &nr_str,
        layer,
        font,
    );
    col_line_x += NR_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Date
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &item.date.format(DATE_FORMAT).to_string(),
        layer,
        font,
    );
    col_line_x += DATE_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // COMPANY + NAME
    let mut company_name_str: String = format!("{} - {}", &item.company.0, &item.name);
    if company_name_str.chars().count() > COMPANY_NAME_CUTOFF_CHARS {
        company_name_str = company_name_str
            .chars()
            .take(COMPANY_NAME_CUTOFF_CHARS)
            .collect();
        company_name_str.push_str("...");
    }
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &company_name_str,
        layer,
        font,
    );
    col_line_x += COMPANY_NAME_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // CATEGORY
    let mut category_str = item.category.0.clone();
    if category_str.chars().count() > CATEGORY_CUTOFF_CHARS {
        category_str = category_str.chars().take(CATEGORY_CUTOFF_CHARS).collect();
        category_str.push_str("...");
    }
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &category_str,
        layer,
        font,
    );
    col_line_x += CATEGORY_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // NET
    let net_str = item.net.to_str();
    render_col_text(
        // right-align for max. 11 characters
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_CHARS_CURRENCY - net_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        net_str,
        layer,
        font,
    );
    col_line_x += NET_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // VAT
    let vat_str = item.vat.name();
    render_col_text(
        // right-align for max. 4 characters
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_CHARS_VAT - vat_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        vat_str,
        layer,
        font,
    );
    col_line_x += VAT_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    let VatCalculationResult { tax, gross } = item.net.calculate_vat(item.vat);
    // Tax
    let tax_str = tax.to_str();
    render_col_text(
        // right-align for max. 10 characters
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_CHARS_CURRENCY - tax_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        tax_str,
        layer,
        font,
    );
    col_line_x += TAX_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Gross
    let gross_str = gross.to_str();
    render_col_text(
        // right-align for max. 11 characters
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_CHARS_CURRENCY - gross_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        gross_str,
        layer,
        font,
    );
    // Omit last col_line, since it's the row's col line

    // END OF ROW
    render_col_line(RIGHT, top, layer);
    render_row_line(Mm(top.0 - ROW_HEIGHT), layer);
}

fn render_row_line(y: Mm, layer: &PdfLayerReference) {
    let line = Line {
        points: vec![(Point::new(LEFT, y), false), (Point::new(RIGHT, y), false)],
        is_closed: true,
    };

    layer.add_line(line);
}

fn render_col_text(x: Mm, y: Mm, text: &str, layer: &PdfLayerReference, font: &IndirectFontRef) {
    layer.set_line_height(TABLE_LINE_HEIGHT.0);
    layer.use_text(text, FONT_SIZE.0, x, y, font);
}

fn render_col_line(x: Mm, y: Mm, layer: &PdfLayerReference) {
    let line = Line {
        points: vec![
            (Point::new(x, y), false),
            (Point::new(x, Mm(y.0 - ROW_HEIGHT)), false),
        ],
        is_closed: true,
    };

    layer.add_line(line);
}

// SUMMARY

fn build_summary(
    summary: &Summary,
    top: Mm,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
) {
    // TITLE
    layer.use_text(
        Messages::AccountingSummary.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0 + (RIGHT.0 - LEFT.0) / 2.0),
        Mm(top.0 - 1.0 * ROW_HEIGHT),
        bold_font,
    );

    // Accounting headers
    layer.use_text(
        Messages::InvoiceType.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0),
        Mm(top.0 - 2.0 * ROW_HEIGHT),
        bold_font,
    );
    layer.use_text(
        Messages::Net.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0 + SUMMARY_INGOING_OUTGOING_WIDTH.0),
        Mm(top.0 - 2.0 * ROW_HEIGHT),
        bold_font,
    );
    layer.use_text(
        Messages::Tax.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0 + SUMMARY_INGOING_OUTGOING_WIDTH.0 + SUMMARY_NET_WIDTH.0),
        Mm(top.0 - 2.0 * ROW_HEIGHT),
        bold_font,
    );
    layer.use_text(
        Messages::Gross.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0 + SUMMARY_INGOING_OUTGOING_WIDTH.0 + SUMMARY_NET_WIDTH.0 + SUMMARY_TAX_WIDTH.0),
        Mm(top.0 - 2.0 * ROW_HEIGHT),
        bold_font,
    );
    // horizontal line
    let line = Line {
        points: vec![
            (
                Point::new(LEFT, Mm(top.0 - 2.0 * ROW_HEIGHT - PADDING)),
                false,
            ),
            (
                Point::new(
                    Mm(LEFT.0
                        + SUMMARY_INGOING_OUTGOING_WIDTH.0
                        + SUMMARY_NET_WIDTH.0
                        + SUMMARY_NET_WIDTH.0
                        + SUMMARY_TAX_WIDTH.0),
                    Mm(top.0 - 2.0 * ROW_HEIGHT - PADDING),
                ),
                false,
            ),
        ],
        is_closed: true,
    };
    layer.add_line(line);
    // vertical line
    let line = Line {
        points: vec![
            (
                Point::new(
                    Mm(LEFT.0 + SUMMARY_INGOING_OUTGOING_WIDTH.0 - PADDING),
                    Mm(top.0 - 1.0 * ROW_HEIGHT - PADDING),
                ),
                false,
            ),
            (
                Point::new(
                    Mm(LEFT.0 + SUMMARY_INGOING_OUTGOING_WIDTH.0 - PADDING),
                    Mm(top.0 - 4.0 * ROW_HEIGHT - PADDING),
                ),
                false,
            ),
        ],
        is_closed: true,
    };
    layer.add_line(line);
    // INGOING
    layer.use_text(
        Messages::Ingoing.msg(),
        FONT_SIZE.0,
        LEFT,
        Mm(top.0 - 3.0 * ROW_HEIGHT),
        bold_font,
    );
    render_accounting_summary(
        summary.accounting.get(&InvoiceType::In),
        layer,
        font,
        Mm(top.0 - 3.0 * ROW_HEIGHT),
    );

    // OUTGOING
    layer.use_text(
        Messages::Outgoing.msg(),
        FONT_SIZE.0,
        LEFT,
        Mm(top.0 - 4.0 * ROW_HEIGHT),
        bold_font,
    );
    render_accounting_summary(
        summary.accounting.get(&InvoiceType::Out),
        layer,
        font,
        Mm(top.0 - 4.0 * ROW_HEIGHT),
    );

    // CATEGORIES
    layer.use_text(
        Messages::CategoriesSummary.msg(),
        FONT_SIZE.0,
        Mm(LEFT.0 + (RIGHT.0 - LEFT.0) / 2.0),
        Mm(top.0 - 6.0 * ROW_HEIGHT),
        bold_font,
    );
    // horizontal line
    let line = Line {
        points: vec![
            (
                Point::new(LEFT, Mm(top.0 - 8.0 * ROW_HEIGHT - PADDING)),
                false,
            ),
            (
                Point::new(RIGHT, Mm(top.0 - 8.0 * ROW_HEIGHT - PADDING)),
                false,
            ),
        ],
        is_closed: true,
    };
    layer.add_line(line);

    let line_padding = 4.0;
    for i in 0..CATEGORIES_SUMMARY_COLS {
        let left = Mm(LEFT.0 + (i as f32 * (SUMMARY_CATEGORY_WIDTH.0 + SUMMARY_NET_WIDTH.0)));
        // Category headers
        layer.use_text(
            Messages::Category.msg(),
            FONT_SIZE.0,
            left,
            Mm(top.0 - 8.0 * ROW_HEIGHT),
            bold_font,
        );
        layer.use_text(
            format!("{} ({})", Messages::Sum.msg(), Messages::Net.msg()),
            FONT_SIZE.0,
            Mm(left.0 + SUMMARY_CATEGORY_WIDTH.0),
            Mm(top.0 - 8.0 * ROW_HEIGHT),
            bold_font,
        );
        if i > 0 {
            let line = Line {
                points: vec![
                    (
                        Point::new(Mm(left.0 - line_padding), Mm(top.0 - 7.0 * ROW_HEIGHT)),
                        false,
                    ),
                    (
                        Point::new(Mm(left.0 - line_padding), Mm(BOTTOM.0 + ROW_HEIGHT)),
                        false,
                    ),
                ],
                is_closed: true,
            };
            layer.add_line(line);
        }

        summary
            .categories
            .iter()
            .skip(i * CATEGORIES_SUMMARY_ITEMS_PER_COL)
            .take(CATEGORIES_SUMMARY_ITEMS_PER_COL)
            .enumerate()
            .for_each(|(idx, (k, v))| {
                let mut category_str = k.0.clone();
                if category_str.chars().count() > CATEGORY_CUTOFF_CHARS {
                    category_str = category_str.chars().take(CATEGORY_CUTOFF_CHARS).collect();
                    category_str.push_str("...");
                }
                layer.use_text(
                    &category_str,
                    FONT_SIZE.0,
                    left,
                    Mm(top.0 - (9.0 + idx as f32) * ROW_HEIGHT),
                    font,
                );
                let net_str = v.to_str();
                layer.use_text(
                    net_str,
                    FONT_SIZE.0,
                    Mm(left.0
                        + SUMMARY_CATEGORY_WIDTH.0
                        + ((MAX_CHARS_CURRENCY - net_str.chars().count() as i32) as f32 * PADDING)),
                    Mm(top.0 - (9.0 + idx as f32) * ROW_HEIGHT),
                    font,
                );
            });
    }
}

fn render_accounting_summary(
    accounting_summary: Option<&AccountingSummary>,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) {
    if let Some(acc_sum) = accounting_summary {
        let net_str = acc_sum.net.to_str();
        layer.use_text(
            net_str,
            FONT_SIZE.0,
            Mm(LEFT.0
                + SUMMARY_INGOING_OUTGOING_WIDTH.0
                + ((MAX_CHARS_CURRENCY - net_str.chars().count() as i32) as f32 * PADDING)),
            top,
            font,
        );
        let tax_str = acc_sum.tax.to_str();
        layer.use_text(
            tax_str,
            FONT_SIZE.0,
            Mm(LEFT.0
                + SUMMARY_INGOING_OUTGOING_WIDTH.0
                + SUMMARY_NET_WIDTH.0
                + ((MAX_CHARS_CURRENCY - tax_str.chars().count() as i32) as f32 * PADDING)),
            top,
            font,
        );
        let gross_str = acc_sum.gross.to_str();
        layer.use_text(
            gross_str,
            FONT_SIZE.0,
            Mm(LEFT.0
                + SUMMARY_INGOING_OUTGOING_WIDTH.0
                + SUMMARY_NET_WIDTH.0
                + SUMMARY_TAX_WIDTH.0
                + ((MAX_CHARS_CURRENCY - gross_str.chars().count() as i32) as f32 * PADDING)),
            top,
            font,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        data::{Company, Vat},
        util::Quarter,
    };
    use uuid::Uuid;
    fn accounting_item(
        it: InvoiceType,
        net: CurrencyValue,
        vat: Vat,
        category: Category,
    ) -> AccountingItem {
        AccountingItem {
            invoice_type: it,
            id: Uuid::now_v7(),
            date: chrono::Local::now().date_naive(),
            name: String::from("some name"),
            company: Company(String::from("some company")),
            net,
            vat,
            category,
            file: PathBuf::from("/some/file"),
        }
    }

    #[test]
    fn calculate_summary_empty() {
        let sheet = AccountingSheet {
            items: vec![],
            year: 2024,
            month: None,
            quarter: Some(Quarter::Q1),
        };
        let result = calculate_summary(&sheet);
        assert!(result.categories.is_empty());
        let ingoing = result.accounting.get(&InvoiceType::In).unwrap();
        let outgoing = result.accounting.get(&InvoiceType::Out).unwrap();

        assert!(ingoing.net.value.eq(&default_currency_value()));
        assert!(ingoing.tax.value.eq(&default_currency_value()));
        assert!(ingoing.gross.value.eq(&default_currency_value()));
        assert!(outgoing.net.value.eq(&default_currency_value()));
        assert!(outgoing.tax.value.eq(&default_currency_value()));
        assert!(outgoing.gross.value.eq(&default_currency_value()));
    }

    #[test]
    fn calculate_summary_one() {
        let net = CurrencyValue::new(225000);
        let vat = Vat::Twenty;
        let sheet = AccountingSheet {
            items: vec![accounting_item(
                InvoiceType::In,
                net.clone(),
                vat,
                Category(String::from("a")),
            )],
            year: 2024,
            month: None,
            quarter: Some(Quarter::Q1),
        };

        let result = calculate_summary(&sheet);
        assert!(!result.categories.is_empty());
        assert!(
            result
                .categories
                .get(&Category(String::from("a")))
                .unwrap()
                .value
                .eq(&net.value)
        );
        let ingoing = result.accounting.get(&InvoiceType::In).unwrap();
        let outgoing = result.accounting.get(&InvoiceType::Out).unwrap();

        let VatCalculationResult { tax, gross } = CurrencyValue::calculate_vat(&net, vat);
        assert!(ingoing.net.value.eq(&net.value));
        assert!(ingoing.tax.value.eq(&tax.value));
        assert!(ingoing.gross.value.eq(&gross.value));
        assert!(outgoing.net.value.eq(&default_currency_value()));
        assert!(outgoing.tax.value.eq(&default_currency_value()));
        assert!(outgoing.gross.value.eq(&default_currency_value()));
    }

    #[test]
    fn calculate_summary_in_out() {
        let net = CurrencyValue::new(225000);
        let vat = Vat::Twenty;
        let sheet = AccountingSheet {
            items: vec![
                accounting_item(
                    InvoiceType::In,
                    net.clone(),
                    vat,
                    Category(String::from("a")),
                ),
                accounting_item(
                    InvoiceType::Out,
                    net.clone(),
                    vat,
                    Category(String::from("a")),
                ),
            ],
            year: 2024,
            month: None,
            quarter: Some(Quarter::Q1),
        };

        let result = calculate_summary(&sheet);
        assert!(!result.categories.is_empty());
        assert!(
            result
                .categories
                .get(&Category(String::from("a")))
                .unwrap()
                .value
                .eq(&net.value)
        );
        let ingoing = result.accounting.get(&InvoiceType::In).unwrap();
        let outgoing = result.accounting.get(&InvoiceType::Out).unwrap();

        let VatCalculationResult { tax, gross } = CurrencyValue::calculate_vat(&net, vat);
        assert!(ingoing.net.value.eq(&net.value));
        assert!(ingoing.tax.value.eq(&tax.value));
        assert!(ingoing.gross.value.eq(&gross.value));
        assert!(outgoing.net.value.eq(&net.value));
        assert!(outgoing.tax.value.eq(&tax.value));
        assert!(outgoing.gross.value.eq(&gross.value));
    }

    #[test]
    fn calculate_summary_multiple() {
        let net = CurrencyValue::new(225000);
        let net_times_two = CurrencyValue::new(450000);
        let vat = Vat::Twenty;
        let sheet = AccountingSheet {
            items: vec![
                accounting_item(
                    InvoiceType::In,
                    net.clone(),
                    vat,
                    Category(String::from("a")),
                ),
                accounting_item(
                    InvoiceType::In,
                    net.clone(),
                    vat,
                    Category(String::from("a")),
                ),
            ],
            year: 2024,
            month: None,
            quarter: Some(Quarter::Q1),
        };

        let result = calculate_summary(&sheet);
        assert!(!result.categories.is_empty());
        assert!(
            result
                .categories
                .get(&Category(String::from("a")))
                .unwrap()
                .value
                .eq(&net_times_two.value)
        );
        let ingoing = result.accounting.get(&InvoiceType::In).unwrap();
        let outgoing = result.accounting.get(&InvoiceType::Out).unwrap();

        let VatCalculationResult { tax, gross } = CurrencyValue::calculate_vat(&net_times_two, vat);
        assert!(ingoing.net.value.eq(&net_times_two.value));
        assert!(ingoing.tax.value.eq(&tax.value));
        assert!(ingoing.gross.value.eq(&gross.value));
        assert!(outgoing.net.value.eq(&default_currency_value()));
        assert!(outgoing.tax.value.eq(&default_currency_value()));
        assert!(outgoing.gross.value.eq(&default_currency_value()));
    }

    #[test]
    fn calculate_summary_multiple_with_negative() {
        let net = CurrencyValue::new(225000);
        let vat = Vat::Twenty;
        let sheet = AccountingSheet {
            items: vec![
                accounting_item(
                    InvoiceType::In,
                    net.clone(),
                    vat,
                    Category(String::from("a")),
                ),
                accounting_item(
                    InvoiceType::In,
                    net.clone(),
                    vat,
                    Category(String::from("b")),
                ),
                accounting_item(
                    InvoiceType::In,
                    CurrencyValue::new(-225000),
                    vat,
                    Category(String::from("a")),
                ),
            ],
            year: 2024,
            month: None,
            quarter: Some(Quarter::Q1),
        };

        let result = calculate_summary(&sheet);
        assert!(!result.categories.is_empty());
        assert!(
            result
                .categories
                .get(&Category(String::from("a")))
                .unwrap()
                .value
                .eq(&default_currency_value())
        );
        let ingoing = result.accounting.get(&InvoiceType::In).unwrap();
        let outgoing = result.accounting.get(&InvoiceType::Out).unwrap();

        let VatCalculationResult { tax, gross } = CurrencyValue::calculate_vat(&net, vat);
        assert!(ingoing.net.value.eq(&net.value));
        assert!(ingoing.tax.value.eq(&tax.value));
        assert!(ingoing.gross.value.eq(&gross.value));
        assert!(outgoing.net.value.eq(&default_currency_value()));
        assert!(outgoing.tax.value.eq(&default_currency_value()));
        assert!(outgoing.gross.value.eq(&default_currency_value()));
    }
}
