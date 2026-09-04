use crate::score::core::step::Step;

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub struct Pitch {
    pub step: Step,
    pub octave: i32,
}

impl Pitch {}
