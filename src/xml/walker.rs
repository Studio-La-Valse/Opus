use crate::xml::visitor::Visitor;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::{Document, Node};

pub struct Walker<'a, V> {
    pub visitor: V,
    pub ctx: &'a mut WalkerCtx<'a>,
}

impl<'a, V: Visitor> Walker<'a, V> {
    pub fn new(visitor: V, ctx: &'a mut WalkerCtx<'a>) -> Self {
        Walker { visitor, ctx }
    }

    pub fn walk(&mut self, document: &Document) {
        let root = document.root_element();
        if root.tag_name().name() != "score-partwise" {
            panic!("Expected a score-partwise root node");
        }

        self.visitor.enter(&root, self.ctx);

        for child in root.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "work" => self.visitor.enter_work(&child, self.ctx),
                "defaults" => {
                    self.visitor.enter_defaults(&child, self.ctx);
                    self.visitor.exit_defaults(self.ctx);
                }
                "part-list" => self.visitor.enter_part_list(&child, self.ctx),
                "part" => self.walk_part(&child),
                _ => {} // todo: ignore for now, panic! later.
            }
        }

        self.visitor.exit(self.ctx);
    }

    pub fn walk_part(&mut self, node: &Node) {
        self.visitor.enter_part(node, self.ctx);

        for child in node.children().filter(|n| n.is_element()) {
            if child.tag_name().name() == "measure" {
                self.walk_measure(&child)
            }
        }
    }

    pub fn walk_measure(&mut self, node: &Node) {
        self.visitor.enter_measure(node, self.ctx);

        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "print" => self.visitor.enter_print(&child, self.ctx),
                "attributes" => self.visitor.enter_attributes(&child, self.ctx),
                "note" => self.visitor.enter_note(&child, self.ctx),
                _ => {} // todo: ignore for now, panic! later.
            }
        }

        self.visitor.exit_measure(self.ctx);
    }
}
