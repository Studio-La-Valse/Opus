use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::smufl::glyphs::brace::Brace as SmuflBrace;
use crate::user_layout::UserLayout;
use crate::visual::score_element::ScoreElement;
use crate::xy::XY;

#[derive(Clone)]
pub struct Brace {
    pub xy: XY,
    pub height: f32,

    pub color: Color,

    pub brace: SmuflBrace,
}

impl Brace {
    pub fn new(brace: SmuflBrace) -> Self {
        Self {
            xy: Default::default(),
            height: Default::default(),

            color: Color::BLACK,

            brace,
        }
    }
}

impl ScoreElement for Brace {
    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
        self.color = _user_layout
            .foreground_color
            .unwrap_or(_app_defaults.foreground_color);
    }
}

impl Layoutable for Brace {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = origin.mv(0., self.height);
    }
}
