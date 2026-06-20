use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub struct BoundingBox {
    pub x_min: f32,
    pub y_min: f32,
    pub x_max: f32,
    pub y_max: f32,
}

impl BoundingBox {
    pub fn width(&self) -> f32 {
        self.x_max - self.x_min
    }

    pub fn height(&self) -> f32 {
        self.y_max - self.y_min
    }

    pub fn zero(&self) -> bool {
        self.width().abs() <= f32::EPSILON && self.height().abs() <= f32::EPSILON
    }

    pub const ZERO: BoundingBox = BoundingBox {
        x_min: 0.,
        x_max: 0.,
        y_min: 0.,
        y_max: 0.,
    };
}
