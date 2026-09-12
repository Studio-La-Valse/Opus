use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use roxmltree::Node;
use std::collections::BTreeMap;

/// Reports a `<beam>` level that is opened and never closed.
///
/// A well-formed beam level runs `begin`, any number of `continue`s, then
/// `end`. A level that never ends leaves the renderer with a beam that has a
/// left edge and no right one, which used to be a panic and is now inferred --
/// the level is taken to run to the last note that carries it (see
/// `beam_level_ends_at` in
/// [`beam_arranger`](crate::score::visual::beam_arranger)). Since the document
/// is repairable rather than un-renderable, this is a `Warning` and not an
/// `Error`.
///
/// Notably *not* caught by the rebeam pass: that compares how many beams a note
/// declares against how many its duration warrants, so a group whose counts are
/// right but whose types are inconsistent passes straight through it.
///
/// Levels are tracked per voice, and separately for grace notes, because that is
/// how [`arrange_beams`](crate::score::visual::beam_arranger::arrange_beams)
/// groups the chords it beams. A group also ends at a note carrying no `<beam>`
/// at all.
///
/// A barline is *not* such a boundary. It was, while beams were a
/// `PartMeasure`'s own business and a measure was as far as one could see, so a
/// level left open at the end of a measure was stranded by construction. Now
/// that a run is a whole part's, a group legally spans a barline and closing
/// every open level at `exit_measure` would report each such beam as a defect.
/// `exit_part` still catches the levels that really are never closed.
#[derive(Default)]
pub struct BeamGroupVisitor {
    /// Per group, the levels currently open and the byte offset of the `<beam>`
    /// that opened each.
    open: BTreeMap<GroupKey, BTreeMap<u32, usize>>,
}

/// What [`arrange_beams`](crate::score::visual::beam_arranger::arrange_beams)
/// beams together: one voice's notes, with grace notes kept apart from the rest.
type GroupKey = (u32, bool);

impl BeamGroupVisitor {
    /// Reports every level still open in `key`'s group and forgets them, on the
    /// understanding that the group has just ended.
    fn close_group(&mut self, key: GroupKey, ctx: &mut ValidationCtx) {
        let Some(levels) = self.open.remove(&key) else {
            return;
        };

        for (number, at) in levels {
            Self::report(ctx, number, at);
        }
    }

    fn close_all(&mut self, ctx: &mut ValidationCtx) {
        for key in self.open.keys().copied().collect::<Vec<_>>() {
            self.close_group(key, ctx);
        }
    }

    fn report(ctx: &mut ValidationCtx, number: u32, at: usize) {
        ctx.issues.push(ValidationIssue {
            severity: Severity::Warning,
            message: format!(
                "<beam number=\"{number}\"> opens a beam level that is never closed; \
                 it is drawn as far as the last note carrying level {number}"
            ),
            at,
        });
    }
}

impl Visitor<ValidationCtx> for BeamGroupVisitor {
    fn enter_note(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        // What gets beamed is a chord, not a note. The second and further notes
        // of a chord carry `<chord/>` and no `<beam>` of their own -- only the
        // first note spells the chord's beams -- so they are not elements of the
        // sequence at all. Rests likewise: they never become chords, so a rest
        // between two beamed notes does not interrupt the run the renderer sees.
        if node.has_child("chord") || node.has_child("rest") {
            return;
        }

        let key: GroupKey = (voice_of(node), node.has_child("grace"));
        let beams = node.get_children("beam");

        // A chord carrying no beams closes the group it lands in, the way
        // `create_beam_groups` treats a standalone chord.
        if beams.is_empty() {
            self.close_group(key, ctx);
            return;
        }

        for beam in beams {
            let Some(number) = beam
                .get_attribute("number")
                .and_then(|n| n.trim().parse::<u32>().ok())
            else {
                continue;
            };

            match beam.text().map(str::trim) {
                Some("begin") => {
                    let levels = self.open.entry(key).or_default();

                    // Opening a level that is already open closes nothing --
                    // the first one is still unaccounted for.
                    if let Some(previous) = levels.insert(number, beam.range().start) {
                        Self::report(ctx, number, previous);
                    }
                }
                Some("end") => {
                    if let Some(levels) = self.open.get_mut(&key) {
                        levels.remove(&number);
                    }
                }
                // `continue` and the two hooks neither open nor close a level.
                _ => {}
            }
        }

        // Level 1 is the group: once it closes, nothing later can close the
        // levels above it.
        let group_open = self
            .open
            .get(&key)
            .is_some_and(|levels| levels.contains_key(&1));

        if !group_open {
            self.close_group(key, ctx);
        }
    }

    fn exit_part(&mut self, ctx: &mut ValidationCtx) {
        self.close_all(ctx);
    }
}

/// A note's `<voice>`, defaulting to 1 the way an absent voice is read
/// everywhere else.
fn voice_of(node: &Node) -> u32 {
    node.get_child("voice")
        .and_then(|n| n.text())
        .and_then(|text| text.trim().parse().ok())
        .unwrap_or(1)
}
