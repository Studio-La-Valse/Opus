use std::collections::BTreeMap;

/// A single `<score-part>`'s metadata, keyed by id in `Layout::parts`.
#[derive(Default, Clone)]
pub struct ScorePart {
    pub name: String,
    pub abbr: String,

    pub section: u32,
    pub part_group: u32,

    pub brace: Option<String>,
}

/// The first-order `<part-group>` nesting level around one or more parts,
/// keyed by index in `Layout::sections`.
#[derive(Default, Clone)]
pub struct PartListSection {
    pub name: Option<String>,
    pub brace: Option<String>,
    pub groups: BTreeMap<u32, PartListGroup>,
}

/// The second-order `<part-group>` nesting level, keyed by index within a
/// `PartListSection`.
#[derive(Default, Clone)]
pub struct PartListGroup {
    pub brace: Option<String>,
    pub name: Option<String>,
}
