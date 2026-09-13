use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::group_symbol::{GroupLevel, GroupSymbol};
use crate::score::core::note_kind::NoteKind;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::smufl::smufl_font::SmuflFont;

/// What every element resolves its own appearance and size from, threaded
/// through the [`Layoutable::measure`] pass.
///
/// Three of these are layout config, in falling precedence: the caller's
/// overrides, the document's declared defaults, and the hard-coded fallbacks.
/// The fourth, `font`, is not config but a resource -- the glyph metrics an
/// element needs to size itself.
///
/// The font is here rather than being handed to elements at construction
/// because *which* glyph an element draws is not always settled by then. A
/// group symbol resolves its shape from the user layout, which is re-resolved on
/// every `arrange_score` -- the wasm bindings walk a document once and re-lay it
/// out per render -- so an element that had picked its glyph during the walk
/// would be frozen at whatever the first render asked for.
#[derive(Clone, Copy)]
pub struct LayoutParams<'a> {
    pub score_defaults: &'a ScoreDefaults,
    pub user_layout: &'a UserLayout,
    pub app_defaults: &'a AppDefaults,
    pub font: &'a SmuflFont,

    /// Whether group and part names in this subtree draw their abbreviation
    /// rather than their full name. The one positional fact in an otherwise
    /// pure-config struct: it is `false` where the params are first built, and
    /// [`System::resolve_layout`](crate::score::visual::system::System)
    /// re-stamps it from the system's own index -- the first system of the
    /// score names in full, every later one abbreviates.
    pub abbreviate_names: bool,
}

impl LayoutParams<'_> {
    /// The fraction of full size a note of this kind is drawn at.
    ///
    /// Lives here rather than on [`NoteKind`] because it is the one place the
    /// precedence is decided, and everything a reduced note owns has to agree on
    /// it: the notehead, its dots, the stem, the flag, and the beams the
    /// enclosing [`PartMeasure`](crate::score::visual::part_measure::PartMeasure)
    /// draws. Resolving it separately in each of those is how a grace note ends
    /// up with beams reduced by a different number than its noteheads.
    pub fn note_size(&self, kind: NoteKind) -> f32 {
        match kind {
            NoteKind::Normal => 1.,
            NoteKind::Grace => self
                .user_layout
                .note_size_grace
                .or(self.score_defaults.appearance.grace)
                .unwrap_or(self.app_defaults.note_size_grace),
            NoteKind::Cue => self
                .user_layout
                .note_size_cue
                .or(self.score_defaults.appearance.cue)
                .unwrap_or(self.app_defaults.note_size_cue),
        }
    }

    /// Which symbol binds a group at `level`, given what its `<part-group>`
    /// declared.
    ///
    /// Three sources in falling precedence, and this is the one place that
    /// order is decided. A caller's override wins outright -- it is how a score
    /// is re-drawn with brackets throughout regardless of what the exporter
    /// wrote. Failing that the document is obeyed. Failing *that* the level's
    /// own convention applies, which is why an absent `<group-symbol>` and an
    /// explicit `<group-symbol>none</group-symbol>` cannot be the same value:
    /// the first falls through to the default, the second is a declaration that
    /// nothing be drawn and stops here.
    pub fn group_symbol(&self, level: GroupLevel, declared: Option<GroupSymbol>) -> GroupSymbol {
        let (user, app_default) = match level {
            GroupLevel::Section => (
                self.user_layout.section_symbol,
                self.app_defaults.section_symbol,
            ),
            GroupLevel::PartGroup => (
                self.user_layout.part_group_symbol,
                self.app_defaults.part_group_symbol,
            ),
            GroupLevel::Part => (self.user_layout.part_symbol, self.app_defaults.part_symbol),
        };

        user.or(declared).unwrap_or(app_default)
    }

    /// How far out `level`'s symbol sits, in tenths, from whatever is already
    /// to the right of it: the system's left edge for a section, and the left
    /// edge of the enclosing level's symbol for the other two. Per level, not
    /// per shape: see [`GroupLevel`].
    pub fn group_symbol_gap(&self, level: GroupLevel) -> f32 {
        match level {
            GroupLevel::Section => self
                .user_layout
                .section_symbol_gap
                .unwrap_or(self.app_defaults.section_symbol_gap),
            GroupLevel::PartGroup => self
                .user_layout
                .part_group_symbol_gap
                .unwrap_or(self.app_defaults.part_group_symbol_gap),
            GroupLevel::Part => self
                .user_layout
                .part_symbol_gap
                .unwrap_or(self.app_defaults.part_symbol_gap),
        }
    }

    /// How thick `symbol`'s vertical stroke is drawn, in tenths. Per shape, not
    /// per level: it describes the drawing rather than the placement. Zero for
    /// the shapes that have no stroke of their own.
    pub fn group_symbol_thickness(&self, symbol: GroupSymbol) -> f32 {
        match symbol {
            GroupSymbol::Bracket => self
                .user_layout
                .group_bracket_thickness
                .unwrap_or(self.app_defaults.group_bracket_thickness),
            GroupSymbol::Line => self
                .user_layout
                .group_line_thickness
                .unwrap_or(self.app_defaults.group_line_thickness),
            GroupSymbol::Square => self
                .user_layout
                .group_square_thickness
                .unwrap_or(self.app_defaults.group_square_thickness),
            // A brace's weight is the glyph's own, and nothing is drawn for
            // `None` to have a weight.
            GroupSymbol::Brace | GroupSymbol::None => 0.,
        }
    }

    /// How far a `square` symbol's arms reach toward the system, in tenths.
    pub fn group_square_arm(&self) -> f32 {
        self.user_layout
            .group_square_arm
            .unwrap_or(self.app_defaults.group_square_arm)
    }

    /// Font size in tenths for a part / part-group name. Two-tier
    /// (user -> app), like [`group_square_arm`](Self::group_square_arm).
    pub fn group_name_size(&self) -> f32 {
        self.user_layout
            .group_name_size
            .unwrap_or(self.app_defaults.group_name_size)
    }

    /// Padding in tenths between a part / part-group name's right edge and the
    /// left edge of the symbol it sits beside.
    pub fn group_name_padding(&self) -> f32 {
        self.user_layout
            .group_name_padding
            .unwrap_or(self.app_defaults.group_name_padding)
    }
}

pub trait Layoutable {
    /// Resolves this element's appearance from `params` -- colour, thickness,
    /// which glyph it draws -- and recurses into every child `measure` forwards
    /// to. Its own downward pass, ahead of `measure`, so that an element's
    /// appearance is settled from the top of the tree down before any element is
    /// sized; see [`arrange_score`](crate::score::engrave::arrange_score).
    fn resolve_layout(&mut self, params: LayoutParams<'_>);

    /// Sizes this element and its children against `available`. Appearance is
    /// already resolved by this point -- see
    /// [`resolve_layout`](Layoutable::resolve_layout) -- so this pass is sizing
    /// only.
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>);

    fn arrange(&mut self, origin: &XY);
}
