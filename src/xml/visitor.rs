use roxmltree::Node;
use crate::core::chain::Chain;

pub trait Visitor: Sized {
    fn enter(&mut self, node: &Node);
    fn enter_work(&mut self, node: &Node);
    fn enter_defaults(&mut self, node: &Node);
    fn exit_defaults(&mut self);
    fn enter_part_list(&mut self, node: &Node);
    fn enter_part(&mut self, node: &Node);
    fn enter_measure(&mut self, node: &Node);
    fn enter_print(&mut self, node: &Node);
    fn enter_attributes(&mut self, node: &Node);
    fn enter_note(&mut self, node: &Node);
    fn exit_measure(&mut self);
    fn exit_part(&mut self);
    fn exit(&mut self);


    fn add_callback<C: Visitor>(self, callback: C) -> Chain<Self, C> {
        Chain::new(self, callback)
    }
}

pub struct DefaultVisitor {

}

impl DefaultVisitor {

}

impl Visitor for DefaultVisitor {
    fn enter(&mut self, _node: &Node) { }

    fn enter_work(&mut self, _node: &Node) { }

    fn enter_defaults(&mut self, _node: &Node){ }

    fn exit_defaults(&mut self) { }

    fn enter_part_list(&mut self, _node: &Node) { }

    fn enter_part(&mut self, _node: &Node) { }

    fn enter_measure(&mut self, _node: &Node) { }

    fn enter_print(&mut self, _node: &Node) { }

    fn enter_attributes(&mut self, _node: &Node) { }

    fn enter_note(&mut self, _node: &Node) { }

    fn exit_measure(&mut self) { }

    fn exit_part(&mut self) { }

    fn exit(&mut self) { }
}


impl<A: Visitor, B: Visitor> Visitor for Chain<A, B> {
    fn enter(&mut self, node: &Node) {
        self.a.enter(node);
        self.b.enter(node);
    }

    fn enter_work(&mut self, node: &Node) {
        self.a.enter_work(node);
        self.b.enter_work(node);
    }

    fn enter_defaults(&mut self, node: &Node) {
        self.a.enter_defaults(node);
        self.b.enter_defaults(node);
    }

    fn exit_defaults(&mut self) {
        self.b.exit_defaults();
        self.a.exit_defaults();
    }

    fn enter_part_list(&mut self, node: &Node) {
        self.a.enter_part_list(node);
        self.b.enter_part_list(node);
    }

    fn enter_part(&mut self, node: &Node) {
        self.a.enter_part(node);
        self.b.enter_part(node);
    }

    fn enter_measure(&mut self, node: &Node) {
        self.a.enter_measure(node);
        self.b.enter_measure(node);
    }

    fn enter_print(&mut self, node: &Node) {
        self.a.enter_print(node);
        self.b.enter_print(node);
    }

    fn enter_attributes(&mut self, node: &Node) {
        self.a.enter_attributes(node);
        self.b.enter_attributes(node);
    }

    fn enter_note(&mut self, node: &Node) {
        self.a.enter_note(node);
        self.b.enter_note(node);
    }

    fn exit_measure(&mut self) {
        self.b.exit_measure();
        self.a.exit_measure();
    }

    fn exit_part(&mut self) {
        self.b.exit_part();
        self.a.exit_part();
    }

    fn exit(&mut self) {
        self.b.exit();
        self.a.exit();
    }
}