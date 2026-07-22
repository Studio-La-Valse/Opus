use crate::core::chain::Chain;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;

pub trait Visitor: Sized {
    fn enter(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_work(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_defaults(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn exit_defaults(&mut self, ctx: &mut WalkerCtx);

    fn enter_part_list(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_part(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_measure(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_print(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_attributes(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_clef(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_staff_details(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_backup(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_forward(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx);

    fn exit_note(&mut self, ctx: &mut WalkerCtx);

    fn exit_measure(&mut self, ctx: &mut WalkerCtx);

    fn exit_part(&mut self, ctx: &mut WalkerCtx);

    fn exit(&mut self, ctx: &mut WalkerCtx);

    fn uses<C: Visitor>(self, callback: C) -> Chain<Self, C> {
        Chain::new(self, callback)
    }
}

pub struct DefaultVisitor {}

impl DefaultVisitor {}

impl Visitor for DefaultVisitor {
    fn enter(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, _ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_clef(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_staff_details(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_backup(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_forward(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, _node: &Node, _ctx: &mut WalkerCtx) {}

    fn exit_note(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, _ctx: &mut WalkerCtx) {}

    fn exit(&mut self, _ctx: &mut WalkerCtx) {}
}

impl<A: Visitor, B: Visitor> Visitor for Chain<A, B> {
    fn enter(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter(node, ctx);
        self.b.enter(node, ctx);
    }

    fn enter_work(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_work(node, ctx);
        self.b.enter_work(node, ctx);
    }

    fn enter_defaults(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_defaults(node, ctx);
        self.b.enter_defaults(node, ctx);
    }

    fn exit_defaults(&mut self, ctx: &mut WalkerCtx) {
        self.a.exit_defaults(ctx);
        self.b.exit_defaults(ctx);
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_part_list(node, ctx);
        self.b.enter_part_list(node, ctx);
    }

    fn enter_part(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_part(node, ctx);
        self.b.enter_part(node, ctx);
    }

    fn enter_measure(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_measure(node, ctx);
        self.b.enter_measure(node, ctx);
    }

    fn enter_print(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_print(node, ctx);
        self.b.enter_print(node, ctx);
    }

    fn enter_attributes(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_attributes(node, ctx);
        self.b.enter_attributes(node, ctx);
    }

    fn enter_clef(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_clef(node, ctx);
        self.b.enter_clef(node, ctx);
    }

    fn enter_staff_details(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_staff_details(node, ctx);
        self.b.enter_staff_details(node, ctx);
    }

    fn enter_backup(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_backup(node, ctx);
        self.b.enter_backup(node, ctx);
    }

    fn enter_forward(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_forward(node, ctx);
        self.b.enter_forward(node, ctx);
    }

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_note(node, ctx);
        self.b.enter_note(node, ctx);
    }

    fn exit_note(&mut self, ctx: &mut WalkerCtx) {
        self.a.exit_note(ctx);
        self.b.exit_note(ctx);
    }

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        self.a.exit_measure(ctx);
        self.b.exit_measure(ctx);
    }

    fn exit_part(&mut self, ctx: &mut WalkerCtx) {
        self.a.exit_part(ctx);
        self.b.exit_part(ctx);
    }

    fn exit(&mut self, ctx: &mut WalkerCtx) {
        self.a.exit(ctx);
        self.b.exit(ctx);
    }
}
