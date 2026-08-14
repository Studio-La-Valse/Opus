use serde::Deserialize;
use std::collections::HashMap;

pub fn load_glyph_names(json_content: &str) -> HashMap<String, GlyphName> {
    let res = serde_json::from_str(json_content);
    res.expect("Invalid glyphnames.json content")
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
