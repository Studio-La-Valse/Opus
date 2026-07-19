use crate::bounding_box::BoundingBox;
use crate::xy::XY;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SmuflMetadata {
    #[serde(rename = "glyphBBoxes")]
    pub glyph_boxes: HashMap<String, GlyphBoundingBox>,

    #[serde(rename = "glyphsWithAnchors")]
    pub glyph_anchors: HashMap<String, GlyphAnchors>,

    #[serde(rename = "glyphsWithAlternates")]
    pub glyph_alternatives: HashMap<String, Alternates>,
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

    #[serde(rename = "stemDownNW")]
    pub stem_down_nw: Option<[f32; 2]>,

    #[serde(rename = "stemDownSW")]
    pub stem_down_sw: Option<[f32; 2]>,

    #[serde(rename = "stemUpSE")]
    pub stem_up_se: Option<[f32; 2]>,

    #[serde(rename = "stemUpNW")]
    pub stem_up_nw: Option<[f32; 2]>,
}

impl GlyphAnchors {
    pub fn to_cutouts(&self, bbox: &BoundingBox) -> Cutouts {
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

    pub fn stem_down_nw(&self) -> Option<XY> {
        self.stem_down_nw.map(map_arr)
    }

    pub fn stem_down_sw(&self) -> Option<XY> {
        self.stem_down_sw.map(map_arr)
    }

    pub fn stem_up_se(&self) -> Option<XY> {
        self.stem_up_se.map(map_arr)
    }

    pub fn stem_up_nw(&self) -> Option<XY> {
        self.stem_up_nw.map(map_arr)
    }
}

fn map_arr(v: [f32; 2]) -> XY {
    XY { x: v[0], y: -v[1] }
}

#[derive(Debug, Clone)]
pub struct Cutouts {
    pub nw: Option<BoundingBox>,
    pub ne: Option<BoundingBox>,
    pub sw: Option<BoundingBox>,
    pub se: Option<BoundingBox>,
}

#[derive(Debug, Deserialize)]
pub struct Alternates {
    pub alternates: Vec<Alternate>,
}

#[derive(Debug, Deserialize)]
pub struct Alternate {
    pub codepoint: String,
    pub name: String,
}
