#[derive(Default, Copy, Clone)]
pub struct StaffCtx {
    pub hidden: bool,
    pub distance_from_top: f32,
    pub scaling: f32,

    /// How many lines the staff is drawn with, so the elements positioned
    /// against it without belonging to it -- ledger lines above all -- know
    /// where its bottom line is. See
    /// [`Staff::lines`](crate::score::visual::staff::Staff).
    pub lines: usize,
}
