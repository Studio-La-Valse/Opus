use crate::musicxml::utils::NodeUtils;
use crate::score::core::group_symbol::GroupSymbol;
use crate::score::part_list::tree::PartListNode;
use roxmltree::Node;
use std::cmp::Reverse;

/// Builds the part-list tree a score is rendered from, out of a `<part-list>`'s
/// `<part-group>` start/stop and `<score-part>` elements in document order.
///
/// This is the render path and nothing else: attributes the format requires are
/// read with `req_attribute`, which panics on a document that omits them, and
/// `GroupSymbol::from_mxml` panics on a `<group-symbol>` naming something the
/// format does not define. That is only safe because the validation walk runs
/// first and reports every one of those causes as an error --
/// `PartConsistencyVisitor` for a `<part-group>` with no `type` and a
/// `<score-part>` with no `id`, `GroupSymbolVisitor` for the symbol -- so
/// nothing reaches here without having been described in plain words already.
pub fn build_part_list(part_list: &Node) -> Vec<PartListNode> {
    let mut builder = PartListBuilder::default();
    builder.build(part_list);
    builder.finish()
}

/// A `<part-group>` that has been opened and is still collecting children.
struct OpenScope {
    index: u32,
    name: Option<String>,
    abbr: Option<String>,
    symbol: Option<GroupSymbol>,
    children: Vec<PartListNode>,
}

/// The `<group-name>`/`<group-abbreviation>`/`<group-symbol>` a
/// `<part-group type="start">` carries, held back until the run of consecutive
/// starts it belongs to is complete.
struct GroupHeader {
    name: Option<String>,
    abbr: Option<String>,
    symbol: Option<GroupSymbol>,
}

/// The two-level state machine behind [`build_part_list`]: a section (the outer
/// level, drawn as a bracket) holding part-groups (the inner level, drawn as a
/// brace). Anything nested deeper than that is flattened, since the visual tree
/// has nowhere to put it.
///
/// `first_order_index`/`second_order_index` are the section/part-group ids each
/// `<score-part>` is stamped with, and `ScoreDefaults::lookup` reads back out of
/// the finished tree. A part at the top level consumes a section index of its
/// own, which is why an ungrouped part between two sections shifts every section
/// index after it.
#[derive(Default)]
struct PartListBuilder {
    top: Vec<PartListNode>,
    open_section: Option<OpenScope>,
    open_group: Option<OpenScope>,
    pending_starts: Vec<GroupHeader>,

    first_order_index: u32,
    second_order_index: u32,
}

impl PartListBuilder {
    /// Walks a `<part-list>`'s children, dispatching `<part-group>` start/stop
    /// and `<score-part>` elements.
    fn build(&mut self, node: &Node) {
        for child in node.children().filter(|n| n.is_element()) {
            if child.has_tag("part-group") {
                match child.req_attribute("type") {
                    "start" => self.queue_part_group(&child),
                    "stop" => self.close_part_group(),
                    _ => {}
                }
            }

            if child.has_tag("score-part") {
                let id = child.req_attribute("id").to_string();
                self.push_score_part(id, &child);
            }
        }
    }

    /// Call on a `<part-group type="start">`, reading `<group-name>`/
    /// `<group-symbol>` directly off the node.
    ///
    /// The header is only queued: nothing about the level it belongs to is
    /// decided until `flush_pending_starts` sees the whole run of starts.
    fn queue_part_group(&mut self, node: &Node) {
        let name = node
            .children()
            .find(|n| n.has_tag("group-name"))
            .and_then(|n| n.text())
            .map(|s| s.to_string());

        // Falls back to the full name when the document names no
        // `<group-abbreviation>`, the same way `push_score_part` does for a
        // `<part-abbreviation>`.
        let abbr = node
            .children()
            .find(|n| n.has_tag("group-abbreviation"))
            .and_then(|n| n.text())
            .map(|s| s.to_string())
            .or_else(|| name.clone());

        let symbol = GroupSymbol::from_mxml(node);

        self.pending_starts.push(GroupHeader { name, abbr, symbol });
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
        headers.sort_by_key(|header| Reverse(GroupSymbol::declared_nesting_rank(header.symbol)));

        for header in headers {
            self.open_part_group(header.name, header.abbr, header.symbol);
        }
    }

    /// Opens one level, ignoring the start entirely once both levels are taken.
    fn open_part_group(
        &mut self,
        name: Option<String>,
        abbr: Option<String>,
        symbol: Option<GroupSymbol>,
    ) {
        if self.open_section.is_none() {
            self.second_order_index = 0;
            self.open_section = Some(OpenScope {
                index: self.first_order_index,
                name,
                abbr,
                symbol,
                children: Vec::new(),
            });
        } else if self.open_group.is_none() {
            self.open_group = Some(OpenScope {
                index: self.second_order_index,
                name,
                abbr,
                symbol,
                children: Vec::new(),
            });
        }
    }

    /// Call on a `<part-group type="stop">`. Closes the innermost open level,
    /// after opening anything still queued so that a start/stop pair with
    /// nothing between it still produces its (empty) node.
    fn close_part_group(&mut self) {
        self.flush_pending_starts();

        if let Some(group) = self.open_group.take() {
            self.second_order_index += 1;

            let node = PartListNode::Group {
                index: group.index,
                name: group.name,
                abbr: group.abbr,
                symbol: group.symbol,
                children: group.children,
            };

            match &mut self.open_section {
                Some(section) => section.children.push(node),
                None => self.top.push(node),
            }
        } else if let Some(section) = self.open_section.take() {
            self.first_order_index += 1;
            self.second_order_index = 0;

            self.top.push(PartListNode::Section {
                index: section.index,
                name: section.name,
                abbr: section.abbr,
                symbol: section.symbol,
                children: section.children,
            });
        }
    }

    /// Call on a `<score-part>`, reading `<part-name>`/`<part-abbreviation>`
    /// off the node and filing it under whichever level is currently open.
    fn push_score_part(&mut self, id: String, node: &Node) {
        self.flush_pending_starts();

        let name = node
            .children()
            .find(|n| n.has_tag("part-name"))
            .and_then(|n| n.text())
            .unwrap_or("")
            .to_string();

        let abbr = node
            .children()
            .find(|n| n.has_tag("part-abbreviation"))
            .and_then(|n| n.text())
            .map(|s| s.to_string())
            .unwrap_or_else(|| name.clone());

        let part = PartListNode::Part {
            id,
            name,
            abbr,
            section: self.first_order_index,
            part_group: self.second_order_index,
        };

        if let Some(group) = &mut self.open_group {
            group.children.push(part);
        } else if let Some(section) = &mut self.open_section {
            section.children.push(part);
            self.second_order_index += 1;
        } else {
            self.top.push(part);
            self.first_order_index += 1;
            self.second_order_index = 0;
        }
    }

    /// Finalizes the tree, auto-closing any `<part-group>` that never got a
    /// matching "stop" rather than silently dropping its contents.
    fn finish(mut self) -> Vec<PartListNode> {
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
