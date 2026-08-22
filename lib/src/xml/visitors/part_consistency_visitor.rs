use crate::score::layout::{PartGroupLevel, PartListTracker};
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

/// A section or part-group's accumulated tree lines, kept open on a stack
/// while its `<part-group>` hasn't hit its matching "stop" yet, so children
/// are appended in true document order instead of being reconstructed
/// afterwards from a sorted map.
struct OpenGroup {
    index: u32,
    name: Option<String>,
    brace: Option<String>,
    children: Vec<Vec<String>>,
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
        } else {
            ctx.issues.push(ValidationIssue {
                severity: Severity::Info,
                message: "Now entering score-partwise".to_string(),
                at: self.root_at,
            });
        }
    }

    fn enter_part_list(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        let at = node.range().start;
        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message: "Collecting parts...".to_string(),
            at,
        });

        // Same nesting state machine SetupVisitor uses to populate
        // Layout::sections, so these counts can never disagree with what
        // render actually assigns -- here we only need the counts, not the
        // brace/name detail render also collects.
        let mut tracker = PartListTracker::default();

        // Tree lines are built up in true document order as we go: a
        // section/part-group pushes a new open scope on "start" and pops it
        // (wrapping its accumulated children into one block) on "stop".
        let mut top_blocks: Vec<Vec<String>> = Vec::new();
        let mut open_section: Option<OpenGroup> = None;
        let mut open_group: Option<OpenGroup> = None;

        for child in node.children().filter(|n| n.is_element()) {
            if child.has_tag_name("part-group") {
                match child.attribute("type") {
                    Some("start") => match tracker.open_part_group() {
                        Some(PartGroupLevel::Section { index }) => {
                            open_section = Some(OpenGroup {
                                index,
                                name: group_name(&child),
                                brace: group_symbol(&child),
                                children: Vec::new(),
                            });
                        }
                        Some(PartGroupLevel::Group { index, .. }) => {
                            open_group = Some(OpenGroup {
                                index,
                                name: group_name(&child),
                                brace: group_symbol(&child),
                                children: Vec::new(),
                            });
                        }
                        None => {}
                    },
                    Some("stop") => {
                        tracker.close_part_group();

                        if let Some(group) = open_group.take() {
                            let block = wrap_tree_block("part-group", group);
                            if let Some(section) = &mut open_section {
                                section.children.push(block);
                            } else {
                                top_blocks.push(block);
                            }
                        } else if let Some(section) = open_section.take() {
                            top_blocks.push(wrap_tree_block("section", section));
                        }
                    }
                    _ => {}
                }
            }

            if child.has_tag_name("score-part") {
                tracker.register_score_part();

                match child.attribute("id") {
                    Some(id) => {
                        self.score_parts
                            .entry(id.to_string())
                            .or_insert(child.range().start);

                        let name = child
                            .children()
                            .find(|n| n.has_tag_name("part-name"))
                            .and_then(|n| n.text())
                            .unwrap_or("")
                            .to_string();

                        let block = vec![format!("{id} \"{name}\"")];
                        if let Some(group) = &mut open_group {
                            group.children.push(block);
                        } else if let Some(section) = &mut open_section {
                            section.children.push(block);
                        } else {
                            top_blocks.push(block);
                        }
                    }
                    None => ctx.issues.push(ValidationIssue {
                        severity: Severity::Error,
                        message: "<score-part> is missing required 'id' attribute".to_string(),
                        at: child.range().start,
                    }),
                }
            }
        }

        let tree = {
            let mut lines = vec!["score-partwise".to_string()];
            lines.extend(render_children(top_blocks));
            lines.join("\n")
        };
        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message: tree,
            at,
        });

        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message: format!(
                "Done, found {} part(s), {} section(s), {} part-group(s).",
                self.score_parts.len(),
                tracker.sections_opened,
                tracker.part_groups_opened
            ),
            at,
        });
    }

    fn enter_part(&mut self, node: &Node, ctx: &mut ValidationCtx) {
        if self.current_part.is_none() {
            let at = node.range().start;
            ctx.issues.push(ValidationIssue {
                severity: Severity::Info,
                message: "Now traversing parts...".to_string(),
                at,
            });
        }

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
        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message: "Done, now gracefully exiting score-partwise".to_string(),
            at: self.root_at,
        });

        let total_measures: u32 = self.measure_counts.values().sum();
        ctx.issues.push(ValidationIssue {
            severity: Severity::Info,
            message: format!(
                "Found {} part(s) and {total_measures} measure(s) total",
                self.parts.len()
            ),
            at: self.root_at,
        });

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

fn group_name(node: &Node) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name("group-name"))
        .and_then(|n| n.text())
        .map(|s| s.to_string())
}

fn group_symbol(node: &Node) -> Option<String> {
    node.children()
        .find(|n| n.has_tag_name("group-symbol"))
        .and_then(|n| n.text())
        .map(|s| s.to_string())
}

/// Wraps an `OpenGroup`'s accumulated children into one tree block, labeled
/// e.g. `section 0 "Strings" (bracket)` or `part-group 1 "..."`.
fn wrap_tree_block(label: &str, group: OpenGroup) -> Vec<String> {
    let name = group.name.as_deref().unwrap_or("(unnamed)");
    let brace = group
        .brace
        .as_deref()
        .map(|b| format!(" ({b})"))
        .unwrap_or_default();

    let mut lines = vec![format!("{label} {} \"{name}\"{brace}", group.index)];
    lines.extend(render_children(group.children));
    lines
}

/// Renders a list of already-formatted (but unprefixed) blocks as tree
/// siblings, applying the connector/indent to each block's first line and
/// deeper indent to the rest.
fn render_children(children: Vec<Vec<String>>) -> Vec<String> {
    let mut out = Vec::new();
    let last_idx = children.len().saturating_sub(1);

    for (i, block) in children.into_iter().enumerate() {
        let is_last = i == last_idx;
        let connector = if is_last { "└── " } else { "├── " };
        let indent = if is_last { "    " } else { "│   " };

        let mut iter = block.into_iter();
        if let Some(first) = iter.next() {
            out.push(format!("{connector}{first}"));
        }
        out.extend(iter.map(|line| format!("{indent}{line}")));
    }

    out
}
