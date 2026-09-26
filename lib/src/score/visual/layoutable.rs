use crate::geometry::color::Color;
use crate::score::core::group_symbol::{GroupLevel, GroupSymbol};
use crate::score::core::note_kind::NoteKind;
use crate::score::layout_options::{APP_DEFAULTS, UserLayout};
use crate::score::score_defaults::ScoreDefaults;
use crate::smufl::smufl_font::SmuflFont;

/// What every element resolves its own appearance and size from, threaded
/// through each element's `measure` pass.
///
/// Two of these are layout config: the caller's overrides and the document's
/// declared defaults, in falling precedence, with
/// [`APP_DEFAULTS`] as the hard-coded fallback beneath both. The third, `font`,
/// is mainly a resource -- the glyph metrics an element needs to size itself --
/// but also carries a tier of its own between the document and the app: the
/// line thicknesses its `engravingDefaults` recommend, in
/// [`SmuflFont::layout`].
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
    pub font: &'a SmuflFont,
}

impl LayoutParams<'_> {
    /// The ink everything on the page is drawn in.
    pub fn foreground_color(&self) -> Color {
        self.user_layout
            .foreground
            .color
            .unwrap_or(APP_DEFAULTS.foreground.color)
    }

    /// The fraction of full size a note of this kind is drawn at.
    ///
    /// Lives here rather than on [`NoteKind`] because it is the one place the
    /// precedence is decided, and everything a reduced note owns has to agree on
    /// it: the notehead, its dots, the stem, the flag, and the beams the
    /// enclosing [`PartMeasure`](crate::score::visual::part_measure::PartMeasure)
    /// draws. Resolving it separately in each of those is how a grace note ends
    /// up with beams reduced by a different number than its noteheads.
    pub fn note_size(&self, kind: NoteKind) -> f32 {
        let note_size = &self.user_layout.note_size;
        match kind {
            NoteKind::Normal => 1.,
            NoteKind::Grace => note_size
                .grace
                .or(self.score_defaults.appearance.grace)
                .unwrap_or(APP_DEFAULTS.note_size.grace),
            NoteKind::Cue => note_size
                .cue
                .or(self.score_defaults.appearance.cue)
                .unwrap_or(APP_DEFAULTS.note_size.cue),
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
        let layout = self.user_layout;
        let (user, app_default) = match level {
            GroupLevel::Section => (layout.section.symbol, APP_DEFAULTS.section.symbol),
            GroupLevel::PartGroup => (layout.part_group.symbol, APP_DEFAULTS.part_group.symbol),
            GroupLevel::Part => (layout.part.symbol, APP_DEFAULTS.part.symbol),
        };

        user.or(declared).unwrap_or(app_default)
    }

    /// How far out `level`'s symbol sits, in tenths, from whatever is already
    /// to the right of it: the system's left edge for a section, and the left
    /// edge of the enclosing level's symbol for the other two. Per level, not
    /// per shape: see [`GroupLevel`].
    pub fn group_symbol_gap(&self, level: GroupLevel) -> f32 {
        let layout = self.user_layout;
        match level {
            GroupLevel::Section => layout
                .section
                .symbol_gap
                .unwrap_or(APP_DEFAULTS.section.symbol_gap),
            GroupLevel::PartGroup => layout
                .part_group
                .symbol_gap
                .unwrap_or(APP_DEFAULTS.part_group.symbol_gap),
            GroupLevel::Part => layout
                .part
                .symbol_gap
                .unwrap_or(APP_DEFAULTS.part.symbol_gap),
        }
    }

    /// How thick `symbol`'s vertical stroke is drawn, in tenths. Per shape, not
    /// per level: it describes the drawing rather than the placement. Zero for
    /// the shapes that have no stroke of their own.
    pub fn group_symbol_thickness(&self, symbol: GroupSymbol) -> f32 {
        let layout = self.user_layout;
        let font = &self.font.layout;
        match symbol {
            GroupSymbol::Bracket => layout
                .group_bracket
                .thickness
                .or(font.group_bracket.thickness)
                .unwrap_or(APP_DEFAULTS.group_bracket.thickness),
            GroupSymbol::Line => layout
                .group_line
                .thickness
                .or(font.group_line.thickness)
                .unwrap_or(APP_DEFAULTS.group_line.thickness),
            GroupSymbol::Square => layout
                .group_square
                .thickness
                .unwrap_or(APP_DEFAULTS.group_square.thickness),
            // A brace's weight is the glyph's own, and nothing is drawn for
            // `None` to have a weight.
            GroupSymbol::Brace | GroupSymbol::None => 0.,
        }
    }

    /// How far a `square` symbol's arms reach toward the system, in tenths.
    pub fn group_square_arm(&self) -> f32 {
        self.user_layout
            .group_square
            .arm
            .unwrap_or(APP_DEFAULTS.group_square.arm)
    }

    /// Font size in tenths for a part / part-group name. Two-tier
    /// (user -> app), like [`group_square_arm`](Self::group_square_arm).
    pub fn group_name_size(&self) -> f32 {
        self.user_layout
            .group_name
            .size
            .unwrap_or(APP_DEFAULTS.group_name.size)
    }

    /// Padding in tenths between a part / part-group name's right edge and the
    /// left edge of the symbol it sits beside.
    pub fn group_name_padding(&self) -> f32 {
        self.user_layout
            .group_name
            .padding
            .unwrap_or(APP_DEFAULTS.group_name.padding)
    }
}
