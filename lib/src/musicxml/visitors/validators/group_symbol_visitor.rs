use crate::musicxml::utils::NodeUtils;
use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use crate::score::core::group_symbol::GroupSymbol;
use roxmltree::Node;

/// Reports, on the validation walk, every `<group-symbol>` that
/// [`GroupSymbol::from_mxml`] panics on when the render walk reaches it through
/// [`build_part_list`](crate::score::part_list::builder::build_part_list).
///
/// That reader is strict for the same reason the page-layout one is: which
/// symbol binds a group of staves together is a visible editorial decision, and
/// a value it cannot read is one it would have to invent. An invented one
/// silently redraws the score's structure -- a bracket where the composer asked
/// for a brace. So it panics, and this describes the cause in plain words first,
/// as an `Error`, since nothing downstream mitigates it.
///
/// Only the vocabulary is checked here. Whether the `<part-group>`s that carry
/// these symbols are balanced, and whether the parts they name exist, is
/// [`PartConsistencyVisitor`](crate::musicxml::visitors::validators::part_consistency_visitor::PartConsistencyVisitor)'s
/// business; the two walk the same `<part-list>` and share nothing else.
#[derive(Default)]
pub struct GroupSymbolVisitor {}

impl Visitor<ValidationCtx> for GroupSymbolVisitor {
    fn enter_part_list(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        // Walks the `<part-list>`'s own children rather than descending: a
        // `<part-group>` and a `<score-part>` are siblings there, which is the
        // one thing this and the consistency walk agree about.
        let part_groups = node
            .children()
            .filter(|n| n.is_element() && n.has_tag("part-group"));

        for part_group in part_groups {
            let Some(symbol) = part_group.get_child("group-symbol") else {
                continue;
            };

            let text = symbol.text().unwrap_or("");
            if let Err(err) = text.parse::<GroupSymbol>() {
                ctx.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("<group-symbol> is unreadable: {err}"),
                    at: symbol.range().start,
                });
            }
        }
    }
}
