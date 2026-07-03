use serde::Serialize;

#[derive(Default, Copy, Clone, Serialize)]
pub struct XY {
    pub x: f32,
    pub y: f32,
}

impl XY {
    pub const ZERO: XY = XY { x: 0.0, y: 0.0 };

    pub fn middle(left: &XY, right: &XY) -> XY {
        XY {
            x: (left.x + right.x) / 2.,
            y: (left.y + right.y) / 2.,
        }
    }

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

    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }
}
