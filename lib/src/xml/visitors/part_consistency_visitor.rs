use crate::xml::validate::ValidationCtx;
use crate::xml::validation_issue::{Severity, ValidationIssue};
use crate::xml::visitor::Visitor;
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
    fn enter(&mut self, node: &Node, _ctx: &mut ValidationCtx) {
        self.root_at = node.range().start;
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        for score_part in node.descendants().filter(|n| n.has_tag_name("score-part")) {
            match score_part.attribute("id") {
                Some(id) => {
                    self.score_parts
                        .entry(id.to_string())
                        .or_insert(score_part.range().start);
                }
                None => ctx.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: "<score-part> is missing required 'id' attribute".to_string(),
                    at: score_part.range().start,
                }),
            }
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

    fn exit_part(&mut self, _ctx: &mut ValidationCtx) {
        self.current_part = None;
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

        for (id, &at) in &self.parts {
            if !self.score_parts.contains_key(id) {
                ctx.issues.push(ValidationIssue {
                    severity: Severity::Error,
                    message: format!(
                        "<part id=\"{id}\"> has no matching <score-part> in <part-list>"
                    ),
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
