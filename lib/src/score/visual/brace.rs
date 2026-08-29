use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::smufl::glyphs::brace::Brace as SmuflBrace;

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

impl Brace {
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

impl Layoutable for Brace {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = origin.mv(0., self.height);
    }
}
