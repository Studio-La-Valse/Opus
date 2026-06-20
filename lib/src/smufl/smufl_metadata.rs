use crate::bounding_box::BoundingBox;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SmuflMetadata {
    #[serde(rename = "glyphBBoxes")]
    pub glyph_boxes: GlyphBoundingBoxes,

    #[serde(rename = "glyphsWithAnchors")]
    pub glyph_anchors: GlyphsWithAnchors,
}

#[derive(Debug, Deserialize)]
pub struct GlyphBoundingBoxes {
    #[serde(rename = "noteheadBlack")]
    pub notehead_black: GlyphBoundingBox,
}

#[derive(Debug, Deserialize)]
pub struct GlyphBoundingBox {
    #[serde(rename = "bBoxNE")]
    pub ne: [f32; 2],
    #[serde(rename = "bBoxSW")]
    pub sw: [f32; 2],
}

impl From<&GlyphBoundingBox> for BoundingBox {
    fn from(value: &GlyphBoundingBox) -> Self {
        BoundingBox {
            x_min: value.sw[0],
            y_min: -value.ne[1],
            x_max: value.ne[0],
            y_max: -value.sw[1],
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize)]
pub struct GlyphsWithAnchors {
    #[serde(rename = "noteheadBlack")]
    pub notehead_black: GlyphAnchors,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GlyphAnchors {
    #[serde(rename = "cutOutNW")]
    pub nw: Option<[f32; 2]>,

    #[serde(rename = "cutOutNE")]
    pub ne: Option<[f32; 2]>,

    #[serde(rename = "cutOutSE")]
    pub se: Option<[f32; 2]>,

    #[serde(rename = "cutOutSW")]
    pub sw: Option<[f32; 2]>,
}

impl GlyphAnchors {
    pub fn to_boxes(&self, bbox: &BoundingBox) -> Cutouts {
        Cutouts {
            nw: self.nw.map(|nw| BoundingBox {
                x_min: bbox.x_min,
                y_min: bbox.y_min,
                x_max: nw[0],
                y_max: -nw[1],
            }),
            ne: self.ne.map(|ne| BoundingBox {
                x_min: ne[0],
                y_min: bbox.y_min,
                x_max: bbox.x_max,
                y_max: -ne[1],
            }),
            se: self.se.map(|se| BoundingBox {
                x_min: se[0],
                y_min: -se[1],
                x_max: bbox.x_max,
                y_max: bbox.y_max,
            }),
            sw: self.sw.map(|sw| BoundingBox {
                x_min: bbox.x_min,
                y_min: -sw[1],
                x_max: sw[0],
                y_max: bbox.y_max,
            }),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Cutouts {
    pub nw: Option<BoundingBox>,
    pub ne: Option<BoundingBox>,
    pub sw: Option<BoundingBox>,
    pub se: Option<BoundingBox>,
}
