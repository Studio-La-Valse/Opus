use crate::color::Color;
use crate::drawable::layoutable::Layoutable;
use crate::smufl::glyphs::number::Number;
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
