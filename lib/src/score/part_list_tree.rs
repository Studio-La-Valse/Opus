use crate::score::layout::{Part, PartGroupLevel, PartListTracker, Section};
use roxmltree::Node;
use std::collections::BTreeMap;

/// An ordered node in a `<part-list>`'s structure. Preserves the exact
/// document order things appeared in, unlike `Layout::sections`/
/// `Layout::parts` (`BTreeMap`s keyed for O(1) render lookups) -- but it
/// carries the same section/part-group assignment those maps need, so
/// `layout_from_part_list` below can build them straight from this tree.
pub enum PartListNode {
    Part {
        id: String,
        name: String,
        abbr: String,
        section: u32,
        part_group: u32,
    },
    Section {
        index: u32,
        name: Option<String>,
        brace: Option<String>,
        children: Vec<PartListNode>,
    },
    Group {
        index: u32,
        name: Option<String>,
        brace: Option<String>,
        children: Vec<PartListNode>,
    },
}

/// What a `<part-group>` element's `type` attribute means to `build()`,
/// decided by the caller so it can decide separately how to read (or panic
/// on) that attribute.
pub enum PartGroupAction {
    Start,
    Stop,
    Other,
}

struct OpenScope {
    index: u32,
    name: Option<String>,
    brace: Option<String>,
    children: Vec<PartListNode>,
}

/// Builds a `Vec<PartListNode>` from a `<part-list>`'s `<part-group>`
/// start/stop and `<score-part>` elements, in document order. Wraps
/// `PartListTracker` for the section/part-group index assignment, so a
/// caller's tree can never disagree with what `SetupVisitor` assigns for
/// render.
#[derive(Default)]
pub struct PartListBuilder {
    tracker: PartListTracker,
    top: Vec<PartListNode>,
    open_section: Option<OpenScope>,
    open_group: Option<OpenScope>,
}

impl PartListBuilder {
    /// Walks a `<part-list>`'s children, dispatching `<part-group>` start/stop
    /// and `<score-part>` elements. `part_group_action`/`score_part_id` are
    /// the *only* two things that genuinely differ between render and
    /// validation (whether a missing/malformed attribute panics or is
    /// reported and skipped) -- everything else about the walk is identical,
    /// so it lives here once instead of in both visitors.
    ///
    /// `score_part_id` returning `None` skips that `<score-part>` (the
    /// caller is expected to have reported why, if it cares to); render's
    /// closure can simply never return in that case, since `req_attribute`
    /// panics before it would.
    pub fn build(
        &mut self,
        node: &Node,
        mut part_group_action: impl FnMut(&Node) -> PartGroupAction,
        mut score_part_id: impl FnMut(&Node) -> Option<String>,
    ) {
        for child in node.children().filter(|n| n.is_element()) {
            if child.has_tag_name("part-group") {
                match part_group_action(&child) {
                    PartGroupAction::Start => self.open_part_group_from_node(&child),
                    PartGroupAction::Stop => self.close_part_group(),
                    PartGroupAction::Other => {}
                }
            }

            if child.has_tag_name("score-part")
                && let Some(id) = score_part_id(&child)
            {
                self.push_score_part(id, &child);
            }
        }
    }

    /// Call on a `<part-group type="start">`.
    pub fn open_part_group(&mut self, name: Option<String>, brace: Option<String>) {
        match self.tracker.open_part_group() {
            Some(PartGroupLevel::Section { index }) => {
                self.open_section = Some(OpenScope {
                    index,
                    name,
                    brace,
                    children: Vec::new(),
                });
            }
            Some(PartGroupLevel::Group { index, .. }) => {
                self.open_group = Some(OpenScope {
                    index,
                    name,
                    brace,
                    children: Vec::new(),
                });
            }
            None => {}
        }
    }

    /// Convenience wrapper over `open_part_group` that reads `<group-name>`/
    /// `<group-symbol>` directly off a `<part-group type="start">` node.
    /// This extraction is identical (and equally panic-free) whether the
    /// caller is render or validation, so it lives here instead of being
    /// duplicated in both visitors.
    pub fn open_part_group_from_node(&mut self, node: &Node) {
        let name = node
            .children()
            .find(|n| n.has_tag_name("group-name"))
            .and_then(|n| n.text())
            .map(|s| s.to_string());

        let brace = node
            .children()
            .find(|n| n.has_tag_name("group-symbol"))
            .and_then(|n| n.text())
            .map(|s| s.to_string());

        self.open_part_group(name, brace);
    }

    /// Call on a `<part-group type="stop">`.
    pub fn close_part_group(&mut self) {
        self.tracker.close_part_group();

        if let Some(group) = self.open_group.take() {
            let node = PartListNode::Group {
                index: group.index,
                name: group.name,
                brace: group.brace,
                children: group.children,
            };

            match &mut self.open_section {
                Some(section) => section.children.push(node),
                None => self.top.push(node),
            }
        } else if let Some(section) = self.open_section.take() {
            self.top.push(PartListNode::Section {
                index: section.index,
                name: section.name,
                brace: section.brace,
                children: section.children,
            });
        }
    }

