#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct Voice(u32);

impl From<u32> for Voice {
    fn from(value: u32) -> Self {
        Voice(value)
    }
}
