use crate::score::visual::layoutable::Layoutable;

pub trait Content: Layoutable {
    fn staff(&self) -> u32;
}
