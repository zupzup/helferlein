use std::{fs::File, io::BufWriter, path::Path};

use chrono::NaiveDate;
use printpdf::{Color, IndirectFontRef, Line, Mm, PdfDocument, PdfLayerReference, Point, Rgb};

use crate::{
    data::{
        currency::{default_currency_value, CurrencyValue, VatCalculationResult},
        Address, Invoice, InvoiceItem, ServicePeriod, Vat,
    },
    util::export::PT_TO_MM,
    GuiError, Messages, DATE_FORMAT,
};

use super::{
    get_text_width, FONT, FONT_SIZE, LINE_WIDTH, MARGIN, MAX_CHARS_CURRENCY, PADDING, ROW_HEIGHT,
    TABLE_LINE_HEIGHT,
};

pub const MAX_ITEMS: usize = 10;

const HEIGHT: Mm = Mm(297.0);
const WIDTH: Mm = Mm(210.0);
const LEFT: Mm = Mm(MARGIN);
const RIGHT: Mm = Mm(WIDTH.0 - MARGIN);
const TOP: Mm = Mm(HEIGHT.0 - MARGIN);
const BOTTOM: Mm = Mm(MARGIN);

const MAX_DIGITS_POS: i32 = 2;
const MAX_DIGITS_QTY: i32 = 3;
const MAX_CHARS_UNIT: i32 = 2;

// COL WIDTHS
const POS_WIDTH: Mm = Mm(10.0);
const DESC_WIDTH: Mm = Mm(61.0);
const QTY_WIDTH: Mm = Mm(12.0);
const UNIT_WIDTH: Mm = Mm(12.0);
const UNIT_PRICE_WIDTH: Mm = Mm(27.0);
const GAP_WIDTH: Mm = Mm(20.0);

#[derive(Debug, Clone)]
pub(crate) struct CreatePDFResult;

#[derive(Debug, Clone)]
pub(crate) struct SumData {
    pub(crate) net: CurrencyValue,
    pub(crate) tax: CurrencyValue,
    pub(crate) total: CurrencyValue,
}

pub(crate) fn create_invoice_pdf(
    file_name: &Path,
    invoice: &Invoice,
) -> Result<CreatePDFResult, GuiError> {
    if invoice.items.len() > MAX_ITEMS {
        return Err(GuiError::ExportFailed("Too many items - max 15".into()));
    }
    let title = "Invoice".to_string();
    let (doc, page1, layer) = PdfDocument::new(&title, WIDTH, HEIGHT, "layer");
    let mut font_reader = std::io::Cursor::new(FONT);
    let font = doc
        .add_external_font(&mut font_reader)
        .expect("font is available");

    let bold_font = doc
        .add_builtin_font(printpdf::BuiltinFont::HelveticaBold)
        .expect("font is available");

    let current_layer = doc.get_page(page1).get_layer(layer);
    current_layer.set_outline_color(Color::Rgb(Rgb::new(0.5, 0.5, 0.5, None)));
    current_layer.set_outline_thickness(LINE_WIDTH);
    current_layer.set_line_height(TABLE_LINE_HEIGHT.0);
    current_layer.set_font(&font, FONT_SIZE.0);

    let from_top = render_from(&invoice.from, &current_layer, &font, TOP);
    let to_top = render_to(&invoice.to, &current_layer, &font, from_top);
    let mt_top = render_metadata(
        &invoice.city,
        &invoice.date,
        &invoice.invoice_number,
        &invoice.service_period,
        &current_layer,
        &font,
        to_top,
    );
    let pre_top = render_pre(&invoice.pre_text, &current_layer, &font, &bold_font, mt_top);
    let items_top = render_items(&invoice.items, &current_layer, &font, &bold_font, pre_top);
    render_post(&invoice.post_text, &current_layer, &font, items_top);
    render_footer(
        &invoice.from,
        &invoice.bank_data,
        &current_layer,
        &font,
        Mm(BOTTOM.0 + 5.0 * ROW_HEIGHT + PADDING),
    );

    // SAVE (overwrites the file)
    doc.save(&mut BufWriter::new(
        File::create(file_name).map_err(|e| GuiError::ExportFailed(e.to_string()))?,
    ))
    .map_err(|e| GuiError::ExportFailed(e.to_string()))?;
    Ok(CreatePDFResult {})
}

fn calc_left(txt_width: f32) -> Mm {
    Mm(RIGHT.0 - PADDING - (txt_width * PT_TO_MM))
}

fn calc_top(top: Mm, from_top: f32) -> Mm {
    Mm(top.0 - from_top * ROW_HEIGHT + PADDING * from_top)
}

