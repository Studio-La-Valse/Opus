use crate::drawable::elements::text::{FontSpec, HorizontalAlign, Text, VerticalAlign};
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::SmuflGlyph;

/// One digit of a [`Number`].
#[derive(Clone)]
pub struct NumberDigit {
    pub codepoint: char,
    pub bbox: BoundingBox,
    /// Advance width in staff spaces -- how far to step before the next digit.
    /// See [`SmuflMetadata::glyph_advance_widths`](crate::smufl::smufl_metadata::SmuflMetadata).
    pub advance: f32,
}

impl SmuflGlyph for NumberDigit {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a> {
        Text {
            vertical_alignment: VerticalAlign::Bottom,
            horizontal_alignment: HorizontalAlign::Left,
            font_size: (Staff::DEFAULT_SPACE_SIZE * scale) * Staff::SPACES as f32,
            font: FontSpec::plain(&font.meta.font),
            text: font.glyph_str(self.codepoint),
            xy,
            color,
        }
    }
}

/// A number set in the music font, as the sequence of digit glyphs that spell
/// it: `12` is `timeSig1` followed by `timeSig2`.
///
/// SMuFL only defines `timeSig0` through `timeSig9`, so anything from 10 upward
/// -- every `x/16`, `x/32` and `x/64` signature, and numerators like 12 -- has
/// to be built out of several glyphs rather than looked up whole.
#[derive(Clone)]
pub struct Number {
    /// Most significant digit first. Never empty.
    pub digits: Vec<NumberDigit>,
}

impl Number {
    /// Total advance in staff spaces: what the number occupies horizontally,
    /// including the side bearing past the last digit's ink.
    pub fn advance(&self) -> f32 {
        self.digits.iter().map(|digit| digit.advance).sum()
    }

    /// Each digit paired with its x offset from the number's own origin, in
    /// staff spaces. Stepping by advance rather than by bounding-box width is
    /// what keeps the side bearings between digits.
    pub fn placed(&self) -> impl Iterator<Item = (&NumberDigit, f32)> {
        let mut offset = 0.;
        self.digits.iter().map(move |digit| {
            let at = offset;
            offset += digit.advance;
            (digit, at)
        })
    }
}
