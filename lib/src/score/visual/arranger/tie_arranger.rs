//! Resolves [`Tie`](crate::score::visual::tie::Tie)s into drawn arcs, after the
//! pages have been arranged.
//!
//! One of the three passes that have to see the whole score at once; see
//! [`ScoreArranger`](crate::score::visual::arranger::ScoreArranger).
//! Every step before this is local: a `StaffMeasure` places its own clef from
//! its own left edge. But a tie's two endpoints may sit in different measures,
//! different systems or different pages, and a note's absolute position does
//! not exist until `arrange` has run. So ties are resolved last, from a flat
//! index of every note's final coordinates.
//!
//! The payoff is that a cross-**page** tie needs no code of its own. Once
//! fragments are keyed by [`SystemKey`], a tie whose endpoints happen to be on
//! different pages files its two halves under two different pages, and the
//! compositor -- which already walks pages, then systems -- picks them up
//! without knowing anything happened.

use std::collections::HashMap;

use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::score::Score;
use crate::score::visual::system::SystemKey;
use crate::score::visual::tie::{TieMetrics, TieSegment, split_tie};

/// Rebuilds every system's tie arcs from `score.ties`.
pub struct TieArranger;

impl ScoreArranger for TieArranger {
    /// Assigns rather than appends, so calling it repeatedly is idempotent --
    /// the same property
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// gets by resetting every stem before it fits a beam ray, and the reason
    /// the wasm render path can re-arrange a cached score for a new
    /// `UserLayout`.
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        let metrics = TieMetrics::resolve(params);
        let anchors = score.note_anchors();
        let extents = score.system_extents();

        let mut out: HashMap<SystemKey, Vec<TieSegment>> = HashMap::new();
        for tie in &score.ties {
            for (key, segment) in split_tie(tie, &anchors, &extents, &metrics) {
                out.entry(key).or_default().push(segment);
            }
        }

        for (page_key, page) in score.pages.iter_mut() {
            for (system_key, system) in page.systems.iter_mut() {
                system.ties = out.remove(&(*page_key, *system_key)).unwrap_or_default();
            }
        }
    }
}
