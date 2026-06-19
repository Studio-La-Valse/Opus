use std::collections::HashMap;
use serde::Deserialize;
use crate::smufl::bounding_box_metadata::BoundingBoxMetadata;

#[derive(Debug, Deserialize)]
pub struct SmuflMetadata {
    #[serde(rename = "glyphBBoxes")]
    pub glyph_bboxes: HashMap<String, BoundingBoxMetadata>,
}
