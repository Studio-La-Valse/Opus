use crate::core::xy::XY;
use crate::score::visual::layoutable::Layoutable;

pub struct Note {
    pub staff: u32,
}

impl Note {}

impl Layoutable for Note {
    fn measure(&mut self, _available: XY) {}
    fn arrange(&mut self, _origin: XY) {}
}
