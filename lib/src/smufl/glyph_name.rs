use serde::Deserialize;
use std::collections::HashMap;
use std::sync::OnceLock;

/// The SMuFL glyph-name table: every glyph the standard defines, by name, with
/// its codepoint.
///
/// Embedded rather than read at run time because it describes the SMuFL
/// standard, not any one font, and the specification names no place it is
/// installed. Every font's metadata keys its glyphs by these names, so which
/// font is chosen never changes this table.
pub fn glyph_names() -> &'static HashMap<String, GlyphName> {
    static GLYPH_NAMES: OnceLock<HashMap<String, GlyphName>> = OnceLock::new();
    GLYPH_NAMES.get_or_init(|| {
        serde_json::from_str(GLYPH_NAMES_JSON).expect("embedded glyphnames.json is invalid")
    })
}

#[derive(Deserialize)]
pub struct GlyphName {
    pub codepoint: String,
    pub description: String,
}

pub trait ToChar {
    fn codepoint_char(&self) -> char;
}

impl ToChar for String {
    fn codepoint_char(&self) -> char {
        let hex = self.trim_start_matches("U+");
        let value = u32::from_str_radix(hex, 16).unwrap();
        char::from_u32(value).unwrap()
    }
}

impl ToChar for GlyphName {
    fn codepoint_char(&self) -> char {
        self.codepoint.codepoint_char()
    }
}

const GLYPH_NAMES_JSON: &str = include_str!("../../../assets/smufl/metadata/glyphnames.json");
