use crate::geometry::xy::XY;

pub trait Layoutable {
    fn measure(&mut self, available: &XY);
    fn arrange(&mut self, origin: &XY);
}
