use crate::score::core::group_symbol::GroupSymbol;

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
        /// The `<group-symbol>` this level declared, `None` when it named one.
        /// Absent is not the same as `Some(GroupSymbol::None)`: the first means
        /// the document expressed no preference and a default applies, the
        /// second that it asked for nothing to be drawn.
        symbol: Option<GroupSymbol>,
        children: Vec<PartListNode>,
    },
    Group {
        index: u32,
        name: Option<String>,
        /// The `<group-symbol>` this level declared; see `Section`'s.
        symbol: Option<GroupSymbol>,
        children: Vec<PartListNode>,
    },
}
