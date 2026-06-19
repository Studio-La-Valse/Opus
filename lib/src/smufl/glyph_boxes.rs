use crate::smufl::bounding_box_metadata::BoundingBoxMetadata;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GlyphBoxes {
    #[serde(rename = "noteheadBlack")]
    pub notehead_black: BoundingBoxMetadata,
}
