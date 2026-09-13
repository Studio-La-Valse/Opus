use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use roxmltree::Node;

/// Reports, on the validation walk, every malformed `<staff-details>` that
/// [`WalkCursorVisitor::enter_staff_details`](crate::musicxml::visitors::builders::walk_cursor_visitor::WalkCursorVisitor)
/// panics on when the render walk reaches it.
///
/// That reader is strict for the same reason the page-layout one is: a staff it
/// cannot size or count lines for is a number it would have to invent, and an
/// invented one silently changes what the staff looks like. So it panics, and
/// this describes the same causes in plain words first, as `Error`s -- nothing
/// downstream mitigates them.
///
/// Keep the causes here in step with that function. They are: a `number`
/// attribute that is not a staff number, a `<staff-lines>` that is not a count,
/// a `<staff-size>` that is not a number, and a `scaling` attribute on it that
/// is not a number either.
#[derive(Default)]
pub struct StaffDetailsVisitor {}

impl StaffDetailsVisitor {
    fn error(ctx: &mut ValidationCtx, message: String, at: usize) {
        ctx.issues.push(ValidationIssue {
            severity: Severity::Error,
            message,
            at,
        });
    }
}

impl Visitor<ValidationCtx> for StaffDetailsVisitor {
    fn enter_staff_details(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        if let Some(number) = node.get_attribute("number")
            && number.trim().parse::<u32>().is_err()
        {
            Self::error(
                ctx,
                format!("<staff-details number=\"{number}\"> is not a staff number"),
                node.range().start,
            );
        }

        // A staff may be drawn with any number of lines, none included --
        // `<staff-lines>0</staff-lines>` is how a document asks for a staff
        // with no lines at all -- but it has to be a whole count of them.
        if let Some(lines) = node.get_child("staff-lines") {
            let text = lines.text().unwrap_or("");
            if text.trim().parse::<usize>().is_err() {
                Self::error(
                    ctx,
                    format!("<staff-lines> is not a number of lines: {text:?}"),
                    lines.range().start,
                );
            }
        }

        if let Some(size) = node.get_child("staff-size") {
            let text = size.text().unwrap_or("");
            if text.trim().parse::<f32>().is_err() {
                Self::error(
                    ctx,
                    format!("<staff-size> is not a number: {text:?}"),
                    size.range().start,
                );
            }

            if let Some(scaling) = size.get_attribute("scaling")
                && scaling.trim().parse::<f32>().is_err()
            {
                Self::error(
                    ctx,
                    format!("<staff-size scaling=\"{scaling}\"> is not a number"),
                    size.range().start,
                );
            }
        }
    }
}
