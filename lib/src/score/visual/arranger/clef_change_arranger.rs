//! Resolves [`ClefChange`]s into drawn clefs, after the pages have been
//! arranged.
//!
//! The third of the passes that see the whole score at once; see
//! [`ScoreArranger`](crate::score::visual::arranger::ScoreArranger). It
//! is here for a variation on the other two's reason. A tie's two endpoints may
//! be systems apart; a mid-measure clef change has only one anchor, but that
//! anchor is in the wrong branch of the tree. The change belongs to a *staff*
//! -- it is a fact about how the staff is read from that point on -- while the
//! only thing that can say where on the page it goes is the note or rest it
//! precedes, and notes hang off a `PartMeasure`, not off a `Staff`. Nothing
//! owns both.
//!
//! The clefs used to be stored on the anchors themselves: a
//! `BTreeMap<StaffIdx, Clef>` on every `Chord` and an `Option<Clef>` on every
//! `Rest`, whether or not a clef ever changed there. That put a staff's business
//! on a note, made every chord carry a map it almost never used, and left the
//! two anchors placing the same glyph with two copies of the same arithmetic.
//! Here there is one copy, and a `Chord` is once again only notes and a stem.

use std::collections::{BTreeMap, HashMap};

use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::arrange_machine::ArrangeMachine;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::clef::{Clef, ClefAnchor, ClefChange};
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::NoteId;
use crate::score::visual::part::Part;
use crate::score::visual::score::Score;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::system::SystemKey;
use crate::score::walk_cursor::Visibility;

/// Rebuilds every system's mid-measure clef changes from `score.clef_changes`.
pub struct ClefChangeArranger;

impl ScoreArranger for ClefChangeArranger {
    /// Assigns rather than appends, so calling it repeatedly is idempotent --
    /// the same property
    /// [`TieArranger`](crate::score::visual::arranger::TieArranger) and
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// have, and the reason the wasm render path can re-arrange a cached score
    /// for a new `UserLayout`.
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        let mut out = place_clef_changes(score, params);

        for (page_key, page) in score.pages.iter_mut() {
            for (system_key, system) in page.systems.iter_mut() {
                system.clef_changes = out.remove(&(*page_key, *system_key)).unwrap_or_default();
            }
        }
    }
}

// ---- internals ----

/// Every clef change that has an anchor on the page, placed and filed under the
/// system it lands in.
///
/// Walks the tree looking for the anchors rather than indexing them first, which
/// is where this parts company with
/// [`Score::note_anchors`](crate::score::visual::score::Score::note_anchors).
/// A tie's geometry needs nothing but its two notes, so an index of notes is
/// enough; a clef change also needs the staff it names -- where its top line
/// sits and what it is scaled by -- and that is a fact about the anchor's
/// *part*. Resolving it where the walk still has the part in hand is cheaper and
/// plainer than carrying it out of the tree.
///
/// Skips hidden parts, for the reason `collect_runs` does: the clefs end up on
/// the `System`, past the point where the compositor filters a part out.
fn place_clef_changes(score: &Score, params: LayoutParams<'_>) -> HashMap<SystemKey, Vec<Clef>> {
    let mut pending: HashMap<NoteId, Vec<&ClefChange>> = HashMap::new();
    for change in score.clef_changes.iter() {
        pending.entry(change.anchor).or_default().push(change);
    }

    let mut out: HashMap<SystemKey, Vec<Clef>> = HashMap::new();
    if pending.is_empty() {
        return out;
    }

    for (page_key, page) in score.pages.iter() {
        for (system_key, system) in page.systems.iter() {
            let key = (*page_key, *system_key);

            for section in system.sections.values() {
                for group in section.part_groups.values() {
                    for part in group.parts.values() {
                        if part.visibility == Visibility::Hidden {
                            continue;
                        }

                        // An empty `Vec` costs no allocation, so a part with
                        // nothing to place leaves `out` untouched and the
                        // system's own list is simply cleared below.
                        let mut clefs = Vec::new();
                        place_part_clef_changes(part, &pending, params, &mut clefs);

                        if !clefs.is_empty() {
                            out.entry(key).or_default().extend(clefs);
                        }
                    }
                }
            }
        }
    }

    out
}

/// Places the clef changes anchored to anything in one part.
///
/// Both kinds of anchor reduce to the same two numbers -- the anchor's left edge
/// and the staff the clef names -- so rests and chord notes differ only in where
/// that left edge is read from.
fn place_part_clef_changes(
    part: &Part,
    pending: &HashMap<NoteId, Vec<&ClefChange>>,
    params: LayoutParams<'_>,
    out: &mut Vec<Clef>,
) {
    // Where each staff of this part sits, as `ArrangeMachine::arrange_part_clear_of` worked it
    // out. Read from here rather than from `Staff::xy` so that a clef change on
    // a hidden staff lands where it always did: a hidden staff is left
    // unarranged, but it is still in this map.
    let staff_ctx = part.create_staff_ctx();

    let place = |anchor: NoteId, left: f32, out: &mut Vec<Clef>| {
        let Some(changes) = pending.get(&anchor) else {
            return;
        };

        for change in changes {
            if let Some(clef) = place_clef_change(change, left, part.xy.y, &staff_ctx, params) {
                out.push(clef);
            }
        }
    };

    for staff in part.staves.values() {
        for measure in staff.measures.values() {
            for rest in measure.rests.iter() {
                place(rest.id, rest.xy.x, out);
            }
        }
    }

    for measure in part.measures.values() {
        for chord in measure.chords.values().flatten() {
            for note in chord.notes.iter() {
                place(note.id, note.xy.x, out);
            }
        }
    }
}

/// One clef change as a drawn glyph.
///
/// `part_top` is the y of the top of the part, which is what the staff offsets
/// in `staff_ctx` are measured from. Returns `None` for a change naming a staff
/// its part does not have, which is a malformed `<clef number=…>` rather than
/// anything this pass can draw.
fn place_clef_change(
    change: &ClefChange,
    left: f32,
    part_top: f32,
    staff_ctx: &BTreeMap<StaffIdx, StaffCtx>,
    params: LayoutParams<'_>,
) -> Option<Clef> {
    let ctx = staff_ctx.get(&change.staff)?;

    let mut clef = Clef::new(change.clef.clone());
    clef.resolve_layout(params);

    // Built here rather than during the measure pass, so it has to size itself
    // before `place_clef` can read the width back off it.
    clef.rescale(ctx.scaling * Clef::COURTESY_SCALE);
    ArrangeMachine.place_clef(
        &mut clef,
        ClefAnchor::GapBefore(left),
        part_top + ctx.distance_from_top,
        ctx.scaling,
    );

    Some(clef)
}
