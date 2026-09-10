use crate::score::core::group_symbol::GroupSymbol;
use crate::score::part_list::tree::PartListNode;

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
            symbol,
            children,
        } => render_group("section", *index, name.as_deref(), *symbol, children),
        PartListNode::Group {
            index,
            name,
            symbol,
            children,
        } => render_group("part-group", *index, name.as_deref(), *symbol, children),
    }
}

fn render_group(
    label: &str,
    index: u32,
    name: Option<&str>,
    symbol: Option<GroupSymbol>,
    children: &[PartListNode],
) -> Vec<String> {
    let name = name.unwrap_or("(unnamed)");
    let symbol_suffix = symbol.map(|s| format!(" ({s})")).unwrap_or_default();

    let mut lines = vec![format!("{label} {index} \"{name}\"{symbol_suffix}")];
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
