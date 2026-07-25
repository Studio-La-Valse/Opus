use crate::color::Color;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::smufl::glyphs::number::Number;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::score_element::ScoreElement;
use crate::xy::XY;

pub struct TimeSignature {
    pub xy: XY,
    pub height: f32,
    pub width: f32,

    pub color: Color,

    pub num: Number,
    pub denom: Number,
}

impl TimeSignature {
    pub fn new(num: Number, denom: Number) -> TimeSignature {
        TimeSignature {
            xy: XY::ZERO,
            height: 0.,
            width: 0.,
            color: Color::BLACK,
            num,
            denom,
        }
    }
}

impl ScoreElement for TimeSignature {}

impl Layoutable for TimeSignature {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
        self.width = 25.;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Drawable for TimeSignature {
    fn content(&self) -> Vec<&dyn Drawable> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut result: Vec<DrawableElement> = vec![];

        let pos_num = self.xy.mv(0., self.height / 4.);
        result.push(self.num.as_text(self.color, pos_num, 1.).into());

        let pos_denom = self.xy.mv(0., self.height / 4. * 3.);
        result.push(self.denom.as_text(self.color, pos_denom, 1.).into());

        result
    }
}
