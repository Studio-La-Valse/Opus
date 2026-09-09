use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::xy::XY;
use crate::score::visual::staff::Staff;

/// A visual element drawn from a SMuFL glyph: it sits at [`xy`](Self::xy), is
/// drawn at [`scale`](Self::scale), and everything else it knows about its
/// glyph -- the ink box, the stem anchors, the cutouts -- reaches it from the
/// font metadata normalized to staff spaces.
///
/// An implementor supplies only those two placement facts and inherits the
/// conversion out of staff spaces, which is the point: the box an element
/// measures itself by, the box the debug overlay draws around it and the box
/// the drawable reports are then one derivation rather than a copy of it per
/// element.
pub trait Placed {
    /// Where the element's glyph origin sits in world space.
    fn xy(&self) -> XY;

    /// The factor the glyph is drawn at, relative to a default-size staff.
    fn scale(&self) -> f32;

    /// World units per staff space at this element's current scale.
    fn unit(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale()
    }

    /// A bounding box from the glyph metadata -- the ink box, a cutout -- in
    /// world space.
    fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        bbox.placed(self.xy(), self.unit())
    }

    /// A point from the glyph metadata -- a stem anchor -- in world space.
    fn scale_pt(&self, pt: &XY) -> XY {
        pt.placed(self.xy(), self.unit())
    }
}
