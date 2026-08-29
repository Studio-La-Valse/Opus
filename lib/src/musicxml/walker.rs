use crate::musicxml::utils::NodeUtils;
use crate::musicxml::visitor::Visitor;
use roxmltree::Document;

pub struct Walker<V> {
    pub visitor: V,
}

impl<V> Walker<V> {
    pub fn new(visitor: V) -> Self {
        Walker { visitor }
    }

    pub fn walk<C>(&mut self, document: &Document, ctx: &mut C)
    where
        V: Visitor<C>,
    {
        let root = document.root_element();

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

                                        for staff_details in child.get_children("staff-details") {
                                            self.visitor.enter_staff_details(&staff_details, ctx)
                                        }

                                        // the key requires all clefs to be visited first, even though
                                        // they may appear after the key changes in the musicxml.
                                        for clef in child.get_children("clef") {
                                            self.visitor.enter_clef(&clef, ctx);
                                        }

                                        if let Some(key) = child.get_child("key") {
                                            self.visitor.enter_key(&key, ctx)
                                        }
                                    }
                                    "note" => {
                                        self.visitor.enter_note(&child, ctx);
                                        self.visitor.exit_note(ctx);
                                    }
                                    "forward" => self.visitor.enter_forward(&child, ctx),
                                    "backup" => self.visitor.enter_backup(&child, ctx),
                                    // unrecognized measure children are ignored
                                    _ => {}
                                }
                            }

                            self.visitor.exit_measure(ctx);
                        }
                    }

                    self.visitor.exit_part(ctx);
                }
                // unrecognized top-level elements are ignored
                _ => {}
            }
        }

        self.visitor.exit(ctx);
    }
}
