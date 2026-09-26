use crate::drawable::elements::glyph::Glyph;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::smufl::smufl_font::SmuflFont;
use crate::smufl::smufl_glyph::{SmuflGlyph, placed_glyph, staff_space};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// Which of SMuFL's brace glyphs is drawn: the plain `brace`, or one of the
/// optional alternates a font may list under `glyphsWithAlternates` --
/// `braceSmall`, `braceLarge`, `braceLarger`, `braceFlat`.
///
/// An enum rather than a free-text glyph name so that a misspelt style is a
/// parse error, not a lookup that fails mid-render. Spelled in lower case
/// without the `brace` prefix, the same way in a CLI flag, a layout file, a JSON
/// option and a CSS custom property.
///
/// `Serialize` is derived only so
/// [`UserLayout`](crate::score::layout_options::UserLayout) can derive it in
/// turn.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(try_from = "String")]
pub enum BraceStyle {
    Default,
    Small,
    Large,
    Larger,
    Flat,
}

impl BraceStyle {
    /// The SMuFL name of the alternate glyph this style asks for, or `None` for
    /// the plain `brace`.
    pub fn alternate_name(&self) -> Option<&'static str> {
        match self {
            BraceStyle::Default => None,
            BraceStyle::Small => Some("braceSmall"),
            BraceStyle::Large => Some("braceLarge"),
            BraceStyle::Larger => Some("braceLarger"),
            BraceStyle::Flat => Some("braceFlat"),
        }
    }
}

#[derive(Debug)]
pub struct BraceStyleParseError(String);

impl fmt::Display for BraceStyleParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid brace style '{}': expected 'default', 'small', 'large', 'larger' or 'flat'",
            self.0
        )
    }
}

impl std::error::Error for BraceStyleParseError {}

impl FromStr for BraceStyle {
    type Err = BraceStyleParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "default" => Ok(BraceStyle::Default),
            "small" => Ok(BraceStyle::Small),
            "large" => Ok(BraceStyle::Large),
            "larger" => Ok(BraceStyle::Larger),
            "flat" => Ok(BraceStyle::Flat),
            _ => Err(BraceStyleParseError(s.to_string())),
        }
    }
}

/// Backs `#[serde(try_from = "String")]` on [`BraceStyle`].
impl TryFrom<String> for BraceStyle {
    type Error = BraceStyleParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

#[derive(Clone)]
pub struct Brace {
    pub codepoint: char,
    pub bbox: BoundingBox,
    /// Advance width in staff spaces. The brace is the one glyph placed by its
    /// right edge rather than its origin, so this is what that placement steps
    /// back by.
    pub advance: f32,
}

impl SmuflGlyph for Brace {
    /// Unlike every other glyph, `xy` is the brace's *right* edge: a brace hangs
    /// to the left of the system it braces, and the system's left edge is what
    /// the layout knows. Stepping back one advance width turns that into the
    /// origin -- which is what the element used to ask the sink to do by
    /// right-aligning the text, back when a glyph was drawn as text.
    fn as_glyph<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Glyph<'a> {
        let origin = xy.mv(-self.advance * staff_space(scale), 0.);

        placed_glyph(font, self.codepoint, &self.bbox, color, origin, scale)
    }
}
