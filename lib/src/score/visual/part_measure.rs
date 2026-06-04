use crate::core::xy::XY;
use crate::drawable::element::Element;
use crate::score::drawable::content::Content;
use crate::score::visual::layoutable::Layoutable;

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,
}

impl PartMeasure {
    pub fn new() -> Self {
        Self {
            specified_width: None,
            final_width: 0.,
            width: 0.,
            height: 0.,
            origin: XY::ZERO,
        }
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;
    }
}

impl Content for PartMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
