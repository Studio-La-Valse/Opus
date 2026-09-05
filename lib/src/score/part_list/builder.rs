use crate::score::part_list::tracker::{PartGroupLevel, PartListTracker};
use crate::score::part_list::tree::PartListNode;
use roxmltree::Node;
use std::cmp::Reverse;

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

/// The `<group-name>`/`<group-symbol>` a `<part-group type="start">` carries,
/// held back until the run of consecutive starts it belongs to is complete.
struct GroupHeader {
    name: Option<String>,
    brace: Option<String>,
}

impl GroupHeader {
    /// How far *out* this header's symbol wants to sit. Standard engraving
    /// nests these strictly: a bracket encloses a brace, a brace encloses a
    /// bare (symbol-less) group, and never the other way around.
    fn nesting_rank(&self) -> u8 {
        match self.brace.as_deref() {
            Some("bracket") | Some("line") | Some("square") => 2,
            Some("brace") => 1,
            _ => 0,
        }
    }
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
    pending_starts: Vec<GroupHeader>,
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
    fn open_part_group(&mut self, name: Option<String>, brace: Option<String>) {
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

    /// Call on a `<part-group type="start">`, reading `<group-name>`/
    /// `<group-symbol>` directly off the node. This extraction is identical
    /// (and equally panic-free) whether the caller is render or validation,
    /// so it lives here instead of being duplicated in both visitors.
    ///
    /// The header is only queued: nothing about the level it belongs to is
    /// decided until `flush_pending_starts` sees the whole run of starts.
    fn open_part_group_from_node(&mut self, node: &Node) {
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

        self.pending_starts.push(GroupHeader { name, brace });
    }

    /// Opens every `<part-group type="start">` seen since the last
    /// `<score-part>` or "stop", outermost first.
    ///
    /// Within such a run the document gives no positional evidence of which
    /// group encloses which -- the starts are simply adjacent -- and exporters
    /// do emit them in the wrong order (ActorPreludeSample writes the braced
    /// "Horns in F" horn pair *before* the bracket that spans all of the
    /// brass, and only its `number` attributes, which are themselves
    /// unreliable there, say otherwise). So instead of trusting document order
    /// or the numbering, the run is ordered by `nesting_rank`, which is the
    /// engraving convention the visual tree already assumes: a section draws a
    /// bracket, a part-group inside it draws a brace. A stable sort leaves
    /// equally-ranked starts in document order.
    fn flush_pending_starts(&mut self) {
        if self.pending_starts.is_empty() {
            return;
        }

        let mut headers = std::mem::take(&mut self.pending_starts);
        headers.sort_by_key(|header| Reverse(header.nesting_rank()));

        for header in headers {
            self.open_part_group(header.name, header.brace);
        }
    }

    /// Call on a `<part-group type="stop">`.
    fn close_part_group(&mut self) {
        self.flush_pending_starts();
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
    fn push_part(&mut self, id: String, name: String, abbr: String) -> (u32, u32) {
        self.flush_pending_starts();

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
    fn push_score_part(&mut self, id: String, node: &Node) -> (u32, u32) {
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
        self.flush_pending_starts();

        if self.open_group.is_some() {
            self.close_part_group();
        }
        if self.open_section.is_some() {
            self.close_part_group();
        }
        self.top
    }
}
