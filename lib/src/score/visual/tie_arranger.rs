//! Resolves [`Tie`]s into drawn arcs, after the pages have been arranged.
//!
//! This is the one pass that has to see the whole score at once. Every other
//! layout step is local -- a `PartMeasure` arranges its own beams from its own
//! chords -- but a tie's two endpoints may sit in different measures, different
//! systems or different pages, and a note's absolute position does not exist
//! until `arrange` has run. So ties are resolved last, from a flat index of
//! every note's final coordinates.
//!
//! The payoff is that a cross-**page** tie needs no code of its own. Once
//! fragments are keyed by [`SystemKey`], a tie whose endpoints happen to be on
//! different pages files its two halves under two different pages, and the
//! compositor -- which already walks pages, then systems -- picks them up
//! without knowing anything happened.

use std::collections::HashMap;

use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::NoteId;
use crate::score::visual::score::Score;
use crate::score::visual::stem::UpDown;
use crate::score::visual::tie::{Tie, TieMetrics, TieSegment, TieSide, tie_arc};
use crate::score::walk_cursor::Visibility;

/// Addresses one system in the score: its page's key and its own key, both taken
/// from the enclosing `BTreeMap`s.
///
/// Deliberately the map keys rather than any stored index: they are document
/// order by construction, which is exactly what "does the tie's start come
/// before its end" means. `Page::number` happens to carry the same value today,
/// but it is there for margin resolution and nothing here should depend on the
/// two staying in step.
pub type SystemKey = (u32, u32);

/// A note's finished geometry, copied out of the tree so the resolution phase
/// can borrow the score immutably and the install phase mutably.
#[derive(Copy, Clone, Debug)]
pub struct NoteAnchor {
    pub key: SystemKey,
    /// Left edge of the notehead at its vertical centre -- exactly what
    /// `Note::xy` is, per `Note::arrange_ctx` and `PartMeasure::ledger_lines`.
    pub left: XY,
    pub width: f32,
    /// Right edge of the `PartMeasure` this note sits in. A tie broken across a
    /// system runs its opening fragment out to here, since "the space available
    /// to the tie" is the remainder of its own measure.
    pub measure_right: f32,
    /// The note's own scale factor, so grace notes get proportionate ties.
    pub scale: f32,
    pub staff_line: i32,
    /// The owning chord's stem direction, for [`TieSide::infer`].
    pub stem: Option<UpDown>,
    pub color: Color,
}

impl NoteAnchor {
    /// Where a tie leaving this note to the right begins.
    fn tip_right(&self, side: TieSide, metrics: &TieMetrics) -> XY {
        self.left.mv(
            self.width + metrics.note_gap,
            side.sign() * metrics.vertical_offset,
        )
    }

    /// Where a tie arriving at this note from the left ends.
    fn tip_left(&self, side: TieSide, metrics: &TieMetrics) -> XY {
        self.left
            .mv(-metrics.note_gap, side.sign() * metrics.vertical_offset)
    }
}

/// A system's horizontal extent, all a broken tie needs to know about it.
#[derive(Copy, Clone, Debug)]
pub struct SystemExtent {
    pub left: f32,
    pub right: f32,
}

