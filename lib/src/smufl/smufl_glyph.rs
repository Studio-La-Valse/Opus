use crate::color::Color;
use crate::drawable::elements::text::Text;
use crate::xy::XY;

pub trait SmuflGlyph {
    fn as_text(&self, color: Color, xy: XY) -> Text;
}
