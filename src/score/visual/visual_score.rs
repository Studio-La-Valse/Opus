use crate::Layout;

pub struct VisualScore<'a> {
    pub layout: &'a mut Layout
}

impl<'a> VisualScore<'a> {
    pub fn new(layout: &'a mut Layout) -> Self {
        Self { layout }
    }
}