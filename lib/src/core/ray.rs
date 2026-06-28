use crate::xy::XY;

#[derive(Copy, Clone)]
pub struct Ray {
    pub origin: XY,
    pub dir: XY,
}

impl Ray {
    pub fn from_dir(o: XY, dir: XY) -> Self {
        Self { origin: o, dir }
    }

    pub fn from_pt(o: XY, pt: XY) -> Self {
        Self {
            origin: o,
            dir: XY {
                x: pt.x - o.x,
                y: pt.y - o.y,
            },
        }
    }

    pub fn mv(&self, x: f32, y: f32) -> Self {
        Self {
            origin: self.origin.mv(x, y),
            dir: self.dir,
        }
    }

    pub fn valid(&self) -> bool {
        self.dir.length() > f32::EPSILON
    }

    pub fn intersect(&self, other: Ray) -> Option<XY> {
        let dx1 = self.dir.x;
        let dy1 = self.dir.y;
        let dx2 = other.dir.x;
        let dy2 = other.dir.y;

        let det = dx1 * dy2 - dy1 * dx2;
        if det.abs() < f32::EPSILON {
            return None; // parallel or degenerate
        }

        let ox = other.origin.x - self.origin.x;
        let oy = other.origin.y - self.origin.y;

        let t = (ox * dy2 - oy * dx2) / det;

        Some(XY {
            x: self.origin.x + t * dx1,
            y: self.origin.y + t * dy1,
        })
    }
}
