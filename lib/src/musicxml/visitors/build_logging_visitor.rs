use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::part_list::display::format_part_list_tree;
use roxmltree::Node;

/// Narrates a build walk: what it entered, the part-list tree it read, and what
/// it found by the end.
///
/// This is where *all* narration lives. The validation walk reports only what is
/// wrong with a document -- warnings and errors -- so that its output is a
/// verdict and nothing else, and everything informational is said here instead,
/// on the walk that actually turns the document into a score.
///
/// The part-list tree is the reason this has to be the build walk rather than
/// the validation one: only this walk has the tree, as `SetupVisitor`'s work on
/// `ScoreDefaults`. Validation would have to build it itself, and the builder is
/// entitled to panic on exactly the documents validation exists to survive.
///
/// Messages go onto `WalkerCtx::messages` as `Severity::Info`, positioned at the
/// element they describe, the way `ValidationCtx::issues` carries a validation
/// walk's findings. The visitor never prints; a caller with nowhere to print
/// simply drops the messages.
#[derive(Default)]
pub struct BuildLoggingVisitor {
    root: Option<RootElement>,
    part_list_at: usize,
    parts: u32,
    measures: u32,
    notes: u32,
    rests: u32,
    backups: u32,
    forwards: u32,
}

/// The document's root element, remembered at `enter` so `exit` can name and
/// position it without a node of its own.
struct RootElement {
    name: String,
    at: usize,
}

impl BuildLoggingVisitor {
    fn info(ctx: &mut WalkerCtx, message: String, at: usize) {
        ctx.messages.push(ValidationIssue {
            severity: Severity::Info,
            message,
            at,
        });
    }
}

impl Visitor<WalkerCtx<'_>> for BuildLoggingVisitor {
    fn enter(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let name = node.tag_name().name().to_string();
        let at = node.range().start;

        Self::info(ctx, format!("Now entering {name}"), at);
        self.root = Some(RootElement { name, at });
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.part_list_at = node.range().start;
        Self::info(ctx, "Collecting parts...".to_string(), self.part_list_at);
    }

    /// Reported once the part-list has been read rather than as it is entered,
    /// since it is `SetupVisitor`'s `enter_part_list` -- chained ahead of this
    /// one -- that puts the tree on the context in the first place.
    fn exit_part_list(&mut self, ctx: &mut WalkerCtx) {
        let tree = format_part_list_tree(&ctx.layout.part_list);
        Self::info(ctx, tree, self.part_list_at);
    }

    fn enter_part(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        if self.parts == 0 {
            Self::info(
                ctx,
                "Now traversing parts...".to_string(),
                node.range().start,
            );
        }

        self.parts += 1;
    }

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.measures += 1;
    }

    fn enter_note(&mut self, node: &Node, _ctx: &mut WalkerCtx) {
        self.notes += 1;

        if node.has_child("rest") {
            self.rests += 1;
        }
    }

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.backups += 1;
    }

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {
        self.forwards += 1;
    }

    fn exit(&mut self, ctx: &mut WalkerCtx) {
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
        Self::info(
            ctx,
            format!(
                "found {} note(s) ({} rest(s)), {} backup(s), {} forward(s)",
                self.notes, self.rests, self.backups, self.forwards
            ),
            root.at,
        );
    }
}