pub(crate) fn render_to(
    address: &Address,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) -> Mm {
    let mut from_top: f32 = 1.0;
    let name = &address.name.trim().to_owned();
    let addr = &address.postal_address.trim().to_owned();
    let zip_city = &format!(
        "{} {}",
        &address.zip.trim().to_owned(),
        &address.city.trim().to_owned()
    );
    let country = &address.country.trim().to_owned();
    let vat = &address.vat.trim().to_owned();
    let misc = &address.misc.trim().to_owned();

    layer.use_text(name, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);

    from_top += 1.0;
    layer.use_text(addr, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);

    from_top += 1.0;
    layer.use_text(zip_city, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);

    if !country.is_empty() {
        from_top += 1.0;
        layer.use_text(country, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
    }

    if !vat.is_empty() {
        from_top += 1.0;
        layer.use_text(vat, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
    }

    if !misc.is_empty() {
        misc.lines().enumerate().for_each(|l| {
            from_top += 1.0;
            layer.use_text(l.1, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
        });
    }
    // return bottom of text for next alignment
    from_top += 1.0;
    calc_top(top, from_top)
}

pub(crate) fn render_from(
    address: &Address,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) -> Mm {
    let name = &address.name.trim().to_owned();
    let addr = &address.postal_address.trim().to_owned();
    let zip_city = &format!(
        "{} {}",
        &address.zip.trim().to_owned(),
        &address.city.trim().to_owned()
    );
    let country = &address.country.trim().to_owned();

    layer.use_text(
        name,
        FONT_SIZE.0,
        calc_left(get_text_width(name)),
        top,
        font,
    );

    let mut from_top: f32 = 1.0;
    layer.use_text(
        addr,
        FONT_SIZE.0,
        calc_left(get_text_width(addr)),
        calc_top(top, from_top),
        font,
    );

    from_top += 1.0;
    layer.use_text(
        zip_city,
        FONT_SIZE.0,
        calc_left(get_text_width(zip_city)),
        calc_top(top, from_top),
        font,
    );

    if !country.is_empty() {
        from_top += 1.0;
        layer.use_text(
            country,
            FONT_SIZE.0,
            calc_left(get_text_width(country)),
            calc_top(top, from_top),
            font,
        );
    }

    // return bottom of text for next alignment
    from_top += 1.0;
    calc_top(top, from_top)
}

pub(crate) fn render_metadata(
    city: &str,
    date: &NaiveDate,
    invoice_number: &str,
    service_period: &ServicePeriod,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) -> Mm {
    let mut from_top: f32 = 2.0;
    let city_date = format!("{}, {}", city.trim(), date.format(DATE_FORMAT));
    layer.use_text(
        &city_date,
        FONT_SIZE.0,
        calc_left(get_text_width(&city_date)),
        calc_top(top, from_top),
        font,
    );
    let inv_nr = format!(
        "{}: {}",
        Messages::InvoiceNumberText.msg(),
        invoice_number.trim()
    );

    from_top += 1.0;
    layer.use_text(
        &inv_nr,
        FONT_SIZE.0,
        calc_left(get_text_width(&inv_nr)),
        calc_top(top, from_top),
        font,
    );

    let serv_period = format!(
        "{}: {} - {}",
        Messages::ServicePeriod.msg(),
        service_period.from.format(DATE_FORMAT),
        service_period.to.format(DATE_FORMAT)
    );
    from_top += 1.0;
    layer.use_text(
        &serv_period,
        FONT_SIZE.0,
        calc_left(get_text_width(&serv_period)),
        calc_top(top, from_top),
        font,
    );

    // return bottom of text for next alignment
    from_top += 1.0;
    calc_top(top, from_top)
}

pub(crate) fn render_pre(
    pre_text: &str,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
    top: Mm,
) -> Mm {
    let mut from_top: f32 = 1.0;
    layer.use_text(
        Messages::Invoice.msg(),
        FONT_SIZE.0 * 1.2,
        LEFT,
        calc_top(top, from_top),
        bold_font,
    );
    from_top += 1.0;
    if !pre_text.is_empty() {
        pre_text.lines().enumerate().for_each(|l| {
            from_top += 1.0;
            layer.use_text(l.1, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
        });
    }

    from_top += 2.0;
    // return bottom of text for next alignment
    calc_top(top, from_top)
}

// TABLE

// ------------------------------------------------------------
// | Pos | Description | Qty | Unit Price |          | Amount |
// ------------------------------------------------------------
// |     | Breaks at   |     |            |          |        |
// |     | x chars     |     |            |          |        |
// ------------------------------------------------------------
// |     |             |     |            |          |        |
// ------------------------------------------------------------
//                                        |      Net |        |
//                                        ---------------------
//                                        | 20 % VAT |        |
//                                        ---------------------
//                                        |    Total |        |
//                                        ---------------------
//                                        ---------------------
pub(crate) fn render_items(
    items: &[InvoiceItem],
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    bold_font: &IndirectFontRef,
    top: Mm,
) -> Mm {
    let mut from_top: f32 = 1.0;
    render_table_header(top, layer, bold_font);
    let mut to_add_for_lines = 0;
    let mut item_lines = 0;
    for (idx, item) in items.iter().enumerate() {
        to_add_for_lines = render_row(
            item,
            Mm(top.0 - ROW_HEIGHT - ((idx + to_add_for_lines) as f32 * ROW_HEIGHT)),
            layer,
            font,
        ) - 1;
        item_lines += to_add_for_lines + 1;
    }
    // start at item lines + 1
    let top_after_items = Mm(top.0 - ROW_HEIGHT * (item_lines + 1) as f32);
    from_top += 1.0;
    // render sum
    let sum_data = calculate_sum(items);
    render_sum(top_after_items, sum_data, layer, font);

    // return bottom of text for next alignment
    from_top += 1.0;
    calc_top(top_after_items, from_top)
}

fn render_table_header(top: Mm, layer: &PdfLayerReference, font: &IndirectFontRef) {
    let mut col_line_x = 0.0;
    // START OF ROW
    render_row_line(top, layer);
    render_col_line(LEFT, top, layer);
    // Pos
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Pos.msg(),
        layer,
        font,
    );
    col_line_x += POS_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Description
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Description.msg(),
        layer,
        font,
    );
    col_line_x += DESC_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Qty
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Qty.msg(),
        layer,
        font,
    );
    col_line_x += QTY_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Unit
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::UnitShort.msg(),
        layer,
        font,
    );
    col_line_x += UNIT_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Unit Price
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::PricePerUnit.msg(),
        layer,
        font,
    );
    col_line_x += UNIT_PRICE_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Gap
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        "",
        layer,
        font,
    );
    col_line_x += GAP_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    // Sum
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Sum.msg(),
        layer,
        font,
    );

    // Omit last col_line, since it's the row's col line

    // END OF ROW
    render_col_line(RIGHT, top, layer);
    render_row_line(Mm(top.0 - ROW_HEIGHT), layer);
}

