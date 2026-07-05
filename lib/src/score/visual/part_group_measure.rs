use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct PartGroupMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl PartGroupMeasure {}

impl ScoreElement for PartGroupMeasure {}

impl Layoutable for PartGroupMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for PartGroupMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
