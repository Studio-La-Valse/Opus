use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use crate::score::part_list::builder::build_part_list;
use crate::score::part_list::display::format_part_list_tree;
use crate::score::part_list::tree::PartListNode;
use roxmltree::Node;

/// Narrates a walk: what it entered, the part-list tree it read, and what it
/// found by the end.
///
/// This is the only visitor whose output is purely informational. Everything it
/// emits is a `Severity::Info` issue on the context, positioned at the element
/// it describes, so a caller decides for itself whether to show it (the CLI
/// prints them through `print_issues`; a caller with nowhere to print ignores
/// them). Nothing here ever prints, and nothing here inspects a document for
/// defects -- the visitors that do are free of narration in return.
#[derive(Default)]
pub struct LoggingVisitor {
    root: Option<RootElement>,
    part_list: Option<PartList>,
    parts: u32,
    measures: u32,
}

/// The document's root element, remembered at `enter` so `exit` can name and
/// position it without a node of its own.
struct RootElement {
    name: String,
    at: usize,
}

/// The tree read out of `<part-list>`, held between `enter_part_list` (which has
/// the node) and `exit_part_list` (which is where it is reported).
struct PartList {
    nodes: Vec<PartListNode>,
    at: usize,
}

impl LoggingVisitor {
    fn info(ctx: &mut ValidationCtx, message: String, at: usize) {
        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message,
            at,
        });
    }
}

impl Visitor<ValidationCtx> for LoggingVisitor {
    fn enter(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        let name = node.tag_name().name().to_string();
        let at = node.range().start;

        Self::info(ctx, format!("Now entering {name}"), at);
        self.root = Some(RootElement { name, at });
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        let at = node.range().start;
        Self::info(ctx, "Collecting parts...".to_string(), at);

        // Built here because this is where the node is; reported on exit. The
        // tree is the *result* of reading the part-list, so it is logged as
        // that reading finishes rather than as it starts.
        self.part_list = Some(PartList {
            nodes: build_part_list(node),
            at,
        });
    }

    fn exit_part_list(&mut self, ctx: &mut ValidationCtx) {
        if let Some(part_list) = self.part_list.take() {
            Self::info(ctx, format_part_list_tree(&part_list.nodes), part_list.at);
        }
    }

    fn enter_part(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        if self.parts == 0 {
            Self::info(
                ctx,
                "Now traversing parts...".to_string(),
                node.range().start,
            );
        }

        self.parts += 1;
    }

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut ValidationCtx) {
        self.measures += 1;
    }

    fn exit(&mut self, ctx: &mut ValidationCtx) {
        let Some(root) = self.root.take() else {
            return;
        };

        Self::info(
            ctx,
            format!("Done, now gracefully exiting {}", root.name),
            root.at,
        );
        Self::info(
            ctx,
            format!(
                "Found {} part(s) and {} measure(s) total",
                self.parts, self.measures
            ),
            root.at,
        );
    }
}
