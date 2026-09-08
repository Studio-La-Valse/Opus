use crate::drawable::drawable_element::Scale;
use crate::drawable::elements::text::FontSpec;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

/// A single glyph from a font that ships its own metrics -- in practice a SMuFL
/// music glyph.
///
/// Deliberately not a [`Text`](super::text::Text). A text element is a run of
/// characters whose extent depends on metrics only the sink has: the browser
/// resolving a font family, the PDF writer's embedded face. A glyph's ink box
/// comes from the font metadata this crate already parses, so it is known here,
/// exactly, with no font file and no canvas involved -- [`bounds`](Self::bounds)
/// is not a hint to be reconciled with anything, it *is* the glyph's geometry.
///
/// That is also why there are no alignment fields. A text element needs them
/// because its anchor is the only handle it has on an extent it cannot measure;
/// a glyph is placed by its [`origin`](Self::origin) -- the SMuFL registration
/// point, on the baseline at the left of the advance width -- and a producer
/// that wants the glyph somewhere else moves the origin, rather than asking the
/// sink to shift by an advance the sink would have to look up.
#[derive(Serialize, Clone)]
pub struct Glyph<'a> {
    /// The glyph's codepoint, as the one-character string a sink draws.
    pub glyph: &'a str,
    pub color: Color,
    pub font_size: f32,
    pub font: FontSpec<'a>,
    /// The glyph origin, in world space: on the baseline, at the left of the
    /// advance width. Sinks draw from here.
    pub origin: XY,
    /// The box the glyph's ink occupies in world space, from the font metadata.
    pub bounds: BoundingBox,
}

impl<'a> Scale for Glyph<'a> {
    fn scale(&self, factor: f32, pivot: XY) -> Glyph<'a> {
        Glyph {
            origin: self.origin.scale_about(factor, pivot),
            font_size: self.font_size * factor,
            bounds: self.bounds.scale_about(factor, pivot),
            ..self.clone()
        }
    }
}
