use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::layout_options::APP_DEFAULTS;
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
            font,
        } = params;

        self.color = params.foreground_color();

        self.light_barline = user_layout
            .barline
            .light
            .or(score_defaults.appearance.light_barline)
            .or(font.layout.barline.light)
            .unwrap_or(APP_DEFAULTS.barline.light)
    }
}
