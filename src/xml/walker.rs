use crate::xml::visitor::Visitor;
use roxmltree::{Document, Node};

pub struct Walker<'a> {
    pub visitor: Visitor<'a>,
}

impl<'a> Walker<'a> {
    pub fn new(visitor: Visitor<'a>) -> Self {        
        Walker { visitor }
    }

    pub fn walk(&mut self, document: &Document) {
        let root = document.root_element();
        if root.tag_name().name() != "score-partwise" {
            panic!("Expected a score-partwise root node");
        }

        self.visitor.enter(&root);

        for child in root.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "work" => self.visitor.enter_work(&child),
                "defaults" => {
                    self.visitor.enter_defaults(&child);
                    self.visitor.exit_defaults();
                }
                "part-list" => self.visitor.enter_part_list(&child),
                "part" => self.walk_part(&child),
                _ => {} // todo: ignore for now, panic! later.
            }
        }

        self.visitor.exit();
    }

    pub fn walk_part(&mut self, node: &Node) {
        self.visitor.enter_part(node);

        for child in node.children().filter(|n| n.is_element()) {
            if child.tag_name().name() == "measure" {
                self.walk_measure(&child)
            }
        }
    }

    pub fn walk_measure(&mut self, node: &Node) {
        self.visitor.enter_measure(node);

        for child in node.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "print" => self.visitor.enter_print(&child),
                "attributes" => self.visitor.enter_attributes(&child),
                "note" => self.visitor.enter_note(&child),
                _ => {} // todo: ignore for now, panic! later.
            }
        }

        self.visitor.exit_measure();
    }
}
