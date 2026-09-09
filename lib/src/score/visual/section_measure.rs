use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub line_width: f32,
}

impl SectionMeasure {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_width = user_layout
            .light_barline
            .or(score_defaults.appearance.light_barline)
            .unwrap_or(app_defaults.barline_light)
    }
}

impl Layoutable for SectionMeasure {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
