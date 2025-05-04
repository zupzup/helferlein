use azul_text_layout::{
    text_layout::{split_text_into_words, words_to_scaled_words},
    text_shaping::get_font_metrics_freetype,
};
use printpdf::Pt;

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
    let space_count = text.chars().filter(|&c| c == ' ').count();
    let font_index: i32 = 0;
    let font_metrics = get_font_metrics_freetype(FONT, font_index);
    let words = split_text_into_words(text);
    // Use pt in pdf as px and assume 72 DPI
    let scaled_words =
        words_to_scaled_words(&words, FONT, font_index as u32, font_metrics, FONT_SIZE.0);

    let total_width: f32 = scaled_words.items.iter().map(|i| i.word_width).sum();
    let space_width: f32 = space_count as f32 * 2.78;
    total_width + space_width
}