fn render_row(
    item: &InvoiceItem,
    top: Mm,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
) -> usize {
    let mut col_line_x = 0.0;
    let lines = item.description.lines().count();
    // START OF ROW
    render_row_line(top, layer);
    render_col_line_with_multiplier(LEFT, top, lines, layer);
    // Pos
    let pos_str = item.nr.to_string();
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_DIGITS_POS - pos_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &pos_str,
        layer,
        font,
    );
    col_line_x += POS_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Description
    item.description.lines().enumerate().for_each(|(i, line)| {
        render_col_text(
            Mm(LEFT.0 + col_line_x + PADDING),
            Mm(top.0 - (ROW_HEIGHT * (i + 1) as f32) + PADDING),
            line,
            layer,
            font,
        );
    });
    col_line_x += DESC_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Qty
    let qty_str = item.amount.to_string();
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_DIGITS_QTY - qty_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &qty_str,
        layer,
        font,
    );
    col_line_x += QTY_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Unit
    let unit_str = item.unit.name();
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + (PADDING * 2.0)
            + ((MAX_CHARS_UNIT - unit_str.chars().count() as i32) as f32 * PADDING)),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        unit_str,
        layer,
        font,
    );
    col_line_x += UNIT_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Price per Unit
    let ppu_str = item.price_per_unit.to_euro_str();
    let pad_no_dot = if ppu_str.contains('.') { 0.0 } else { 1.0 }; // if val is < 1000
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + PADDING
            + ((MAX_CHARS_CURRENCY - ppu_str.chars().count() as i32) as f32 * PADDING)
            + pad_no_dot),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &ppu_str,
        layer,
        font,
    );
    col_line_x += UNIT_PRICE_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Gap
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        "",
        layer,
        font,
    );
    col_line_x += GAP_WIDTH.0;
    render_col_line_with_multiplier(Mm(LEFT.0 + col_line_x), top, lines, layer);
    // Sum
    let sum_str = CurrencyValue::new_from_decimal(
        item.price_per_unit
            .value
            .checked_mul(item.amount)
            .expect("mul works"),
    )
    .to_euro_str();
    let mut pad_no_dot = if sum_str.contains('.') { 0.0 } else { -1.0 }; // if val is < 1000
    if item.price_per_unit.value < default_currency_value() {
        // for negative numbers, pad
        pad_no_dot = 1.0;
    }
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + (PADDING * 2.0)
            + ((MAX_CHARS_CURRENCY - sum_str.chars().count() as i32) as f32 * PADDING)
            + pad_no_dot),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &sum_str,
        layer,
        font,
    );
    // Omit last col_line, since it's the row's col line

    // END OF ROW
    render_col_line_with_multiplier(RIGHT, top, lines, layer);
    render_row_line(Mm(top.0 - ROW_HEIGHT * lines as f32), layer);
    lines
}

