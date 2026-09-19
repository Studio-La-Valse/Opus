use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub light_barline: f32,
}

impl SectionMeasure {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.light_barline = user_layout
            .light_barline
            .or(score_defaults.appearance.light_barline)
            .unwrap_or(app_defaults.light_barline)
    }

    pub fn measure(&mut self, available: &XY, _params: LayoutParams<'_>) {
        self.height = available.y;
    }

    pub fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
