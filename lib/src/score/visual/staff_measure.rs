use crate::core::xy::XY;
use crate::drawable::element::Element;
use crate::drawable::content::Content as _content;
use crate::score::visual::content::Content;
use crate::score::visual::layoutable::Layoutable;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub content: Vec<Box<dyn Content>>,
}

impl StaffMeasure {}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl _content for StaffMeasure {
    fn content(&self) -> Vec<&dyn _content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