fn render_sum(top: Mm, sum_data: SumData, layer: &PdfLayerReference, font: &IndirectFontRef) -> Mm {
    let mut col_line_x = 0.0;
    col_line_x += POS_WIDTH.0;
    col_line_x += DESC_WIDTH.0;
    col_line_x += QTY_WIDTH.0;
    col_line_x += UNIT_WIDTH.0;
    col_line_x += UNIT_PRICE_WIDTH.0;
    let line_from = col_line_x;

    // Net
    render_sum_line(Mm(LEFT.0 + line_from), top, layer);
    render_sum_line(Mm(LEFT.0 + line_from), Mm(top.0 + 0.1), layer);
    let col_line_x_left_line = col_line_x;
    render_col_line(Mm(LEFT.0 + col_line_x_left_line), top, layer);
    render_col_text(
        Mm(LEFT.0 + col_line_x + PADDING),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        Messages::Net.msg(),
        layer,
        font,
    );
    col_line_x += GAP_WIDTH.0;
    render_col_line(Mm(LEFT.0 + col_line_x), top, layer);
    let net_str = sum_data.net.to_euro_str();
    let pad_no_dot = if net_str.contains('.') { 0.0 } else { 1.0 }; // if val is < 1000
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + (PADDING * 2.0)
            + ((MAX_CHARS_CURRENCY - net_str.chars().count() as i32) as f32 * PADDING)
            + pad_no_dot),
        Mm(top.0 - ROW_HEIGHT + PADDING),
        &net_str,
        layer,
        font,
    );
    render_col_line(RIGHT, top, layer);
    render_sum_line(Mm(LEFT.0 + line_from), Mm(top.0 - ROW_HEIGHT), layer);
    // Tax
    render_col_line(
        Mm(LEFT.0 + col_line_x_left_line),
        Mm(top.0 - ROW_HEIGHT),
        layer,
    );
    render_col_text(
        Mm(LEFT.0 + col_line_x_left_line + PADDING),
        Mm(top.0 - (ROW_HEIGHT * 2.0) + PADDING),
        &format!("{} {}", Vat::Twenty.name(), Messages::Vat.msg()),
        layer,
        font,
    );
    render_col_line(Mm(LEFT.0 + col_line_x), Mm(top.0 - ROW_HEIGHT), layer);
    let tax_str = sum_data.tax.to_euro_str();
    let pad_no_dot = if tax_str.contains('.') { 0.0 } else { -1.0 }; // if val is < 1000
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + (PADDING * 2.0)
            + ((MAX_CHARS_CURRENCY - tax_str.chars().count() as i32) as f32 * PADDING)
            + pad_no_dot),
        Mm(top.0 - (ROW_HEIGHT * 2.0) + PADDING),
        &tax_str,
        layer,
        font,
    );
    render_sum_line(
        Mm(LEFT.0 + line_from),
        Mm(top.0 - (ROW_HEIGHT * 2.0)),
        layer,
    );
    render_sum_line(
        Mm(LEFT.0 + line_from),
        Mm(top.0 - (ROW_HEIGHT * 2.0) - 0.1),
        layer,
    );
    render_col_line(RIGHT, Mm(top.0 - ROW_HEIGHT), layer);
    // total
    render_col_line(
        Mm(LEFT.0 + col_line_x_left_line),
        Mm(top.0 - (ROW_HEIGHT * 2.0)),
        layer,
    );
    render_col_text(
        Mm(LEFT.0 + col_line_x_left_line + PADDING),
        Mm(top.0 - (ROW_HEIGHT * 3.0) + PADDING),
        Messages::Total.msg(),
        layer,
        font,
    );
    render_col_line(
        Mm(LEFT.0 + col_line_x),
        Mm(top.0 - (ROW_HEIGHT * 2.0)),
        layer,
    );
    let total_string = sum_data.total.to_euro_str();
    let pad_no_dot = if total_string.contains('.') { 0.0 } else { 1.0 }; // if val is < 1000
    render_col_text(
        // right-align
        Mm(LEFT.0
            + col_line_x
            + (PADDING * 2.0)
            + ((MAX_CHARS_CURRENCY - total_string.chars().count() as i32) as f32 * PADDING)
            + pad_no_dot),
        Mm(top.0 - (ROW_HEIGHT * 3.0) + PADDING),
        &total_string,
        layer,
        font,
    );
    render_sum_line(
        Mm(LEFT.0 + line_from),
        Mm(top.0 - (ROW_HEIGHT * 3.0)),
        layer,
    );
    render_sum_line(
        Mm(LEFT.0 + line_from),
        Mm(top.0 - (ROW_HEIGHT * 3.0) + 0.5),
        layer,
    );
    render_col_line(RIGHT, Mm(top.0 - (ROW_HEIGHT * 2.0)), layer);

    top
}