    /// Call on a `<score-part>`. Returns its (section, part_group) assignment.
    pub fn push_part(&mut self, id: String, name: String, abbr: String) -> (u32, u32) {
        let (section, part_group) = self.tracker.register_score_part();

        let node = PartListNode::Part {
            id,
            name,
            abbr,
            section,
            part_group,
        };

        if let Some(group) = &mut self.open_group {
            group.children.push(node);
        } else if let Some(section) = &mut self.open_section {
            section.children.push(node);
        } else {
            self.top.push(node);
        }

        (section, part_group)
    }

    /// Convenience wrapper over `push_part` that reads `<part-name>`/
    /// `<part-abbreviation>` directly off a `<score-part>` node. The `id`
    /// stays a caller-supplied parameter since how it's read (panicking in
    /// render, graceful in validation) is the one thing that has to differ.
    pub fn push_score_part(&mut self, id: String, node: &Node) -> (u32, u32) {
        let name = node
            .children()
            .find(|n| n.has_tag_name("part-name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_string();

        let abbr = node
            .children()
            .find(|n| n.has_tag_name("part-abbreviation"))
            .and_then(|n| n.text())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());

        self.push_part(id, name, abbr)
    }

    /// Finalizes the tree, auto-closing any `<part-group>` that never got a
    /// matching "stop" rather than silently dropping its contents.
    pub fn finish(mut self) -> Vec<PartListNode> {
        if self.open_group.is_some() {
            self.close_part_group();
        }
        if self.open_section.is_some() {
            self.close_part_group();
        }
        self.top
    }
}

/// Renders a `PartListNode` tree as `├──`/`└──` ASCII art, rooted at
/// `score-partwise`.
pub fn format_part_list_tree(nodes: &[PartListNode]) -> String {
    let blocks: Vec<Vec<String>> = nodes.iter().map(render_node).collect();
    let mut lines = vec!["score-partwise".to_string()];
    lines.extend(render_children(blocks));
    lines.join("\n")
}

fn render_node(node: &PartListNode) -> Vec<String> {
    match node {
        PartListNode::Part { id, name, .. } => vec![format!("{id} \"{name}\"")],
        PartListNode::Section {
            index,
            name,
            brace,
            children,
        } => render_group(
            "section",
            *index,
            name.as_deref(),
            brace.as_deref(),
            children,
        ),
        PartListNode::Group {
            index,
            name,
            brace,
            children,
        } => render_group(
            "part-group",
            *index,
            name.as_deref(),
            brace.as_deref(),
            children,
        ),
    }
}

fn render_group(
    label: &str,
    index: u32,
    name: Option<&str>,
    brace: Option<&str>,
    children: &[PartListNode],
) -> Vec<String> {
    let name = name.unwrap_or("(unnamed)");
    let brace_suffix = brace.map(|b| format!(" ({b})")).unwrap_or_default();

    let mut lines = vec![format!("{label} {index} \"{name}\"{brace_suffix}")];
    let blocks: Vec<Vec<String>> = children.iter().map(render_node).collect();
    lines.extend(render_children(blocks));
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

/// Converts an already-built part-list tree into the keyed maps render
/// actually needs (`Layout::parts`/`Layout::sections`), so `SetupVisitor`
/// can share the same tree-building step as validation instead of
/// maintaining its own separate walk of `<part-group>` nesting.
pub fn layout_from_part_list(
    nodes: &[PartListNode],
) -> (BTreeMap<String, Part>, BTreeMap<u32, Section>) {
    let mut parts = BTreeMap::new();
    let mut sections = BTreeMap::new();

    for node in nodes {
        collect_into_layout(node, None, &mut parts, &mut sections);
    }

    (parts, sections)
}

fn collect_into_layout(
    node: &PartListNode,
    current_section: Option<u32>,
    parts: &mut BTreeMap<String, Part>,
    sections: &mut BTreeMap<u32, Section>,
) {
    match node {
        PartListNode::Part {
            id,
            name,
            abbr,
            section,
            part_group,
        } => {
            parts.insert(
                id.clone(),
                Part {
                    name: name.clone(),
                    abbr: abbr.clone(),
                    section: *section,
                    part_group: *part_group,
                    brace: None,
                },
            );
        }
        PartListNode::Section {
            index,
            name,
            brace,
            children,
        } => {
            let entry = sections.entry(*index).or_default();
            entry.name = name.clone();
            entry.brace = brace.clone();

            for child in children {
                collect_into_layout(child, Some(*index), parts, sections);
            }
        }
        PartListNode::Group {
            index,
            name,
            brace,
            children,
        } => {
            if let Some(section_index) = current_section {
                let group = sections
                    .entry(section_index)
                    .or_default()
                    .groups
                    .entry(*index)
                    .or_default();
                group.name = name.clone();
                group.brace = brace.clone();
            }

            for child in children {
                collect_into_layout(child, current_section, parts, sections);
            }
        }
    }
}
