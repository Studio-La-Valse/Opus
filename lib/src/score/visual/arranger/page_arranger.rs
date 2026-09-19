//! Runs the page-layout pass: placing every page -- and so every system
//! beneath it -- at its own, page-local origin.

use crate::geometry::xy::XY;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::score::Score;

/// Places every page -- and so every system, measure and note beneath it -- at
/// its own origin, `XY::ZERO`. Coordinates are page-local: nothing compares a
/// coordinate on one page against a coordinate on another. Arranging pages
/// relative to each other is left to whatever consumes a rendered page --
/// the browser component stacks them with CSS, and the PDF and SVG sinks
/// already emit one physical page at a time.
///
/// The first of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS):
/// nothing in the tree has a resolved coordinate until this has run, which is
/// exactly what beams, ties and mid-measure clef changes need from it.
pub struct PageArranger;

impl ScoreArranger for PageArranger {
    fn arrange(&self, score: &mut Score, _params: LayoutParams<'_>) {
        for page in score.pages.values_mut() {
            page.arrange(&XY::ZERO);
        }
    }
}