fn render_row_line(y: Mm, layer: &PdfLayerReference) {
    let line = Line {
        points: vec![(Point::new(LEFT, y), false), (Point::new(RIGHT, y), false)],
        is_closed: true,
    };

    layer.add_line(line);
}

fn render_sum_line(x: Mm, y: Mm, layer: &PdfLayerReference) {
    let line = Line {
        points: vec![(Point::new(x, y), false), (Point::new(RIGHT, y), false)],
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

fn render_col_line_with_multiplier(x: Mm, y: Mm, multiplier: usize, layer: &PdfLayerReference) {
    let line = Line {
        points: vec![
            (Point::new(x, y), false),
            (
                Point::new(x, Mm(y.0 - ROW_HEIGHT * multiplier as f32)),
                false,
            ),
        ],
        is_closed: true,
    };

    layer.add_line(line);
}

pub(crate) fn render_post(
    post_text: &str,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) {
    let mut from_top: f32 = 2.0;
    if !post_text.is_empty() {
        post_text.lines().enumerate().for_each(|l| {
            from_top += 1.0;
            layer.use_text(l.1, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
        });
    }
}

pub(crate) fn render_footer(
    address: &Address,
    bank_data: &str,
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    top: Mm,
) {
    let line = Line {
        points: vec![
            (Point::new(LEFT, top), false),
            (Point::new(RIGHT, top), false),
        ],
        is_closed: true,
    };

    layer.add_line(line);
    let mut from_top: f32 = 2.0;
    let name = &address.name.trim().to_owned();
    let addr = &format!(
        "{}, {} {}",
        &address.postal_address.trim().to_owned(),
        &address.zip.trim().to_owned(),
        &address.city.trim().to_owned()
    );
    layer.use_text(name, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);

    from_top += 1.0;
    layer.use_text(
        addr,
        FONT_SIZE.0,
        LEFT,
        Mm(top.0 - from_top * ROW_HEIGHT + PADDING * from_top),
        font,
    );

    let vat = &address.vat.trim().to_owned();
    let misc = &address.misc.trim().to_owned();

    if !vat.is_empty() {
        from_top += 1.0;
        layer.use_text(vat, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
    }

    if !misc.is_empty() {
        misc.lines().enumerate().for_each(|l| {
            from_top += 1.0;
            layer.use_text(l.1, FONT_SIZE.0, LEFT, calc_top(top, from_top), font);
        });
    }

    if !bank_data.is_empty() {
        let mut from_top = 1.0;
        bank_data.lines().enumerate().for_each(|l| {
            from_top += 1.0;
            layer.use_text(
                l.1,
                FONT_SIZE.0,
                calc_left(get_text_width(l.1)),
                calc_top(top, from_top),
                font,
            );
        });
    }
}

fn calculate_sum(items: &[InvoiceItem]) -> SumData {
    let mut net_sum = default_currency_value();
    let mut tax_sum = default_currency_value();
    let mut total_sum = default_currency_value();

    items.iter().for_each(|item| {
        let net = item
            .price_per_unit
            .value
            .checked_mul(item.amount)
            .unwrap_or_else(default_currency_value);
        let VatCalculationResult { tax, gross } =
            CurrencyValue::new_from_decimal(net).calculate_vat(item.vat);
        net_sum = net_sum.checked_add(net).unwrap_or(default_currency_value());
        tax_sum = tax_sum
            .checked_add(tax.value)
            .unwrap_or(default_currency_value());
        total_sum = total_sum
            .checked_add(gross.value)
            .unwrap_or(default_currency_value());
    });

    SumData {
        net: CurrencyValue::new_from_decimal(net_sum),
        tax: CurrencyValue::new_from_decimal(tax_sum),
        total: CurrencyValue::new_from_decimal(total_sum),
    }
}
