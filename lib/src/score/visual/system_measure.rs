use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct SystemMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,

    pub number: u32,
}

impl SystemMeasure {
    pub fn init_width(&mut self, width_specified: Option<f32>) {
        if let Some(width) = width_specified {
            self.width = self.width.max(width);
        }
    }
}

impl ScoreElement for SystemMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

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

impl Layoutable for SystemMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, _origin: &XY) {
        self.xy = *_origin;
    }
}

impl Content for SystemMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
