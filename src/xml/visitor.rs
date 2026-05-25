use crate::score::visual::visual_score::VisualScore;
use crate::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use crate::xml::visitors::layout_visitor::LayoutVisitor;
use crate::{LayoutCtx, UserLayout};

use crate::core::chain::Chain;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Node;

// pub struct Visitor<'a> {
//     user_layout: &'a mut UserLayout,
//     layout_ctx: &'a mut LayoutCtx,
//     visual_score: &'a mut VisualScore,
// }
//
// impl<'a> Visitor<'a> {
//     pub fn new(
//         user_layout: &'a mut UserLayout,
//         layout_ctx: &'a mut LayoutCtx,
//         visual_score: &'a mut VisualScore<'a>,
//     ) -> Self {
//         Self {
//             user_layout,
//             layout_ctx,
//             visual_score,
//         }
//     }
//
//     pub fn enter(&mut self, node: &Node) {
//         LayoutContextVisitor {}.enter(node, self.layout_ctx);
//     }
//
//     pub fn enter_work(&mut self, node: &Node) {
//         LayoutVisitor {}.enter_work(node, self.visual_score.layout);
//     }
//
//     pub fn enter_defaults(&mut self, element: &Node) {
//         LayoutVisitor {}.enter_defaults(element, self.visual_score.layout);
//     }
//
//     pub fn exit_defaults(&mut self) {
//         LayoutVisitor {}.exit_defaults(self.visual_score.layout, self.user_layout);
//     }
//
//     pub fn enter_part_list(&mut self, element: &Node) {
//         LayoutVisitor {}.enter_part_list(element, self.visual_score.layout);
//     }
//
//     pub fn enter_part(&mut self, _node: &Node) {
//         LayoutContextVisitor {}.enter_part(_node, self.layout_ctx);
//     }
//
//     pub fn enter_measure(&mut self, _node: &Node) {
//         LayoutContextVisitor {}.enter_measure(_node, self.layout_ctx);
//     }
//
//     pub fn enter_print(&mut self, _node: &Node) {
//         LayoutContextVisitor {}.enter_print(_node, self.layout_ctx);
//     }
//
//     pub fn enter_attributes(&mut self, _node: &Node) {
//         LayoutContextVisitor {}.enter_attributes(_node, self.layout_ctx);
//     }
//
//     pub fn enter_note(&mut self, _node: &Node) {
//         LayoutContextVisitor {}.enter_note(_node, self.layout_ctx);
//     }
//
//     pub fn exit_measure(&mut self) {}
//
//     pub fn exit_part(&mut self) {}
//
//     pub fn exit(&mut self) {}
// }
//
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
    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx);
    fn exit_measure(&mut self, ctx: &mut WalkerCtx);
    fn exit_part(&mut self, ctx: &mut WalkerCtx);
    fn exit(&mut self, ctx: &mut WalkerCtx);

    fn add_callback<C: Visitor>(self, callback: C) -> Chain<Self, C> {
        Chain::new(self, callback)
    }
}

pub struct DefaultVisitor {}

impl DefaultVisitor {}

impl Visitor for DefaultVisitor {
    fn enter(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_work(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_defaults(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn exit_defaults(&mut self, ctx: &mut WalkerCtx) {}

    fn enter_part_list(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_part(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_measure(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_print(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_attributes(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn enter_note(&mut self, _node: &Node, ctx: &mut WalkerCtx) {}

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {}

    fn exit_part(&mut self, ctx: &mut WalkerCtx) {}

    fn exit(&mut self, ctx: &mut WalkerCtx) {}
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
        self.b.exit_defaults(ctx);
        self.a.exit_defaults(ctx);
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

    fn enter_note(&mut self, node: &Node, ctx: &mut WalkerCtx) {
        self.a.enter_note(node, ctx);
        self.b.enter_note(node, ctx);
    }

    fn exit_measure(&mut self, ctx: &mut WalkerCtx) {
        self.b.exit_measure(ctx);
        self.a.exit_measure(ctx);
    }

    fn exit_part(&mut self, ctx: &mut WalkerCtx) {
        self.b.exit_part(ctx);
        self.a.exit_part(ctx);
    }

    fn exit(&mut self, ctx: &mut WalkerCtx) {
        self.b.exit(ctx);
        self.a.exit(ctx);
    }
}
