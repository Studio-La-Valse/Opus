use crate::core::xy::XY;
use crate::score::visual::content::Content;
use crate::score::visual::layoutable::Layoutable;

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f64>,
    pub content: Vec<Box<dyn Content>>,
    pub width: f32,
    pub height: f32,
    pub origin: XY,
}

impl PartMeasure {
    pub fn new() -> Self {
        Self {
            specified_width: None,
            content: Vec::new(),
            width: 0.0,
            height: 0.0,
            origin: XY::ZERO,
        }
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: XY) {
        self.height = available.y;
        self.width = available.x;
    }

    fn arrange(&mut self, origin: XY) {
        self.origin = origin;
    }
}
