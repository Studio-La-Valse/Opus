use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::smufl::glyphs::brace::Brace as SmuflBrace;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
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

impl DrawableContent for Brace {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        vec![]
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut elements: Vec<DrawableElement> = Vec::new();

        let def_height = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;
        let scale = self.height / def_height;

        let text = self.brace.as_text(self.color, self.xy, scale);
        elements.push(text.into());

        elements
    }
}
