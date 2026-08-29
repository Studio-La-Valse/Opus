use crate::drawable::layoutable::Layoutable;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score_element::ScoreElement;
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

impl ScoreElement for Bracket {
    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Bracket {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
