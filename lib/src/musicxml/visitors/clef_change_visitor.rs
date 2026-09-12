use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::clef::ClefChange;
use crate::smufl::glyphs::clef::Clef as SmuflClef;
use roxmltree::Node;
use std::collections::BTreeMap;

/// Records mid-measure clef changes as [`ClefChange`]s on the score.
///
/// Kept separate from `ContentVisitor` -- which builds the notes and rests
/// themselves -- for the reason `TieVisitor` is: one populates the visual tree,
/// this one records something that has to be positioned against a node of it
/// without belonging to that node. The only thing the two must agree on is which
/// note is which, and that comes from `WalkCursor::note_id` rather than from
/// either visitor.
///
/// A clef written at position 0 is not a change part-way through anything: it is
/// the measure's opening clef, drawn at the *previous* measure's barline, and
/// `ContentVisitor` puts it there. Only clefs past the start of a measure are
/// this visitor's business.
pub struct ClefChangeVisitor {
    /// Clef changes seen so far in the current measure, waiting for the note or
    /// rest they are drawn in front of. Keyed by staff because a part's staves
    /// share one stream of `<note>` elements, and a change on staff 2 waits for
    /// a staff-2 note however many staff-1 notes come first.
    pending: BTreeMap<StaffIdx, SmuflClef>,
    /// Changes anchored so far in the current part, flushed in `exit_part`.
    changes: Vec<ClefChange>,
}

impl ClefChangeVisitor {
    pub fn new() -> Self {
        Self {
            pending: BTreeMap::new(),
            changes: Vec::new(),
        }
    }
}

impl Default for ClefChangeVisitor {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> Visitor<WalkerCtx<'a>> for ClefChangeVisitor {
    /// Holds a mid-measure clef until the note or rest it precedes comes along.
    ///
    /// The glyph is picked here rather than during the arrange because the choice
    /// depends on how many lines the staff is drawn with, which the walk knows
    /// and the arrange would have to look up again.
    fn enter_clef(&mut self, _node: &Node, ctx: &mut WalkerCtx) {
        if ctx.cursor.position == 0 {
            return;
        }

        let staff_idx: StaffIdx = ctx.cursor.staff.number;
        let Some(clef) = ctx.cursor.staff.active_clef.get(&staff_idx) else {
            return;
        };
        let staff_lines = ctx.cursor.staff.lines(&staff_idx);

        self.pending
            .insert(staff_idx, ctx.font.clef(clef, staff_lines));
    }

    /// Anchors whatever this note's staff was waiting for to this note.
    ///
    /// No attempt is made here to check that the note will actually be built:
    /// `ContentVisitor` drops a note with no `default-x`, exactly as it does for
    /// a tie endpoint. A change naming a note that never made it into the tree
    /// finds no anchor and is discarded by
    /// [`arrange_clef_changes`](crate::score::visual::clef_change_arranger::arrange_clef_changes).
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
