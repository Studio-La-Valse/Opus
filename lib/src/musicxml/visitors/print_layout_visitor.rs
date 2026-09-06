use crate::musicxml::utils::NodeUtils;
use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::score_defaults::PageLayout;
use roxmltree::Node;

/// Records the `<page-layout>` a mid-document `<print>` carries.
///
/// `<defaults><page-layout>` sets the geometry the score starts with;
/// a `<print>` may then change the page size or the margins from its own page
/// onward, which is how an export gives one page a different top margin from the
/// next. ActorPreludeSample does exactly that on every page.
///
/// Only the recording happens here. The fold -- an override applies from its
/// page onward, and changes only what it names -- lives in
/// [`ScoreDefaults::resolve_page`](crate::score::score_defaults::ScoreDefaults::resolve_page),
/// so that `Page::resolve_layout` asks one question and gets one answer.
///
/// Must be chained after
/// [`WalkCursorVisitor`](crate::musicxml::visitors::walk_cursor_visitor::WalkCursorVisitor),
/// which is what advances the page number: a `<print new-page="yes">` carrying a
/// `<page-layout>` describes the page it opens, not the one it closes.
pub struct PrintLayoutVisitor {}

impl<'a> Visitor<WalkerCtx<'a>> for PrintLayoutVisitor {
    fn enter_print(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        let Some(page_layout) = node.get_child("page-layout") else {
            return;
        };

        let layout = PageLayout::from_mxml(&page_layout);
        let page_number = ctx.cursor.page.page_number;

        // Every part repeats the same `<print>` elements, so this is written
        // once per part with the same values. Folding rather than replacing
        // also keeps a page described by more than one `<print>` intact.
        ctx.layout
            .page_overrides
            .entry(page_number)
            .or_default()
            .apply(&layout);
    }
}
