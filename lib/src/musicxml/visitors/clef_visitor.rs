use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::clef::{Clef, ClefChange};
use crate::smufl::glyphs::clef::Clef as SmuflClef;
use roxmltree::Node;
use std::collections::BTreeMap;

/// Every clef the engine draws, from the one a staff opens a system with to a
/// change written in front of a note.
///
/// MusicXML spells all three with the same `<clef>` element and leaves the
/// reader to tell them apart by where it appears, so splitting them across
/// visitors would mean answering that one question in two places. They are one
/// concern:
///
/// - **The opening clef**, at the left of the first measure of every system.
///   `WalkCursor` tracks which clef each staff is reading in, and
///   [`Part::set_opening_clef`](crate::score::visual::part::Part) stamps it onto
///   that measure. Note this is per *system*, not per clef element -- a staff
///   opens every system with a clef whether or not the document repeats it.
/// - **The courtesy at a barline**, where a measure opens in a new clef and the
///   *previous* measure announces it at its own right edge. It belongs to that
///   previous `StaffMeasure` and is stored on it.
/// - **The mid-measure change**, which has no barline to sit at and hangs off
///   the note or rest that follows instead. It belongs to a staff but can only
///   be positioned against something in a `PartMeasure`, so it goes flat on
///   `Score::clef_changes` and is placed by
///   [`clef_change_arranger`](crate::score::visual::clef_change_arranger); see
///   [`ClefChange`].
///
/// Kept out of `ContentVisitor`, which builds the notes and rests themselves,
/// for the reason `TieVisitor` is: the two record different things about the
/// same document, and the only fact they must agree on is which note is which --
/// which comes from `WalkCursor::note_id` rather than from either of them.
pub struct ClefVisitor {
    /// Mid-measure changes seen so far, waiting for the note or rest they are
    /// drawn in front of. Keyed by staff because a part's staves share one
    /// stream of `<note>` elements, and a change on staff 2 waits for a staff-2
    /// note however many staff-1 notes come first.
    pending: BTreeMap<StaffIdx, SmuflClef>,
    /// Changes anchored so far in the current part, flushed in `exit_part`.
    changes: Vec<ClefChange>,
}

impl ClefVisitor {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            changes: Vec::new(),
        }
    }

    /// The glyph for whatever clef the staff under the cursor is now reading in.
    ///
    /// Picked here rather than during the arrange because the choice depends on
    /// how many lines the staff is drawn with, which the walk knows and the
    /// arrange would have to look up again. `WalkCursorVisitor` stamps the
    /// active clef before any visitor chained after it sees the element, so the
    /// lookup only comes up empty for a `<clef>` nothing tracked -- which the
    /// cursor has no arm for either.
    fn active_glyph(&self, staff_idx: &StaffIdx, ctx: &WalkerCtx) -> Option<SmuflClef> {
        let clef = ctx.cursor.staff.active_clef.get(staff_idx)?;
        let staff_lines = ctx.cursor.staff.lines(staff_idx);

        Some(ctx.font.clef(clef, staff_lines))
    }
}

impl Default for ClefVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Visitor<WalkerCtx<'a>> for ClefVisitor {
    /// Sorts one `<clef>` into the two cases the document distinguishes only by
    /// position: at the start of a measure it is the clef that measure opens in,
    /// announced at the previous barline; past the start it is a change, and
    /// waits for a note.
    fn enter_clef(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        let staff_idx: StaffIdx = ctx.cursor.staff.number;
        let Some(glyph) = self.active_glyph(&staff_idx, ctx) else {
            return;
        };

        if ctx.cursor.position > 0 {
            self.pending.insert(staff_idx, glyph);
            return;
        }

        // The courtesy belongs to the measure *before* this one, which is where
        // it is drawn. The first measure of the score has no such measure, and
        // needs none: its clef is simply the one it opens with.
        let measure_number = ctx.cursor.measure.number;
        let part_id = ctx.cursor.part_id.as_str();

        if measure_number > 1
            && let Some(previous_measure) =
                ctx.visual_score
                    .locate_staff_measure_mut(part_id, &staff_idx, measure_number - 1)
        {
            previous_measure.clef_end = Some(Clef::new(glyph));
        }
    }

    /// Anchors whatever this note's staff was waiting for to this note.
    ///
    /// No attempt is made here to check that the note will actually be built:
    /// `ContentVisitor` drops a note with no `default-x`, exactly as it does for
    /// a tie endpoint. A change naming a note that never made it into the tree
    /// finds no anchor and is discarded by
    /// [`ClefChangeArranger`](crate::score::visual::clef_change_arranger::ClefChangeArranger).
    fn enter_note(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        let staff_idx: StaffIdx = ctx.cursor.staff.number;

        if let Some(clef) = self.pending.remove(&staff_idx) {
            self.changes.push(ClefChange {
                anchor: ctx.cursor.note_id,
                staff: staff_idx,
                clef,
            });
        }
    }

    /// Gives every staff of this part the clef it opens its system in.
    ///
    /// Runs on every measure rather than only on the first of a system, because
    /// `set_opening_clef` writes to each staff's *first* measure -- which is the
    /// first one of the system, the tree having been split into systems already.
    /// It has to run after the layout walk's `consolidate_measure_width`, so
    /// that every staff has the measures it is about to be indexed by.
    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        let page_number = ctx.cursor.page.page_number;
        let system_index = ctx.cursor.system.index;
        let part_id = ctx.cursor.part_id.clone();
        let assignment = ctx.layout.lookup(&part_id).unwrap();

        let part = ctx.visual_score.locate_or_create_part(
            page_number,
            system_index,
            &assignment,
            &part_id,
        );

        part.set_opening_clef(&ctx.cursor.staff.opening_clef, |clef, staff_lines| {
            Clef::new(ctx.font.clef(&clef, staff_lines))
        });
    }

    /// Hands this part's changes to the score and resets.
    ///
    /// A partwise document finishes each part before starting the next, so a
    /// change still pending here never found a note to sit in front of -- a clef
    /// written after the last note of a part, which nothing reads.
    fn exit_part(&mut self, ctx: &mut WalkerCtx) {
        ctx.visual_score.clef_changes.append(&mut self.changes);
        self.pending.clear();
    }
}
