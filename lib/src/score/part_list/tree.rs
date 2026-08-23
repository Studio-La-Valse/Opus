/// An ordered node in a `<part-list>`'s structure. Preserves the exact
/// document order things appeared in, unlike `Layout::sections`/
/// `Layout::parts` (`BTreeMap`s keyed for O(1) render lookups) -- but it
/// carries the same section/part-group assignment those maps need, so
/// `from_tree::layout_from_part_list` can build them straight from this tree.
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
