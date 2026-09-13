use crate::geometry::color::Color;
use crate::score::core::group_symbol::GroupSymbol;
use crate::score::page_orientation::PageOrientation;

pub struct AppDefaults {
    pub page_color: Color,
    pub foreground_color: Color,
    pub page_orientation: PageOrientation,
    pub horizontal_gutter_even: f32,
    pub horizontal_gutter_uneven: f32,
    pub vertical_gutter: f32,
    pub staff: f32,
    pub stem_thickness: f32,
    pub beam_thickness: f32,
    pub beam_spacing: f32,
    pub light_barline: f32,
    pub heavy_barline: f32,

    pub note_size_grace: f32,
    pub note_size_cue: f32,

    /// Augmentation-dot radius, in tenths.
    pub dot_radius: f32,
    /// Augmentation-dot spacing, in tenths: both the gap from the notehead /
    /// rest right edge to the first dot's centre and the centre-to-centre step
    /// between successive dots.
    pub dot_spacing: f32,

    /// Tie thickness at a notehead end, in tenths. Bravura's
    /// `tieEndpointThickness` is 0.1 staff spaces; one space is
    /// [`Staff::DEFAULT_SPACE_SIZE`](crate::score::visual::staff::Staff) = 10
    /// tenths. Quoted rather than read, because `SmuflMetadata` does not
    /// deserialize `engravingDefaults` -- the same reason `beam_thickness` is
    /// hard-coded to Bravura's 0.5 spaces.
    pub tie_endpoint_thickness: f32,
    /// Tie thickness at its widest point, in tenths. Bravura's
    /// `tieMidpointThickness`, 0.22 staff spaces.
    pub tie_midpoint_thickness: f32,
    /// A tie's arc height as a fraction of its horizontal span, before clamping.
    pub tie_height_ratio: f32,
    /// Lower clamp on a tie's arc height, in tenths -- so a very short tie still
    /// reads as a curve rather than a smear.
    pub tie_height_min: f32,
    /// Upper clamp on a tie's arc height, in tenths.
    pub tie_height_max: f32,
    /// Gap between a notehead's edge and the tie tip that meets it, in tenths.
    pub tie_note_gap: f32,
    /// Offset from a notehead's vertical centre to the tie tip, in tenths.
    pub tie_vertical_offset: f32,
    /// Margin, in tenths, between the barline and the far end of the opening
    /// fragment of a tie broken across a system break. That fragment runs out to
    /// the end of its own measure, less this.
    pub tie_break_inset: f32,
    /// Length of the closing fragment of a broken tie -- the short "courtesy"
    /// arc that arrives at the note on the next system, in tenths. Also the
    /// minimum length of the opening fragment, for a note sitting right at the
    /// end of its measure.
    pub tie_break_fragment: f32,

    /// What binds a section's part-groups together when the document's
    /// `<part-group>` names no `<group-symbol>`. A section is the outermost of
    /// the three, so it gets the outermost symbol.
    pub section_symbol: GroupSymbol,
    /// What binds a part-group's parts together, absent a `<group-symbol>`.
    pub part_group_symbol: GroupSymbol,
    /// What binds one part's own staves together. MusicXML declares this in
    /// `<attributes><part-symbol>`, which nothing reads yet, so today this is
    /// always what a multi-staff part gets.
    pub part_symbol: GroupSymbol,

    /// Distance in tenths from the system's left edge to the right edge of a
    /// section symbol's vertical stroke.
    ///
    /// A section's symbol is the innermost of the three, so this one is
    /// measured from the system itself. The other two are measured from
    /// whatever is already there.
    ///
    /// A bracket's tips flare rightward past its own stroke -- that is how
    /// SMuFL draws them -- so its ink reaches nearer the staff than the gap
    /// suggests, and may cross it.
    pub section_symbol_gap: f32,
    /// Padding in tenths between the left edge of what a section drew and the
    /// right edge of a part-group's symbol inside it.
    ///
    /// Padding rather than a distance from the system, because a fixed ladder
    /// of distances cannot hold: a brace's width follows the span it covers, so
    /// a tall part-group's symbol is far wider than a short one's and would
    /// eventually reach under the section symbol outside it.
    pub part_group_symbol_gap: f32,
    /// Padding in tenths between the left edge of what a part-group drew and
    /// the right edge of the symbol joining one part's own staves. As
    /// [`Self::part_group_symbol_gap`], measured from what is already there.
    pub part_symbol_gap: f32,

