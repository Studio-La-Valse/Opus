use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::{Layout, UserLayout};
use crate::score::visual::layoutable::Layoutable;
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

    fn apply_layout(&mut self, layout: &Layout, user_layout: &UserLayout) {
        self.color = layout.page_color;

        let user_page_color = user_layout.page_color;
        if let Some(user_page_color) = user_page_color {
            self.color = user_page_color;
        }
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
