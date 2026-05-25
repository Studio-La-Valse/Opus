use crate::core::xy::XY;
use crate::score::visual::content::Content;
use crate::score::visual::layoutable::Layoutable;

pub struct Note {
    pub staff: u32,
}

impl Note {}

impl Content for Note {
    fn staff(&self) -> u32 {
        self.staff
    }
}

impl Layoutable for Note {
    fn measure(&mut self, _available: XY) {}
    fn arrange(&mut self, _origin: XY) {}
}
