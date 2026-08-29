use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::score_element::ScoreElement;

pub struct SystemMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,

    pub number: u32,
}

impl SystemMeasure {
    pub fn new(number: u32) -> Self {
        Self {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,
            color: Color::BLACK,
            number,
        }
    }

    pub fn init_width(&mut self, width_specified: Option<f32>) {
        if let Some(width) = width_specified {
            self.width = self.width.max(width);
        }
    }
}

impl ScoreElement for SystemMeasure {
    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for SystemMeasure {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self._apply_layout(
            params.score_defaults,
            params.user_layout,
            params.app_defaults,
        );

        self.height = available.y;
    }

    fn arrange(&mut self, _origin: &XY) {
        self.xy = *_origin;
    }
}
