use serde::Deserialize;
use std::collections::HashMap;
use std::fs;

pub fn load_glyph_names(json: &str) -> HashMap<String, GlyphName> {
    let raw = fs::read_to_string(json).expect("Failed to read file");

    println!("RAW START: {:?}", &raw[..raw.len().min(50)]);

    let res = serde_json::from_str(&raw);
    res.expect("Invalid glyphnames.json file")
}

#[derive(Deserialize)]
pub struct GlyphName {
    pub codepoint: String,
    pub description: String,
}

impl GlyphName {
    pub fn codepoint_char(&self) -> char {
        let hex = self.codepoint.trim_start_matches("U+");
        let value = u32::from_str_radix(hex, 16).unwrap();
        char::from_u32(value).unwrap()
    }
}
