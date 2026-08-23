use crate::score::part_list::entries::{PartListSection, ScorePart};
use crate::score::part_list::tree::PartListNode;
use std::collections::BTreeMap;

/// Converts an already-built part-list tree into the keyed maps render
/// actually needs (`Layout::parts`/`Layout::sections`), so `SetupVisitor`
/// can share the same tree-building step as validation instead of
/// maintaining its own separate walk of `<part-group>` nesting.
pub fn layout_from_part_list(
    nodes: &[PartListNode],
) -> (BTreeMap<String, ScorePart>, BTreeMap<u32, PartListSection>) {
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
    parts: &mut BTreeMap<String, ScorePart>,
    sections: &mut BTreeMap<u32, PartListSection>,
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
                ScorePart {
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
