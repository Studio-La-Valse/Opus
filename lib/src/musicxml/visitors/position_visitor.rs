use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use crate::score::core::duration_base::BaseDuration;
use roxmltree::Node;

/// Tracks the running position within each measure via `WalkCursor` -- the same
/// state model and transition methods `WalkCursorVisitor` uses for render --
/// and reports when a note/backup/forward pushes it past what the time
/// signature allows, instead of panicking the way render's guard used to.
#[derive(Default)]
pub struct PositionVisitor {
    last_note_at: usize,
    measure_at: usize,
    measure_has_content: bool,
}

impl Visitor<ValidationCtx> for PositionVisitor {
    fn enter_part(&mut self, _node: &Node, ctx: &mut ValidationCtx) {
        ctx.cursor.reset();
    }

    fn enter_measure(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        ctx.cursor.begin_measure();
        self.measure_at = node.range().start;
        self.measure_has_content = false;
    }

    fn exit_measure(&mut self, ctx: &mut ValidationCtx) {
        if !self.measure_has_content {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: "measure has no notes, backup, or forward -- it contributes no duration"
                    .to_string(),
                at: self.measure_at,
            });
        }
    }

    fn enter_attributes(&mut self, element: &Node, ctx: &mut ValidationCtx) {
        for node in element.children() {
            if node.has_tag_name("divisions")
                && let Some(v) = node.text().and_then(|s| s.trim().parse::<u32>().ok())
            {
                ctx.cursor.set_divisions(v);
            }

            if node.has_tag_name("time") {
                for child in node.children() {
                    if child.has_tag_name("beats")
                        && let Some(v) = child.text().and_then(|s| s.trim().parse::<u8>().ok())
                    {
                        ctx.cursor.set_beats(v);
                    }

                    if child.has_tag_name("beat-type") {
                        match child
                            .text()
                            .and_then(|s| s.trim().parse::<u8>().ok())
                            .map(BaseDuration::try_from)
                        {
                            Some(Ok(beat_type)) => ctx.cursor.set_beat_type(beat_type),
                            _ => ctx.issues.push(ValidationIssue {
                                severity: Severity::Error,
                                message: format!(
                                    "<beat-type> has an invalid or missing value: {:?}",
                                    child.text()
                                ),
                                at: child.range().start,
                            }),
                        }
                    }
                }
            }
        }
    }

    fn enter_backup(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        self.measure_has_content = true;

        let Some(duration) = node
            .get_child("duration")
            .and_then(|n| n.text())
            .and_then(|s| s.trim().parse::<u32>().ok())
        else {
            return;
        };

        if ctx.cursor.apply_backup(duration).is_err() {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!("<backup> duration {duration} exceeds the current position"),
                at: node.range().start,
            });
        }
    }

    fn enter_forward(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        self.measure_has_content = true;

        let Some(duration) = node
            .get_child("duration")
            .and_then(|n| n.text())
            .and_then(|s| s.trim().parse::<u32>().ok())
        else {
            return;
        };

        if ctx.cursor.apply_forward(duration).is_err() {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!("<forward> duration {duration} overflowed the position"),
                at: node.range().start,
            });
            return;
        }

        self.check_position(ctx, node.range().start);
    }

    fn enter_note(&mut self, element: &Node, ctx: &mut ValidationCtx) {
        self.last_note_at = element.range().start;
        self.measure_has_content = true;

        let is_chord = element.has_child("chord");
        let is_grace = element.has_child("grace");
        let is_cue = element.has_child("cue");
        let duration = element
            .get_child("duration")
            .and_then(|n| n.text())
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        if ctx
            .cursor
            .enter_note(duration, is_chord, is_grace, is_cue)
            .is_err()
        {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "chord note duration exceeds the current position".to_string(),
                at: self.last_note_at,
            });
        }
    }

    fn exit_note(&mut self, ctx: &mut ValidationCtx) {
        if ctx.cursor.exit_note().is_err() {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "note duration overflowed the position".to_string(),
                at: self.last_note_at,
            });
            return;
        }

        self.check_position(ctx, self.last_note_at);
    }
}

impl PositionVisitor {
    fn check_position(&self, ctx: &mut ValidationCtx, at: usize) {
        if ctx.cursor.position_exceeds_measure() {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "position {} exceeds the divisions available in a measure with {} beats of type {}",
                    ctx.cursor.position, ctx.cursor.beats, ctx.cursor.beat_type
                ),
                at,
            });
        }
    }
}
