use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use roxmltree::Node;

/// The four edges every `<page-margins>` must carry.
const MARGIN_EDGES: [&str; 4] = ["left-margin", "right-margin", "top-margin", "bottom-margin"];

/// Reports, on the validation walk, every malformed `<page-layout>` that
/// [`PageLayout::from_mxml`](crate::score::score_defaults::PageLayout::from_mxml)
/// panics on when the render walk reaches it.
///
/// That parser is deliberately strict: a page size or a margin it cannot read is
/// a number it would have to invent, and inventing one silently moves the music
/// on the page. So it panics, and this describes the same causes in plain words
/// first -- as `Error`s, since nothing downstream mitigates them.
///
/// The two places a `<page-layout>` may appear are checked identically, because
/// one parser reads both: `<defaults>` for the score's own geometry, and
/// `<print>` for a mid-document change to it.
///
/// Keep the causes here in step with that function. They are, in the order it
/// hits them: a `<page-width>` / `<page-height>` that is not a number, a
/// `<page-margins>` missing one of its four edges, an edge that is not a number,
/// and a `type` attribute naming something other than the three MusicXML allows.
#[derive(Default)]
pub struct PageLayoutVisitor {}

impl PageLayoutVisitor {
    fn check(node: &Node, ctx: &mut ValidationCtx) {
        let Some(page_layout) = node.get_child("page-layout") else {
            return;
        };

        for name in ["page-width", "page-height"] {
            if let Some(child) = page_layout.get_child(name) {
                Self::check_number(&child, name, ctx);
            }
        }

        for margins in page_layout.children().filter(|n| n.has_tag("page-margins")) {
            // Absent means "both", which is why only a present-and-wrong value
            // is a defect.
            if let Some(kind) = margins.get_attribute("type")
                && !matches!(kind, "both" | "odd" | "even")
            {
                Self::error(
                    ctx,
                    format!(
                        "<page-margins type=\"{kind}\"> is not a page-margins type; \
                         expected \"both\", \"odd\" or \"even\""
                    ),
                    margins.range().start,
                );
            }

            for edge in MARGIN_EDGES {
                match margins.get_child(edge) {
                    Some(child) => Self::check_number(&child, edge, ctx),
                    None => Self::error(
                        ctx,
                        format!("<page-margins> is missing its <{edge}>"),
                        margins.range().start,
                    ),
                }
            }
        }
    }

    /// `node` must hold a number; page geometry has no sensible stand-in.
    fn check_number(node: &Node, name: &str, ctx: &mut ValidationCtx) {
        let text = node.text().unwrap_or("");

        if text.trim().parse::<f32>().is_err() {
            Self::error(
                ctx,
                format!("<{name}> is not a number: {text:?}"),
                node.range().start,
            );
        }
    }

    fn error(ctx: &mut ValidationCtx, message: String, at: usize) {
        ctx.issues.push(ValidationIssue {
            severity: Severity::Error,
            message,
            at,
        });
    }
}

impl Visitor<ValidationCtx> for PageLayoutVisitor {
    fn enter_defaults(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        Self::check(node, ctx);
    }

    fn enter_print(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        Self::check(node, ctx);
    }
}
