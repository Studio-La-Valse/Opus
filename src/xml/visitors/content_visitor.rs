use crate::score::visual::visual_score::VisualScore;
use crate::xml::walker_ctx::WalkerCtx;
use crate::{Layout, LayoutCtx, Visitor};
use roxmltree::Node;

pub struct ContentVisitor {}

impl ContentVisitor {}

impl Visitor for ContentVisitor {
    fn enter(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, ctx: &mut WalkerCtx) {}

    fn exit(&mut self, ctx: &mut WalkerCtx) {}
}
