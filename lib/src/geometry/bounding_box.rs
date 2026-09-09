use serde::Serialize;

use crate::geometry::xy::XY;

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
    pub fn contains_box(&self, other: &BoundingBox) -> bool {
        other.x_min() >= self.x_min() - f32::EPSILON
            && other.x_max() <= self.x_max() + f32::EPSILON
            && other.y_min() >= self.y_min() - f32::EPSILON
            && other.y_max() <= self.y_max() + f32::EPSILON
    }

    /// The smallest box containing both. Used where one element's ink is
    /// several pieces -- a bracket is a spine plus two tips -- and the box it
    /// reports has to cover all of them.
    pub fn union(&self, other: &BoundingBox) -> BoundingBox {
        let x_min = self.x_min().min(other.x_min());
        let y_min = self.y_min().min(other.y_min());
        let x_max = self.x_max().max(other.x_max());
        let y_max = self.y_max().max(other.y_max());

        BoundingBox {
            xy: XY { x: x_min, y: y_min },
            size: XY {
                x: x_max - x_min,
                y: y_max - y_min,
            },
        }
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

    /// Scales the box by `factor` about `pivot`, leaving `pivot` fixed: both
    /// the corner and the extent grow, unlike [`scale`](Self::scale), which
    /// resizes in place.
    pub fn scale_about(&self, factor: f32, pivot: XY) -> BoundingBox {
        BoundingBox {
            xy: self.xy.scale_about(factor, pivot),
            size: self.size.scale(factor),
        }
    }

    /// Reads `self` as a box normalized to some unit and measured from a glyph
    /// origin -- the form SMuFL metadata states its bounding boxes in, in staff
    /// spaces -- and places it in world space at `origin`, `unit` world units
    /// to the staff space.
    ///
    /// This is the one derivation from normalized glyph metadata to world
    /// geometry; everything that draws or measures a glyph goes through it, so
    /// that the box a glyph reports, the box the debug overlay draws and the
    /// width layout reserves cannot drift apart.
    pub fn placed(&self, origin: XY, unit: f32) -> BoundingBox {
        BoundingBox {
            xy: self.xy.placed(origin, unit),
            size: self.size.scale(unit),
        }
    }

    pub fn mv(&self, x: f32, y: f32) -> BoundingBox {
        BoundingBox {
            xy: self.xy.mv(x, y),
            ..*self
        }
    }

    /// A zero-sized box at `xy`. Every edge, and both midpoints, are that one
    /// point, so a [`Text`](crate::drawable::elements::text::Text) boxed this
    /// way anchors there whatever its alignments -- which is how a producer
    /// with nothing to reserve positions text by a bare point.
    pub fn point(xy: XY) -> BoundingBox {
        BoundingBox { xy, size: XY::ZERO }
    }

    pub const ZERO: BoundingBox = BoundingBox {
        xy: XY::ZERO,
        size: XY::ZERO,
    };
}
