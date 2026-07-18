use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::layout::Layout;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::layoutable::Layoutable;
use crate::xy::XY;

pub struct Clef {
    pub xy: XY,
    pub color: Color,
    pub scale: f32,

    pub pad_left: f32,

    pub clef: crate::smufl::glyphs::clef::Clef,
}

impl Clef {
    pub fn new(clef: crate::smufl::glyphs::clef::Clef) -> Clef {
        Clef {
            xy: Default::default(),
            color: Color::BLACK,
            scale: 1.,

            pad_left: 10.,

            clef,
        }
    }
}

impl ScoreElement for Clef {
    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Clef {
    fn measure(&mut self, _available: &XY) {}

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl DrawableContent for Clef {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        Vec::new()
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut result = Vec::new();

        let text = self.clef.as_text(self.color, self.xy, self.scale);
        result.push(text.into());

        result
    }
}
