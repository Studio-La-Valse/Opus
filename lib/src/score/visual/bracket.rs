use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::rect::Rect;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::smufl::glyphs::bracket::{BracketBottom, BracketTop};
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::score_element::ScoreElement;
use crate::xy::XY;

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
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
        self.color = _user_layout
            .foreground_color
            .unwrap_or(_app_defaults.foreground_color);
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

impl DrawableContent for Bracket {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut elements: Vec<DrawableElement> = Vec::new();

        let text = self.bracket_top.as_text(self.color, self.xy, self.scale);
        elements.push(text.into());

        let text = self
            .bracket_bottom
            .as_text(self.color, self.xy.mv(0., self.height), self.scale);
        elements.push(text.into());

        let rect = Rect {
            xy: self.xy.mv(0., -1.),
            width: 5.,
            height: self.height + 2.,
            color: self.color,
            stroke_color: None,
            stroke_width: None,
        };
        elements.push(rect.into());

        elements
    }
}
