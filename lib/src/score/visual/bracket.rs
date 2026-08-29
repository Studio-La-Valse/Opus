use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};

#[derive(Clone)]
pub struct Bracket {
    pub xy: XY,
    pub height: f32,

    pub bracket_top: BracketTop,
    pub bracket_bottom: BracketBottom,

    pub color: Color,
    pub scale: f32,
}

impl Bracket {
    pub fn new(bracket_top: BracketTop, bracket_bottom: BracketBottom) -> Self {
        Self {
            xy: Default::default(),
            height: Default::default(),

            bracket_top,
            bracket_bottom,

            color: Color::BLACK,
            scale: 1.0,
        }
    }
}

impl Bracket {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Bracket {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
