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
}
