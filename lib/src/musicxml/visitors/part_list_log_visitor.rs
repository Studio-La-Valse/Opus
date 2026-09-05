use crate::musicxml::visitor::Visitor;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::part_list::display::format_part_list_tree;

/// Logs the part-list tree a walk produced, as `├──`/`└──` ASCII art.
///
/// It reports the *result* -- `ScoreDefaults::part_list` as the rest of the
/// pipeline will read it -- and knows nothing about how the tree was built, so
/// changing the builder can never break the log or vice versa.
///
/// The visitor never prints. It writes through a `sink` its caller supplies,
/// which is what lets `lib` stay presentation-free (see `score::engrave`) and
/// lets a caller with nowhere to print -- the wasm bindings -- simply not chain
/// it at all.
pub struct PartListLogVisitor<F: FnMut(&str)> {
    enabled: bool,
    sink: F,
}

impl<F: FnMut(&str)> PartListLogVisitor<F> {
    /// `enabled` is the caller's own switch (the CLI's `--debug`); when false
    /// the tree is never even formatted.
    pub fn new(enabled: bool, sink: F) -> Self {
        PartListLogVisitor { enabled, sink }
    }
}

impl<'a, F: FnMut(&str)> Visitor<WalkerCtx<'a>> for PartListLogVisitor<F> {
    /// Logs at the end of the walk rather than on `<part-list>` itself: a
    /// `<part>` whose id was never declared in the part-list is appended by
    /// `ScoreDefaults::ensure_part` as the walk goes on, so the tree is only
    /// final once every part has been seen.
    fn exit(&mut self, ctx: &mut WalkerCtx) {
        if !self.enabled {
            return;
        }

        (self.sink)(&format_part_list_tree(&ctx.layout.part_list));
    }
}
