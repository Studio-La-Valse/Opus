use crate::drawable::layoutable::Layoutable;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score_element::ScoreElement;

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub line_width: f32,
}

impl SectionMeasure {}

impl ScoreElement for SectionMeasure {
    fn _apply_layout(
        &mut self,
        layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_width = user_layout
            .light_barline
            .or(layout.appearance.light_barline)
            .unwrap_or(app_defaults.barline_light)
    }
}

impl Layoutable for SectionMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
