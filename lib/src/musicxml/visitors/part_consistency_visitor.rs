use crate::musicxml::validate::ValidationCtx;
use crate::musicxml::validation_issue::{Severity, ValidationIssue};
use crate::musicxml::visitor::Visitor;
use roxmltree::Node;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PartConsistencyVisitor {
    root_at: usize,
    score_parts: BTreeMap<String, usize>,
    parts: BTreeMap<String, usize>,
    measure_counts: BTreeMap<String, u32>,
    current_part: Option<String>,
}

impl Visitor<ValidationCtx> for PartConsistencyVisitor {
    fn enter(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        self.root_at = node.range().start;

        if node.tag_name().name() != "score-partwise" {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: format!(
                    "Expected root element <score-partwise>, found <{}>",
                    node.tag_name().name()
                ),
                at: node.range().start,
            });
        }
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        let at = node.range().start;

        // Validation walks the <part-list> itself rather than borrowing the
        // tree the builder produces: it cares about things that tree has no
        // room for -- notably whether every <part-group> is closed, which the
        // builder silently repairs. The two walks agree on one thing only,
        // that a part-list's children are flat: a <part-group> and a
        // <score-part> are siblings, and nesting is expressed purely by
        // start/stop order.
        let mut open_groups: u32 = 0;

        for child in node.children().filter(|n| n.is_element()) {
            if child.has_tag_name("part-group") {
                match child.attribute("type") {
                    Some("start") => open_groups += 1,
                    Some("stop") if open_groups == 0 => ctx.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: "<part-group type=\"stop\"> has no matching start".to_string(),
                        at: child.range().start,
                    }),
                    Some("stop") => open_groups -= 1,
                    Some(_) => {}
                    None => ctx.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: "<part-group> is missing required 'type' attribute".to_string(),
                        at: child.range().start,
                    }),
                }
            }

            if child.has_tag_name("score-part") {
                match child.attribute("id") {
                    Some(id) => {
                        self.score_parts
                            .entry(id.to_string())
                            .or_insert(child.range().start);
                    }
                    None => ctx.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: "<score-part> is missing required 'id' attribute".to_string(),
                        at: child.range().start,
                    }),
                }
            }
        }

        if open_groups > 0 {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Warning,
                message: format!("<part-list> leaves {open_groups} <part-group>(s) unclosed"),
                at,
            });
        }
    }

    fn enter_part(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        match node.attribute("id") {
            Some(id) => {
                self.parts
                    .entry(id.to_string())
                    .or_insert(node.range().start);
                self.measure_counts.entry(id.to_string()).or_insert(0);
                self.current_part = Some(id.to_string());

                if !self.score_parts.contains_key(id) {
                    ctx.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: format!(
                            "<part id=\"{id}\"> has no matching <score-part> in <part-list>"
                        ),
                        at: node.range().start,
                    });
                }
            }
            None => {
                ctx.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: "<part> is missing required 'id' attribute".to_string(),
                    at: node.range().start,
                });
                self.current_part = None;
            }
        }
    }

    fn enter_measure(&mut self, _node: &Node, _ctx: &mut ValidationCtx) {
        if let Some(id) = &self.current_part {
            *self.measure_counts.entry(id.clone()).or_insert(0) += 1;
        }
    }

    fn exit(&mut self, ctx: &mut ValidationCtx) {
        if self.parts.is_empty() {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "score has no <part> elements".to_string(),
                at: self.root_at,
            });
        } else if self.measure_counts.values().all(|&count| count == 0) {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Error,
                message: "no part in the score has any measures".to_string(),
                at: self.root_at,
            });
        }

        for (id, &at) in &self.score_parts {
            if !self.parts.contains_key(id) {
                ctx.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!("<score-part id=\"{id}\"> has no matching <part id=\"{id}\">"),
                    at,
                });
            }
        }

        if let Some(&baseline_count) = self.measure_counts.values().next() {
            for (id, &count) in &self.measure_counts {
                if count != baseline_count {
                    let at = self.parts.get(id).copied().unwrap_or(0);
                    ctx.issues.push(ValidationIssue {
                        severity: Severity::Warning,
                        message: format!(
                            "part '{id}' has {count} measures, other parts have {baseline_count}"
                        ),
                        at,
                    });
                }
            }
        }
    }
}
