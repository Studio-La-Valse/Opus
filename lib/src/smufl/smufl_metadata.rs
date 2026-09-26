use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::xy::XY;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SmuflMetadata {
    #[serde(rename = "fontName")]
    pub font: String,

    #[serde(rename = "glyphBBoxes")]
    pub glyph_boxes: HashMap<String, GlyphBoundingBox>,

    /// How far the pen moves after drawing each glyph, in staff spaces. Wider
    /// than the glyph's own bounding box by its side bearings, which is what
    /// makes it the right step between two glyphs set next to each other -- a
    /// multi-digit time signature is the only place this crate does that today.
    ///
    /// Optional in the SMuFL spec, so a font whose metadata omits it still
    /// loads; callers fall back to the bounding-box width.
    #[serde(rename = "glyphAdvanceWidths", default)]
    pub glyph_advance_widths: HashMap<String, f32>,

    #[serde(rename = "glyphsWithAnchors")]
    pub glyph_anchors: HashMap<String, GlyphAnchors>,

    #[serde(rename = "glyphsWithAlternates")]
    pub glyph_alternatives: HashMap<String, Alternates>,

    /// The font designer's recommended line thicknesses. Optional in the SMuFL
    /// spec, so a font whose metadata omits the block still loads.
    #[serde(rename = "engravingDefaults", default)]
    pub engraving_defaults: EngravingDefaults,
}

/// The `engravingDefaults` keys this crate reads, verbatim, in staff spaces.
///
/// Only the keys some layout option is resolved from; the spec defines more.
/// Every key is optional, per the spec: an absent one falls through to
/// [`APP_DEFAULTS`](crate::score::layout_options::APP_DEFAULTS). Turned into a
/// layout tier by
/// [`UserLayout::from_engraving_defaults`](crate::score::layout_options::UserLayout::from_engraving_defaults).
#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngravingDefaults {
    pub staff_line_thickness: Option<f32>,
    pub leger_line_thickness: Option<f32>,
    pub thin_barline_thickness: Option<f32>,
    pub thick_barline_thickness: Option<f32>,
    pub beam_thickness: Option<f32>,
    pub beam_spacing: Option<f32>,
    pub stem_thickness: Option<f32>,
    pub tie_endpoint_thickness: Option<f32>,
    pub tie_midpoint_thickness: Option<f32>,
    pub bracket_thickness: Option<f32>,
    pub sub_bracket_thickness: Option<f32>,
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
            xy: XY {
                x: value.sw[0],
                y: -value.ne[1],
            },
            size: XY {
                x: value.ne[0] - value.sw[0],
                y: -value.sw[1] - -value.ne[1],
            },
        }
    }
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct GlyphAnchors {
    #[serde(rename = "cutOutNW")]
    pub cutout_nw: Option<[f32; 2]>,

    #[serde(rename = "cutOutNE")]
    pub cutout_ne: Option<[f32; 2]>,

    #[serde(rename = "cutOutSE")]
    pub cutout_se: Option<[f32; 2]>,

    #[serde(rename = "cutOutSW")]
    pub cutout_sw: Option<[f32; 2]>,

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
            nw: self.cutout_nw.map(|nw| BoundingBox {
                xy: bbox.xy,
                size: XY {
                    x: nw[0] - bbox.xy.x,
                    y: -nw[1] - bbox.xy.y,
                },
            }),
            ne: self.cutout_ne.map(|ne| BoundingBox {
                xy: XY {
                    x: ne[0],
                    y: bbox.xy.y,
                },
                size: XY {
                    x: bbox.x_max() - ne[0],
                    y: -ne[1] - bbox.xy.y,
                },
            }),
            se: self.cutout_se.map(|se| BoundingBox {
                xy: XY {
                    x: se[0],
                    y: -se[1],
                },
                size: XY {
                    x: bbox.x_max() - se[0],
                    y: bbox.y_max() - -se[1],
                },
            }),
            sw: self.cutout_sw.map(|sw| BoundingBox {
                xy: XY {
                    x: bbox.xy.x,
                    y: -sw[1],
                },
                size: XY {
                    x: sw[0] - bbox.xy.x,
                    y: bbox.y_max() - -sw[1],
                },
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
