use serde::Serialize;

use crate::xy::XY;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BoundingBox {
    pub xy: XY,
    pub size: XY,
}

impl BoundingBox {
    pub fn width(&self) -> f32 {
        self.size.x
    }

    pub fn height(&self) -> f32 {
        self.size.y
    }

    pub fn x_max(&self) -> f32 {
        self.xy.x + self.width()
    }

    pub fn y_max(&self) -> f32 {
        self.xy.y + self.height()
    }

    pub fn is_zero(&self) -> bool {
        self.width().abs() <= f32::EPSILON && self.height().abs() <= f32::EPSILON
    }

    pub fn mv(&self, x: f32, y: f32) -> BoundingBox {
        BoundingBox {
            xy: self.xy.mv(x, y),
            ..*self
        }
    }

    pub const ZERO: BoundingBox = BoundingBox {
        xy: XY::ZERO,
        size: XY::ZERO,
    };
}
