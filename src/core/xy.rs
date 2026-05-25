#[derive(Default)]
pub struct XY {
    pub x: f32,
    pub y: f32,
}

impl XY {
    pub const ZERO: XY = XY { x: 0.0, y: 0.0 };
}