    /// Thickness in tenths of a bracket's vertical stroke. Bravura's
    /// `engravingDefaults.bracketThickness` is 0.5 staff spaces, and one space
    /// is [`Staff::DEFAULT_SPACE_SIZE`](crate::score::visual::staff::Staff) = 10
    /// tenths. Quoted rather than read, for the reason the `tie_*` values are.
    ///
    /// The bracket's tip glyphs are scaled to match, so a thicker stroke keeps
    /// serifs in proportion to it.
    pub group_bracket_thickness: f32,
    /// Thickness in tenths of a `line` symbol. Bravura's `subBracketThickness`,
    /// 0.16 staff spaces -- the weight SMuFL documents for the vertical line
    /// grouping staves of one instrument, which is what `line` is for.
    pub group_line_thickness: f32,
    /// Thickness in tenths of a `square` symbol's stroke, spine and arms alike.
    ///
    /// SMuFL defines no glyph or figure of its own for a square, so unlike the
    /// bracket's and the line's this weight is quoted from nothing: it is tuned
    /// by eye, and deliberately lighter than a bracket's.
    pub group_square_thickness: f32,
    /// How far in tenths a `square` symbol's arms reach toward the system,
    /// measured from the right edge of its spine. Tuned by eye, as
    /// [`Self::group_square_thickness`] is.
    pub group_square_arm: f32,

    /// Default font family for titles / work-level text. A generic CSS family so
    /// both a browser and a system font database can resolve it.
    pub title_font: String,
    /// Default font family for lyrics.
    pub lyric_font: String,
    /// Default font family for part / part-group names.
    pub group_name_font: String,

    /// Font size in tenths for a part / part-group name. About 1.6 staff spaces,
    /// a staff space being 10 tenths.
    pub group_name_size: f32,
    /// Padding in tenths between a name's right edge and the symbol it sits
    /// beside. One staff space.
    pub group_name_padding: f32,
}

impl Default for AppDefaults {
    fn default() -> Self {
        AppDefaults {
            page_color: Color::WHITE,
            foreground_color: Color::BLACK,
            page_orientation: PageOrientation::Horizontal,
            horizontal_gutter_even: 200.,
            horizontal_gutter_uneven: 200.,
            vertical_gutter: 200.,
            staff: 1.1,
            stem_thickness: 1.,
            beam_thickness: 5.,
            beam_spacing: 1.5,
            light_barline: 1.875,
            heavy_barline: 5.,
            note_size_grace: 0.66,
            note_size_cue: 0.66,
            dot_radius: 2.,
            dot_spacing: 5.,
            tie_endpoint_thickness: 1.,
            tie_midpoint_thickness: 2.2,
            tie_height_ratio: 0.15,
            tie_height_min: 5.,
            tie_height_max: 16.,
            tie_note_gap: 2.,
            tie_vertical_offset: 5.,
            tie_break_inset: 10.,
            tie_break_fragment: 20.,

            section_symbol: GroupSymbol::Bracket,
            part_group_symbol: GroupSymbol::Brace,
            part_symbol: GroupSymbol::Brace,
            section_symbol_gap: 5.,
            part_group_symbol_gap: 2.,
            part_symbol_gap: 5.,
            group_bracket_thickness: 5.,
            group_line_thickness: 1.6,
            group_square_thickness: 1.2,
            group_square_arm: 8.,

            title_font: "serif".to_string(),
            lyric_font: "serif".to_string(),
            group_name_font: "serif".to_string(),
            group_name_size: 16.,
            group_name_padding: 10.,
        }
    }
}
