use crate::drawable::layoutable::Layoutable;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::layout::Layout;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score_element::ScoreElement;
use crate::smufl::glyphs::clef::Clef as SmuflClef;

pub struct Clef {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub scale: f32,

    pub clef: SmuflClef,
}

impl Clef {
    pub fn new(clef: crate::smufl::glyphs::clef::Clef) -> Clef {
        Clef {
            xy: Default::default(),
            color: Color::BLACK,
            scale: 1.,

            width: 35.,  // TODO: MEASURE FROM META BBOXES
            height: 15., // TODO: MEASURE FROM META BBOXES

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

    /// Supplied origin x coordinate is left of clef, y coordinate is the line in the staff.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
