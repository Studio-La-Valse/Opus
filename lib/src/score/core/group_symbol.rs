use crate::musicxml::utils::NodeUtils;
use roxmltree::Node;
use serde::Deserialize;
use std::fmt;
use std::str::FromStr;

/// What binds a run of staves together at the left of a system.
///
/// MusicXML declares this in two places, with the same five values: a
/// `<part-group>`'s `<group-symbol>`, and an `<attributes><part-symbol>` for the
/// brace joining one part's own staves. Both are the same visual decision, so
/// they are the same type; where the value came from is the caller's business.
/// Only `<group-symbol>` is read today -- see [`Self::from_mxml`].
///
/// Deserializes from, and parses out of, the strings MusicXML itself uses, so a
/// `<group-symbol>`, a CLI flag, a JSON option and a CSS custom property all
/// spell it the same way -- the arrangement [`PageOrientation`] already uses.
///
/// Deliberately not [`Default`]: which symbol stands in for one the document
/// never named is a per-level decision, and it lives on `AppDefaults` where the
/// three levels can differ. A bare `GroupSymbol::default()` would have to pick
/// one of them and be wrong for the other two.
///
/// [`PageOrientation`]: crate::score::page_orientation::PageOrientation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub enum GroupSymbol {
    /// Drawn as nothing. Distinct from an *absent* `<group-symbol>`, which is
    /// `Option::None` and means "no preference" rather than "no symbol".
    None,
    Brace,
    Bracket,
    Line,
    Square,
}

impl GroupSymbol {
    /// How far *out* this symbol wants to sit. Standard engraving nests these
    /// strictly: a bracket encloses a brace, a brace encloses a bare
    /// (symbol-less) group, and never the other way around.
    ///
    /// Taken over a `<part-group>`'s *declared* symbol, so an absent one ranks
    /// innermost -- see [`Self::declared_nesting_rank`].
    pub fn nesting_rank(&self) -> u8 {
        match self {
            GroupSymbol::Bracket | GroupSymbol::Line | GroupSymbol::Square => 2,
            GroupSymbol::Brace => 1,
            GroupSymbol::None => 0,
        }
    }

    /// [`Self::nesting_rank`] for a symbol the document may not have named at
    /// all. An absent one ranks with `none`: nothing is the least enclosing
    /// thing there is.
    pub fn declared_nesting_rank(declared: Option<GroupSymbol>) -> u8 {
        declared.map(|symbol| symbol.nesting_rank()).unwrap_or(0)
    }

    /// The `<group-symbol>` of a `<part-group>`, or `None` when it names none.
    ///
    /// Panics on a value outside the five the format allows, rather than
    /// guessing at one: which symbol a group is bound by is a visible editorial
    /// decision, and inventing one silently redraws the score's structure.
    /// `GroupSymbolVisitor` reports the same cause in plain words on the
    /// validation walk that runs first.
    pub fn from_mxml(node: &Node) -> Option<GroupSymbol> {
        let symbol = node.get_child("group-symbol")?;
        let text = symbol.text().unwrap_or("").trim();

        Some(text.parse().unwrap_or_else(|err| panic!("{err}")))
    }

    /// The MusicXML spelling, which is also the CLI and option spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            GroupSymbol::None => "none",
            GroupSymbol::Brace => "brace",
            GroupSymbol::Bracket => "bracket",
            GroupSymbol::Line => "line",
            GroupSymbol::Square => "square",
        }
    }
}

#[derive(Debug)]
pub struct GroupSymbolParseError(String);

impl fmt::Display for GroupSymbolParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "invalid group symbol '{}': expected 'none', 'brace', 'bracket', 'line' or 'square'",
            self.0
        )
    }
}

impl std::error::Error for GroupSymbolParseError {}

impl FromStr for GroupSymbol {
    type Err = GroupSymbolParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.trim().to_lowercase().as_str() {
            "none" => Ok(GroupSymbol::None),
            "brace" => Ok(GroupSymbol::Brace),
            "bracket" => Ok(GroupSymbol::Bracket),
            "line" => Ok(GroupSymbol::Line),
            "square" => Ok(GroupSymbol::Square),
            _ => Err(GroupSymbolParseError(s.to_string())),
        }
    }
}

/// Backs `#[serde(try_from = "String")]` on [`GroupSymbol`].
impl TryFrom<String> for GroupSymbol {
    type Error = GroupSymbolParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.parse()
    }
}

impl fmt::Display for GroupSymbol {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}
