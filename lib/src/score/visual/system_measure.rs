use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;

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

impl SystemMeasure {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }

    pub fn measure(&mut self, available: &XY, _params: LayoutParams<'_>) {
        self.height = available.y;
    }

    pub fn arrange(&mut self, _origin: &XY) {
        self.xy = *_origin;
    }
}