/// Rebuilds every system's tie arcs from `score.ties`.
///
/// Runs after `LayoutEngine::arrange_pages`, and assigns rather than appends, so
/// calling it repeatedly is idempotent -- the same property `PartMeasure` gets
/// from clearing `self.beams` before rebuilding them, and the reason the wasm
/// render path can re-arrange a cached score for a new `UserLayout`.
pub fn arrange_ties(score: &mut Score, params: LayoutParams<'_>) {
    let metrics = TieMetrics::resolve(params);
    let anchors = collect_note_anchors(score);
    let extents = collect_system_extents(score);

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

/// Every note's final geometry, keyed by id.
///
/// Indexes exactly the notes `RenderCompositor::walk_pages` draws, so a tie can
/// never point at a note that is not on the page. That means skipping hidden
/// *parts* but not hidden *staves*: the compositor's `walk_part` skips a hidden
/// staff when drawing staff lines, yet still walks every `PartMeasure` chord
/// regardless of which staff its notes sit on, so those noteheads do get drawn.
/// Filtering them here instead would leave a notehead rendered with its tie
/// missing. One O(notes) pass per arrange, negligible next to the arrange
/// itself.
pub fn collect_note_anchors(score: &Score) -> HashMap<NoteId, NoteAnchor> {
    let mut anchors = HashMap::new();

    for (page_key, page) in score.pages.iter() {
        for (system_key, system) in page.systems.iter() {
            let key = (*page_key, *system_key);

            for section in system.sections.values() {
                for group in section.part_groups.values() {
                    for part in group.parts.values() {
                        if part.visibility == Visibility::Hidden {
                            continue;
                        }

                        for measure in part.measures.values() {
                            // `Part::arrange` walks its measures left to right,
                            // advancing the origin by each measure's width, so
                            // these two are the measure's own span.
                            let measure_right = measure.origin.x + measure.width;

                            for chord in measure.chords.values().flatten() {
                                let stem = chord.stem.as_ref().map(|s| s.direction);

                                for (id, note) in chord.notes.iter() {
                                    anchors.insert(
                                        *id,
                                        NoteAnchor {
                                            key,
                                            left: note.xy,
                                            width: note.width,
                                            measure_right,
                                            scale: note.scale,
                                            staff_line: note.staff_line,
                                            stem,
                                            color: note.color,
                                        },
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    anchors
}

/// Every system's horizontal extent, keyed the same way as the anchors.
pub fn collect_system_extents(score: &Score) -> HashMap<SystemKey, SystemExtent> {
    let mut extents = HashMap::new();

    for (page_key, page) in score.pages.iter() {
        for (system_key, system) in page.systems.iter() {
            extents.insert(
                (*page_key, *system_key),
                SystemExtent {
                    left: system.xy.x,
                    right: system.xy.x + system.width,
                },
            );
        }
    }

    extents
}

/// The arcs one tie contributes, each tagged with the system it belongs to.
///
/// One segment when both endpoints share a system; two when they do not -- an
/// *opening* fragment leaving the start note and running out to the end of its
/// measure, and a short *closing* fragment (the "courtesy tie") arriving at the
/// end note from the left. The asymmetry is intentional: the opening fragment
/// takes the space available to it, the closing one is a fixed stub.
///
/// Each fragment is a **complete tie shape**, tapered to a point at both of its
/// own ends, which is how printed music engraves a broken tie. It is tempting to
/// instead slice one long arc at its apex and let the two blunt halves line up
/// across the break, but that is not what the convention looks like on the page.
/// Because both fragments are ordinary arcs between two tapered endpoints at the
/// same height, `tie_arc` needs no notion of a cut end and serves the unbroken
/// and broken cases identically.
///
/// Takes anchors and extents rather than a `&Score` on purpose: it keeps the
/// split rule a pure function, testable without building a multi-system
/// document, and reusable as-is when slurs arrive.
///
/// Returns nothing at all for a tie that cannot be drawn: an endpoint with no
/// anchor (its note was skipped for lacking `default-x`), an end that sorts
/// before its start, or a same-system pair whose end is not to the right of its
/// start.
pub fn split_tie(
    tie: &Tie,
    anchors: &HashMap<NoteId, NoteAnchor>,
    extents: &HashMap<SystemKey, SystemExtent>,
    metrics: &TieMetrics,
) -> Vec<(SystemKey, TieSegment)> {
    let (Some(start), Some(end)) = (anchors.get(&tie.start), anchors.get(&tie.end)) else {
        return Vec::new();
    };

    if end.key < start.key {
        return Vec::new();
    }

    let side = tie
        .side
        .unwrap_or_else(|| TieSide::infer(start.stem, start.staff_line));

    let p0 = start.tip_right(side, metrics);
    let p1 = end.tip_left(side, metrics);

    if start.key == end.key {
        if p1.x <= p0.x {
            return Vec::new();
        }

        let shape = tie_arc(p0, p1, side, start.scale, metrics, start.color);
        return vec![(start.key, TieSegment { shape })];
    }

    let (Some(start_system), Some(end_system)) = (extents.get(&start.key), extents.get(&end.key))
    else {
        return Vec::new();
    };

    let mut segments = Vec::with_capacity(2);

    // Opening fragment: a complete arc leaving the note and taking up the space
    // available to it, which is the rest of its own measure less a margin so it
    // clears the barline. It ends level with the note it left, not raised to the
    // apex, because it is a whole tie shape in its own right.
    //
    // The two fragments are deliberately not the same length: this one spans
    // what is left of the measure, while the closing one below is short and
    // fixed. Floored at `break_fragment` so a note falling right at the end of a
    // measure still gets a visible arc rather than a sliver.
    //
    // The cap at the system's right edge is a *typographic* limit, not a
    // technical one -- there is plenty of real page past the final barline (the
    // right margin, and nothing clips before the page edge), but a tie reaching
    // into the margin reads as a mistake. The trade-off it forces: for a note
    // crowded against the last barline of a system, the cap wins over the floor
    // and the fragment comes out shorter than `break_fragment`. Lifting the cap
    // is the alternative if that ever looks worse than the overhang would.
    let measure_end = start.measure_right.min(start_system.right) - metrics.break_inset;
    let out_x = measure_end
        .max(p0.x + metrics.break_fragment)
        .min(start_system.right);
    if out_x > p0.x {
        let shape = tie_arc(
            p0,
            XY { x: out_x, y: p0.y },
            side,
            start.scale,
            metrics,
            start.color,
        );
        segments.push((start.key, TieSegment { shape }));
    }

    // Closing fragment: likewise a complete arc, arriving at the note from the
    // left. Its length is fixed rather than "back to the system edge", because
    // the next system opens with a clef and key signature it must not run
    // through.
    let in_x = (p1.x - metrics.break_fragment).max(end_system.left);
    if p1.x > in_x {
        let shape = tie_arc(
            XY { x: in_x, y: p1.y },
            p1,
            side,
            end.scale,
            metrics,
            end.color,
        );
        segments.push((end.key, TieSegment { shape }));
    }

    segments
}
