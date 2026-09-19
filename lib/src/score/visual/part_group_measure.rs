use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;

#[derive(Default)]
pub struct PartGroupMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl PartGroupMeasure {
    pub fn resolve_layout(&mut self, _params: LayoutParams<'_>) {}

    pub fn measure(&mut self, available: &XY, _params: LayoutParams<'_>) {
        self.height = available.y;
    }

    pub fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
