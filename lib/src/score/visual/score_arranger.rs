//! The shared contract of every pass that resolves a score-wide notation
//! element into drawn elements, once the pages have been arranged.

use crate::score::visual::beam_arranger::BeamArranger;
use crate::score::visual::clef_change_arranger::ClefChangeArranger;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::score::Score;
use crate::score::visual::tie_arranger::TieArranger;

/// Rebuilds one score-wide notation element -- beams, ties, or mid-measure
/// clef changes -- from the arranged tree.
///
/// Runs after
/// [`LayoutEngine::arrange_pages`](crate::score::visual::layout_engine::LayoutEngine::arrange_pages),
/// once every coordinate in the tree is absolute, for elements whose position
/// depends on something the visual tree does not own end to end: a tie's two
/// endpoints, a beam group's chords, or a clef change's staff can be measures,
/// systems or pages apart.
///
/// Implementations assign rather than append, and so must be idempotent: the
/// wasm render path re-runs `arrange_score` on a cached [`Score`] for every
/// frame, so a pass that read back what an earlier run of itself had written
/// would drift from one frame to the next.
pub trait ScoreArranger {
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>);
}

/// The passes [`arrange_score`](crate::score::engrave::arrange_score) runs, in
/// order.
///
/// Beams first because they move stem tips, and nothing in the tie geometry
/// reads a stem's length. Clef changes read noteheads and staves, which
/// neither of the others touch, so their position in the list is free.
pub const SCORE_ARRANGERS: [&dyn ScoreArranger; 3] =
    [&BeamArranger, &TieArranger, &ClefChangeArranger];
