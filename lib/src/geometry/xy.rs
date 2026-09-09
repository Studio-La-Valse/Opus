use serde::Serialize;
use std::ops::{Add, Sub};

#[derive(Default, Debug, Copy, Clone, Serialize)]
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

    pub fn middle(left: &XY, right: &XY) -> XY {
        XY {
            x: (left.x + right.x) / 2.,
            y: (left.y + right.y) / 2.,
        }
    }

    pub fn mv(&self, x: f32, y: f32) -> XY {
        XY {
            x: self.x + x,
            y: self.y + y,
        }
    }

    pub fn scale(&self, scale: f32) -> XY {
        XY {
            x: self.x * scale,
            y: self.y * scale,
        }
    }

    /// Scales the point by `factor` about `pivot`, leaving `pivot` fixed.
    pub fn scale_about(&self, factor: f32, pivot: XY) -> XY {
        pivot + (*self - pivot).scale(factor)
    }

    /// Reads `self` as a figure normalized to some unit and measured from a
    /// glyph origin -- the form SMuFL metadata states its anchors in, in staff
    /// spaces -- and places it in world space at `origin`, `unit` world units
    /// to the staff space.
    pub fn placed(&self, origin: XY, unit: f32) -> XY {
        origin + self.scale(unit)
    }

    pub fn length(&self) -> f32 {
        self.x.hypot(self.y)
    }
}

impl Add for XY {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            x: self.x + other.x,
            y: self.y + other.y,
        }
    }
}

impl Sub for XY {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self {
            x: self.x - other.x,
            y: self.y - other.y,
        }
    }
}
