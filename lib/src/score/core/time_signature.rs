use crate::score::core::duration_base::BaseDuration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TimeSignature {
    pub time: u8,
    pub base: BaseDuration,
}
