use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;

pub struct SystemMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub number: u32,
}

impl SystemMeasure {
    pub fn init_width(&mut self, width_specified: Option<f32>) {
        if let Some(width) = width_specified {
            self.width = self.width.max(width);
        }
    }
}

impl Default for SystemMeasure {
    fn default() -> Self {
        Self {
            width: 1.,
            height: 0.,
            xy: XY::ZERO,
            number: 0,
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
