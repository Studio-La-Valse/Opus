use crate::drawable::layoutable::Layoutable;
use crate::geometry::xy::XY;
use crate::score::visual::score_element::ScoreElement;

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
