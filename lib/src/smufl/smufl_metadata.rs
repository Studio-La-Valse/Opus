use crate::smufl::glyph_boxes::GlyphBoxes;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SmuflMetadata {
    #[serde(rename = "glyphBBoxes")]
    pub glyph_boxes: GlyphBoxes,
}
