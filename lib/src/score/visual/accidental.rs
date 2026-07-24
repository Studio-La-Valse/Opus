use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::smufl::glyphs::accidental::Accidental as SmuflAccidental;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::score_element::ScoreElement;
use crate::xy::XY;

pub struct Accidental {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
    pub color: Color,

    pub accidental: SmuflAccidental,
}

impl Accidental {
    pub fn new(accidental: SmuflAccidental) -> Self {
        Self {
            xy: XY::ZERO,
            width: 10.,
            height: 10.,

            scale: 1.,
            color: Color::BLACK,

            accidental,
        }
    }

    pub fn bounding_box(&self) -> BoundingBox {
        BoundingBox {
            x_min: self.xy.x,
            y_min: self.xy.y,
            x_max: self.xy.x + self.width,
            y_max: self.xy.y + self.height,
        }
    }
}

impl ScoreElement for Accidental {}

impl Layoutable for Accidental {
    fn measure(&mut self, _available: &XY) {
        // todo: Measure from bounding box
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl DrawableContent for Accidental {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![
            self.accidental
                .as_text(self.color, self.xy, self.scale)
                .into(),
        ]
    }
}
