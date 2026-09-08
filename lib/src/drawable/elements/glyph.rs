use crate::drawable::drawable_element::Scale;
use crate::drawable::elements::text::FontSpec;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

/// A single glyph from a font that ships its own metrics -- in practice a SMuFL
/// music glyph.
///
/// Deliberately not a [`Text`](super::text::Text). A text element reserves a
/// layout box and lays its run out inside it, because what the run actually
/// covers depends on metrics only the sink has. A glyph is the other way round:
/// its box comes from the font metadata this crate already parses, so it is
/// *measured*, exactly, with no font file and no canvas involved -- and its
/// position is the origin that metadata registers it against.
///
/// That is also why it has no alignment fields. A glyph is drawn from its
/// [`origin`](Self::origin) -- on the baseline, at the left of the advance width
/// -- and a producer wanting it elsewhere moves the origin, rather than asking
/// the sink to shift by an advance the sink would have to look up.
///
/// The origin and the box are two views of one placement, so neither is a field
/// you set: [`new`](Self::new) derives both from the metadata box and the unit
/// it is normalized to, and they are readable but not writable. Nothing can put
/// a glyph's ink somewhere its origin is not.
#[derive(Serialize, Clone)]
pub struct Glyph<'a> {
    /// The glyph's codepoint, as the one-character string a sink draws.
    pub glyph: &'a str,
    pub color: Color,
    pub font_size: f32,
    pub font: FontSpec<'a>,
    /// Invariant, maintained by [`new`](Self::new) and by [`Scale`]: `bounds` is
    /// `origin` with the font's normalized box hung off it. Private so that it
    /// stays that way -- a public `origin` could be moved out from under the
    /// box it was measured against.
    origin: XY,
    bounds: BoundingBox,
}

impl<'a> Glyph<'a> {
    /// A glyph placed with its origin at `origin`.
    ///
    /// `bbox` is the box the font metadata gives the glyph: measured from the
    /// glyph origin and normalized, with `unit` world units to one normalized
    /// one. Taking the box in the form the metadata states it, rather than
    /// already placed, is what makes an origin and a box that disagree
    /// unconstructible.
    pub fn new(
        glyph: &'a str,
        font: FontSpec<'a>,
        font_size: f32,
        color: Color,
        origin: XY,
        bbox: &BoundingBox,
        unit: f32,
    ) -> Glyph<'a> {
        Glyph {
            glyph,
            font,
            font_size,
            color,
            origin,
            bounds: bbox.placed(origin, unit),
        }
    }

    /// Where the glyph is drawn from: the font's registration point, on the
    /// baseline at the left of the advance width.
    pub fn origin(&self) -> XY {
        self.origin
    }

    /// The box the glyph's ink occupies in world space.
    pub fn bounds(&self) -> BoundingBox {
        self.bounds
    }
}

impl<'a> Scale for Glyph<'a> {
    fn scale(&self, factor: f32, pivot: XY) -> Glyph<'a> {
        // Scaling both about the same pivot by the same factor is what keeps
        // them in step: the box stays exactly where the origin puts it.
        Glyph {
            origin: self.origin.scale_about(factor, pivot),
            bounds: self.bounds.scale_about(factor, pivot),
            font_size: self.font_size * factor,
            ..self.clone()
        }
    }
}
