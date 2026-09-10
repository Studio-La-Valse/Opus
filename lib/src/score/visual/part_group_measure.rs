use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};

#[derive(Default)]
pub struct PartGroupMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl Layoutable for PartGroupMeasure {
    fn resolve_layout(&mut self, _params: LayoutParams<'_>) {}

    fn measure(&mut self, available: &XY, _params: LayoutParams<'_>) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
