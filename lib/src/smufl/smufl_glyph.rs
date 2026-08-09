use crate::color::Color;
use crate::drawable::elements::text::Text;
use crate::smufl::smufl_font::SmuflFont;
use crate::xy::XY;

pub trait SmuflGlyph {
    fn as_text<'a>(&self, font: &'a SmuflFont, color: Color, xy: XY, scale: f32) -> Text<'a>;
}
