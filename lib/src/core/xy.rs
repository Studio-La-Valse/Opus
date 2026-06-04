use serde::Serialize;

#[derive(Default, Copy, Clone, Serialize)]
pub struct XY {
    pub x: f32,
    pub y: f32,
}

impl XY {
    pub const ZERO: XY = XY { x: 0.0, y: 0.0 };

    pub const INFINITE: XY = XY {
        x: f32::INFINITY,
        y: f32::INFINITY,
    };

    pub fn mv(&self, x: f32, y: f32) -> XY {
        XY {
            x: self.x + x,
            y: self.y + y,
        }
    }
}
