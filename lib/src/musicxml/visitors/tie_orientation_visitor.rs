use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use roxmltree::Node;
use std::collections::HashMap;

/// Reports a tie whose notated orientation is written only on its `stop`.
///
/// The renderer takes a tie's side from the note it *leaves*: see
/// [`tie`](crate::musicxml::visitors::content_visitor) in `ContentVisitor`,
/// which reads `<tied orientation=…>` / `placement=…` off that note alone and
/// leaves [`TieSide::infer`](crate::score::visual::tie::TieSide::infer) to
/// settle the rest. That is what lets the content walk record a tie without
/// correlating two notes -- but it means an orientation an exporter wrote only
/// on the far end is not seen, and the arc curves the way its stem implies
/// instead.
///
/// A `Warning` and not an `Error`: the tie is still drawn, still in a legal
/// place, just not on the side the document asked for.
///
/// Rare but real. Of the bundled samples exactly two carry one -- one in
/// `Dichterliebe01.musicxml` among its 52 stops, one in `FaurReveSample.musicxml`
/// among its 6 -- which is why this pairs ties up rather than flagging every
/// orientation on a `stop`: an orientation that merely repeats what the start
/// already said costs nothing and is not worth a warning.
///
/// The one case it stays quiet about is a `continue`, which both ends a tie and
/// starts the next. Its orientation is read, because the note carrying it is the
/// start of the arc leaving it.
#[derive(Default)]
pub struct TieOrientationVisitor {
    /// Tie starts still waiting for their stop, and whether each named an
    /// orientation of its own.
    pending: HashMap<TieKey, bool>,
}

/// What MusicXML pairs a tie on: the same pitch, in the same voice, on the same
/// staff.
///
/// Read straight off the `<note>` rather than from
/// [`ValidationCtx::cursor`](crate::musicxml::validate::ValidationCtx), which
/// tracks position but not voice or staff. Both default to 1 when the element is
/// absent, exactly as MusicXML specifies, and the pitch is kept as written --
/// nothing here needs to understand it, only to tell two pitches apart.
type TieKey = (String, String, String);

fn tie_key(node: &Node) -> Option<TieKey> {
    let text = |name: &str, default: &str| {
        node.get_child(name)
            .and_then(|n| n.text())
            .map(|t| t.trim().to_string())
            .unwrap_or_else(|| default.to_string())
    };

    let pitch = node.get_child("pitch")?;
    let part = |name: &str| {
        pitch
            .get_child(name)
            .and_then(|n| n.text())
            .unwrap_or("")
            .trim()
    };

    Some((
        text("staff", "1"),
        text("voice", "1"),
        format!("{}{}{}", part("step"), part("alter"), part("octave")),
    ))
}

/// Whether any `<tied>` or `<tie>` on this note declares one of `types`, and
/// whether any `<tied>` that does also names an orientation.
fn declares(node: &Node, types: [&str; 2]) -> (bool, bool) {
    let tied = node
        .get_child("notations")
        .map(|n| n.get_children("tied"))
        .unwrap_or_default();
    let ties = node.get_children("tie");

    let wanted = |n: &Node| n.get_attribute("type").is_some_and(|t| types.contains(&t));

    let declared = tied.iter().chain(ties.iter()).any(wanted);
    let oriented = tied.iter().filter(|n| wanted(n)).any(|n| {
        n.get_attribute("orientation").is_some() || n.get_attribute("placement").is_some()
    });

    (declared, oriented)
}

impl Visitor<ValidationCtx> for TieOrientationVisitor {
    fn enter_note(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        let Some(key) = tie_key(node) else {
            return;
        };

        // The stop is handled before the start, so a chained a-b-c tie works:
        // `b` carries both and consumes `a`'s entry before registering its own.
        let (stops, stop_oriented) = declares(node, ["stop", "continue"]);
        if stops
            && let Some(start_oriented) = self.pending.remove(&key)
            && stop_oriented
            && !start_oriented
        {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: "<tied type=\"stop\"> names an orientation its matching start does \
                          not; a tie is drawn on the side its starting note asks for, so this \
                          one curves away from its stem instead"
                    .to_string(),
                at: node.range().start,
            });
        }

        let (starts, start_oriented) = declares(node, ["start", "continue"]);
        if starts {
            self.pending.insert(key, start_oriented);
        }
    }

    /// Ties never cross parts, and a partwise document finishes one part before
    /// starting the next, so anything still pending here never found its stop.
    /// That is a defect of its own, and not this rule's to report.
    fn exit_part(&mut self, _ctx: &mut ValidationCtx) {
        self.pending.clear();
    }
}
