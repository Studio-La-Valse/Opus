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

    pub fn x_min(&self) -> f32 {
        self.xy.x
    }

    pub fn x_max(&self) -> f32 {
        self.xy.x + self.size.x
    }

    pub fn y_min(&self) -> f32 {
        self.xy.y
    }

    pub fn y_max(&self) -> f32 {
        self.xy.y + self.size.y
    }

    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.x_min() < other.x_max()
            && self.x_max() > other.x_min()
            && self.y_min() < other.y_max()
            && self.y_max() > other.y_min()
    }

    /// Returns the overlapping rectangle (intersection) of two bounding boxes, if any.
    /// AI generated.
    pub fn intersection(&self, other: &BoundingBox) -> Option<BoundingBox> {
        if !self.intersects(other) {
            return None;
        }

        let inter_min_x = self.x_min().max(other.x_min());
        let inter_max_x = self.x_max().min(other.x_max());
        let inter_min_y = self.y_min().max(other.y_min());
        let inter_max_y = self.y_max().min(other.y_max());

        Some(BoundingBox {
            xy: XY {
                x: inter_min_x,
                y: inter_min_y,
            },
            size: XY {
                x: inter_max_x - inter_min_x,
                y: inter_max_y - inter_min_y,
            },
        })
    }

    /// Returns true if `other` is fully contained inside `self`.
    ///
    /// Includes a tiny `EPSILON` tolerance to prevent floating-point inaccuracies
    /// from failing a true boundary match.
    /// AI generated.
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        other.x_min() >= self.x_min() - f32::EPSILON
            && other.x_max() <= self.x_max() + f32::EPSILON
            && other.y_min() >= self.y_min() - f32::EPSILON
            && other.y_max() <= self.y_max() + f32::EPSILON
    }

    pub fn is_zero(&self) -> bool {
        self.width().abs() <= f32::EPSILON && self.height().abs() <= f32::EPSILON
    }

    pub fn scale(&self, scale: f32) -> BoundingBox {
        BoundingBox {
            xy: self.xy,
            size: self.size.scale(scale),
        }
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
