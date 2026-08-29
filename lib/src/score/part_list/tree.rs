/// An ordered node in a `<part-list>`'s structure, preserving the exact
/// document order things appeared in. Stored directly as `ScoreDefaults::part_list`;
/// `ScoreDefaults::lookup` searches it for a part's section/part-group assignment.
#[derive(Clone)]
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
