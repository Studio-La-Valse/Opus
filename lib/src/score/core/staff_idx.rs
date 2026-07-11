#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct StaffIdx(u32);

impl From<u32> for StaffIdx {
    fn from(value: u32) -> Self {
        StaffIdx(value)
    }
}
