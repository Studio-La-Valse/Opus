use crate::musicxml::utils::NodeUtils;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::core::pitch::Pitch;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::step::Step;
use crate::score::core::voice::Voice;
use crate::score::visual::note::NoteId;
use crate::score::visual::tie::{Tie, TieSide};
use roxmltree::Node;
use std::collections::HashMap;

/// What MusicXML says a tie connects: the same pitch, in the same voice, on the
/// same staff. Ties never cross parts, and the walk finishes one part before
/// starting the next, so the part is implicit in the matcher's lifetime.
type TieKey = (StaffIdx, Voice, Pitch);

/// A tie start still waiting for its stop.
#[derive(Copy, Clone)]
struct PendingTie {
    id: NoteId,
    side: Option<TieSide>,
}

/// Pairs up `<tied>` / `<tie>` endpoints into [`Tie`]s on the score.
///
/// Kept separate from `ContentVisitor` -- which builds the notes themselves --
/// because they are separate concerns: one populates the visual tree, this one
/// records a relation between two of its nodes. The only thing they must agree
/// on is which note is which, and that comes from `WalkCursor::note_id` rather
/// than from either visitor, so neither can drift from the other.
///
/// This does mean re-reading `<pitch>` here after `ContentVisitor` has already
/// read it. That is a handful of integer parses per note and buys the two
/// visitors full independence.
pub struct TieVisitor {
    /// Tie starts still waiting for their stop, within the current part.
    pending: HashMap<TieKey, PendingTie>,
    /// Ties matched so far in the current part, flushed in `exit_part`.
    ties: Vec<Tie>,
}

impl TieVisitor {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
            ties: Vec::new(),
        }
    }
}

impl Default for TieVisitor {
    fn default() -> Self {
        Self::new()
    }
}

/// The pitch a tie is keyed on, or `None` for an unpitched note.
///
/// Unpitched notes (percussion) are dropped by `ContentVisitor` too, so a tie
/// touching one has no notehead to attach to either end of.
fn tie_pitch(node: &Node) -> Option<Pitch> {
    let pitch = node.get_child("pitch")?;
    let step = pitch.get_child("step")?.text()?;
    let alter = pitch
        .get_child("alter")
        .and_then(|n| n.text())
        .and_then(|t| t.trim().parse().ok())
        .unwrap_or(0);
    let octave = pitch.get_child("octave")?.text()?.trim().parse().ok()?;

    Some(Pitch {
        step: Step::parse(step, alter),
        octave,
    })
}

impl<'a> Visitor<WalkerCtx<'a>> for TieVisitor {
    /// Records the ties this note takes part in.
    ///
    /// MusicXML spells a tie twice: `<tie>` is a direct child of `<note>` and
    /// carries the sounding semantics, `<notations><tied>` is the notated curve
    /// and carries the placement attributes. Exports usually emit both, but not
    /// always, so a note counts as starting or stopping a tie if *either* says
    /// so.
    ///
    /// The stop is handled before the start, so a chained a-b-c tie works: `b`
    /// carries both, consumes `a`'s pending entry, then registers itself for
    /// `c`. `<tied type="continue">` means exactly that pair.
    ///
    /// `<tied number=…>` is ignored. It exists to disambiguate overlapping
    /// spanners, but a tie is already pinned down by its pitch -- unlike a slur,
    /// which will need it.
    ///
    /// No attempt is made here to check that the note will actually be built
    /// (`ContentVisitor` also drops notes with no `default-x`). A tie naming a
    /// note that never made it into the tree is discarded later by
    /// [`split_tie`](crate::score::visual::tie::split_tie), which has
    /// to be defensive about that anyway.
    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let Some(pitch) = tie_pitch(node) else {
            return;
        };
        let key = (ctx.cursor.staff.number, ctx.cursor.voice, pitch);
        let id = ctx.cursor.note_id;

        let tied: Vec<Node> = node
            .get_child("notations")
            .map(|n| n.get_children("tied"))
            .unwrap_or_default();
        let ties: Vec<Node> = node.get_children("tie");

        let has_type = |t: &str| {
            tied.iter().chain(ties.iter()).any(|n| {
                let ty = n.get_attribute("type");
                ty == Some(t) || ty == Some("continue")
            })
        };

        // Orientation belongs to the tie, and exports put it on whichever
        // endpoint they please -- most often the start. Carry the start's along
        // and let the stop's fill in only when the start had none.
        let side = tied.iter().find_map(|n| {
            TieSide::parse(n.get_attribute("orientation"), n.get_attribute("placement"))
        });

        if has_type("stop")
            && let Some(pending) = self.pending.remove(&key)
        {
            self.ties.push(Tie {
                start: pending.id,
                end: id,
                side: pending.side.or(side),
            });
        }

        if has_type("start") {
            self.pending.insert(key, PendingTie { id, side });
        }
    }

    /// Hands this part's ties to the score and resets the matcher.
    ///
    /// A partwise document finishes each part before starting the next and ties
    /// never cross parts, so any start still pending here never found its stop
    /// -- a tie into a repeat, or a truncated export. Those are discarded rather
    /// than drawn to nowhere.
    fn exit_part(&mut self, ctx: &mut WalkerCtx) {
        ctx.visual_score.ties.append(&mut self.ties);
        self.pending.clear();
    }
}
