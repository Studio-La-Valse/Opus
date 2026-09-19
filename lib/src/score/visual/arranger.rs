//! The passes that lay a score out once its appearance is resolved: measuring
//! it, placing the pages, then resolving every score-wide notation element
//! that placement makes possible. Grouped in one folder because each is
//! otherwise a small, self-contained module easy to lose among `visual`'s
//! other, purely structural ones.

mod arrange_machine;
mod beam_arranger;
mod clef_change_arranger;
mod content_arranger;
mod page_arranger;
mod score_measurement;
mod tie_arranger;

pub use arrange_machine::ArrangeMachine;
pub use beam_arranger::BeamArranger;
pub use clef_change_arranger::ClefChangeArranger;
pub use content_arranger::ContentArranger;
pub use page_arranger::PageArranger;
pub use score_measurement::ScoreMeasurement;
pub use tie_arranger::TieArranger;

use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::score::Score;

/// Runs one top-level layout pass over a [`Score`]: sizing it, placing the
/// pages, or -- once that has happened -- resolving beams, ties, or
/// mid-measure clef changes.
///
/// The latter three depend on something the visual tree does not own end to
/// end: a tie's two endpoints, a beam group's chords, or a clef change's staff
/// can be measures, systems or pages apart, so none of them can run until
/// every coordinate in the tree is absolute -- which is exactly what placing
/// the pages produces.
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
/// Measuring first, because nothing can be placed before it has a size. Page
/// layout next, because nothing else has an absolute coordinate before
/// it. Content placement after, because every note, rest and opening column
/// needs its container's final position and nothing downstream can run
/// without them: beams move stem tips, ties and clef changes read noteheads
/// and staves. Beams before ties because they move stem tips and nothing in
/// the tie geometry reads a stem's length. Clef changes read noteheads and
/// staves, which neither of the others touch, so their position in the list is
/// free.
pub const SCORE_ARRANGERS: [&dyn ScoreArranger; 6] = [
    &ScoreMeasurement,
    &PageArranger,
    &ContentArranger,
    &BeamArranger,
    &TieArranger,
    &ClefChangeArranger,
];
