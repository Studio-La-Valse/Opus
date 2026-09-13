//! Chooses and runs the page-layout pass: placing every page, and so every
//! system beneath it, at its final origin.

use crate::geometry::xy::XY;
use crate::score::page_orientation::PageOrientation;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::score::Score;

/// Places every page -- and so every system, measure and note beneath it -- at
/// its final, absolute origin.
///
/// The first of [`SCORE_ARRANGERS`](crate::score::visual::arranger::SCORE_ARRANGERS):
/// nothing in the tree has an absolute coordinate until this has run, which is
/// exactly what beams, ties and mid-measure clef changes need from it.
pub struct PageArranger;

impl ScoreArranger for PageArranger {
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        page_layout_engine(params).arrange_pages(score, &XY::ZERO);
    }
}

// ---- internals ----

trait LayoutEngine {
    fn arrange_pages(&self, score: &mut Score, origin: &XY);
}

struct HorizontalPageLayout {
    gutter_even: f32,
    gutter_uneven: f32,
}

impl LayoutEngine for HorizontalPageLayout {
    fn arrange_pages(&self, score: &mut Score, origin: &XY) {
        let mut cursor = *origin;
        for (i, page) in score.pages.values_mut().enumerate() {
            page.arrange(&cursor);
            let gutter = if i % 2 == 0 {
                self.gutter_even
            } else {
                self.gutter_uneven
            };
            cursor = cursor.mv(page.width + gutter, 0.);
        }
    }
}

struct VerticalPageLayout {
    gutter: f32,
}

impl LayoutEngine for VerticalPageLayout {
    fn arrange_pages(&self, score: &mut Score, origin: &XY) {
        let mut cursor = *origin;
        for page in score.pages.values_mut() {
            page.arrange(&cursor);
            cursor = cursor.mv(0., page.height + self.gutter);
        }
    }
}

/// Picks the page-layout engine for the effective [`PageOrientation`], resolving
/// each gutter against the user layout then the app defaults.
fn page_layout_engine(params: LayoutParams<'_>) -> Box<dyn LayoutEngine> {
    let LayoutParams {
        user_layout: user,
        app_defaults: defaults,
        ..
    } = params;

    let orientation = user.page_orientation.unwrap_or(defaults.page_orientation);
    match orientation {
        PageOrientation::Horizontal => Box::new(HorizontalPageLayout {
            gutter_even: user
                .horizontal_gutter_even
                .unwrap_or(defaults.horizontal_gutter_even),
            gutter_uneven: user
                .horizontal_gutter_uneven
                .unwrap_or(defaults.horizontal_gutter_uneven),
        }),
        PageOrientation::Vertical => Box::new(VerticalPageLayout {
            gutter: user.vertical_gutter.unwrap_or(defaults.vertical_gutter),
        }),
    }
}
