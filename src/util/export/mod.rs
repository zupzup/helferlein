use printpdf::Pt;
use ttf_parser::Face;

pub(crate) mod accounting;
pub(crate) mod invoice;

const FONT: &[u8] = include_bytes!("../../Helvetica.ttf");
const PT_TO_MM: f32 = 0.352_778_f32;
const MARGIN: f32 = 20.0;
const TABLE_LINE_HEIGHT: Pt = Pt(7.5); // pt
const FONT_SIZE: Pt = Pt(10.0); // pt
const PADDING: f32 = 2.0; // Mm
const LINE_WIDTH: f32 = 0.0; // 1 px everywhere
const ROW_HEIGHT: f32 = (TABLE_LINE_HEIGHT.0 * PT_TO_MM) + 2.0 * PADDING; // Mm
const MAX_CHARS_VAT: i32 = 4;
const MAX_CHARS_CURRENCY: i32 = 12;

fn get_text_width(text: &str) -> f32 {
    if text.is_empty() {
        return 0.0;
    }

    let Ok(face) = Face::parse(FONT, 0) else {
        return 0.0;
    };

    let units_per_em = face.units_per_em() as f32;
    let font_units = text
        .chars()
        .filter_map(|c| {
            face.glyph_index(c)
                .and_then(|glyph_id| face.glyph_hor_advance(glyph_id))
        })
        .map(f32::from)
        .sum::<f32>();

    font_units * FONT_SIZE.0 / units_per_em
}
