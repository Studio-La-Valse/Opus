use crate::xml::visitor::Visitor;
use crate::xml::walker_ctx::WalkerCtx;
use roxmltree::Document;

pub struct Walker<V> {
    pub visitor: V,
}

impl<V: Visitor> Walker<V> {
    pub fn new(visitor: V) -> Self {
        Walker { visitor }
    }

    pub fn walk(&mut self, document: &Document, ctx: &mut WalkerCtx) {
        let root = document.root_element();
        if root.tag_name().name() != "score-partwise" {
            panic!("Expected a score-partwise root node");
        }

        self.visitor.enter(&root, ctx);

        for child in root.children().filter(|n| n.is_element()) {
            match child.tag_name().name() {
                "work" => self.visitor.enter_work(&child, ctx),
                "defaults" => {
                    self.visitor.enter_defaults(&child, ctx);
                    self.visitor.exit_defaults(ctx);
                }
                "part-list" => self.visitor.enter_part_list(&child, ctx),
                "part" => {
                    self.visitor.enter_part(&child, ctx);

                    for child in child.children().filter(|n| n.is_element()) {
                        if child.tag_name().name() == "measure" {
                            self.visitor.enter_measure(&child, ctx);

                            for child in child.children().filter(|n| n.is_element()) {
                                match child.tag_name().name() {
                                    "print" => self.visitor.enter_print(&child, ctx),
                                    "attributes" => {
                                        self.visitor.enter_attributes(&child, ctx);
                                        for child in child.children().filter(|n| n.is_element()) {
                                            match child.tag_name().name() {
                                                "clef" => self.visitor.enter_clef(&child, ctx),
                                                "staff-details" => {
                                                    self.visitor.enter_staff_details(&child, ctx)
                                                }
                                                _ => {}
                                            }
                                        }
                                    }
                                    "note" => {
                                        self.visitor.enter_note(&child, ctx);
                                        self.visitor.exit_note(ctx);
                                    }
                                    "forward" => self.visitor.enter_forward(&child, ctx),
                                    "backup" => self.visitor.enter_backup(&child, ctx),
                                    _ => {} // todo: ignore for now, panic! later.
                                }
                            }

                            self.visitor.exit_measure(ctx);
                        }
                    }
                }
                _ => {} // todo: ignore for now, panic! later.
            }
        }

        self.visitor.exit(ctx);
    }
}
