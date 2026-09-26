use clap::Args;
use lib::geometry::color::Color;
use lib::score::core::group_symbol::GroupSymbol;
use lib::score::layout_options::{
    BarlineLayout, BeamLayout, DotLayout, ForegroundLayout, GroupBraceLayout, GroupBracketLayout,
    GroupLineLayout, GroupNameLayout, GroupSquareLayout, LyricLayout, MeasureStartLayout,
    NoteSizeLayout, PageLayout, PartGroupLayout, PartLayout, SectionLayout, StaffLayout,
    StemLayout, TieLayout, TitleLayout, UserLayout,
};
use lib::smufl::glyphs::brace::BraceStyle;

/// One flag per [`UserLayout`] option, each named after the kebab-case of the
/// option's path: `tie.height_max` is `--tie-height-max`. See
/// [`lib::score::layout_options`] for what each one means.
///
/// Converted with an exhaustive struct literal, so an option added to
/// `UserLayout` without a flag here fails to compile.
#[derive(Args, Debug)]
pub struct LayoutArgs {
    #[arg(long)]
    page_color: Option<Color>,

    #[arg(long)]
    foreground_color: Option<Color>,

    #[arg(long)]
    staff_line_width: Option<f32>,

    #[arg(long)]
    staff_ledger_line_width: Option<f32>,

    #[arg(long)]
    barline_light: Option<f32>,

    #[arg(long)]
    barline_heavy: Option<f32>,

    #[arg(long)]
    beam_thickness: Option<f32>,

    #[arg(long)]
    beam_spacing: Option<f32>,

    #[arg(long)]
    stem_thickness: Option<f32>,

    /// Fraction of full size a grace note is drawn at.
    #[arg(long)]
    note_size_grace: Option<f32>,

    /// Fraction of full size a cue note is drawn at.
    #[arg(long)]
    note_size_cue: Option<f32>,

    #[arg(long)]
    dot_radius: Option<f32>,

    #[arg(long)]
    dot_spacing: Option<f32>,

    /// Padding in tenths between a measure's left edge and the opening clef,
    /// between the clef column and the key signature, and between the key
    /// signature column and the time signature.
    #[arg(long)]
    measure_start_clef_padding: Option<f32>,

    #[arg(long)]
    measure_start_key_signature_padding: Option<f32>,

    #[arg(long)]
    measure_start_time_signature_padding: Option<f32>,

    #[arg(long)]
    tie_endpoint_thickness: Option<f32>,

    #[arg(long)]
    tie_midpoint_thickness: Option<f32>,

    #[arg(long)]
    tie_height_ratio: Option<f32>,

    #[arg(long)]
    tie_height_min: Option<f32>,

    #[arg(long)]
    tie_height_max: Option<f32>,

    #[arg(long)]
    tie_note_gap: Option<f32>,

    #[arg(long)]
    tie_vertical_offset: Option<f32>,

    #[arg(long)]
    tie_break_inset: Option<f32>,

    #[arg(long)]
    tie_break_fragment: Option<f32>,

    /// Force one symbol on every section / part-group / part in the score,
    /// overriding whatever its `<part-group>` declared: `none`, `brace`,
    /// `bracket`, `line` or `square`. A part has no symbol of its own in
    /// MusicXML, so `--part-symbol` is the only way to change what joins one
    /// part's staves.
    #[arg(long)]
    section_symbol: Option<GroupSymbol>,

    /// Tenths between the system's left edge and the right edge of that level's
    /// symbol. Per level, since the three have to nest against each other.
    #[arg(long)]
    section_symbol_gap: Option<f32>,

    #[arg(long)]
    part_group_symbol: Option<GroupSymbol>,

    #[arg(long)]
    part_group_symbol_gap: Option<f32>,

    #[arg(long)]
    part_symbol: Option<GroupSymbol>,

    #[arg(long)]
    part_symbol_gap: Option<f32>,

    /// Stroke thickness in tenths, per shape. A bracket's tip glyphs scale with
    /// its stroke, so thickening one keeps it in proportion.
    #[arg(long)]
    group_bracket_thickness: Option<f32>,

    /// Which brace glyph to draw: `default`, `small`, `large`, `larger` or
    /// `flat`. A font without that alternate draws its plain brace.
    #[arg(long)]
    group_brace_style: Option<BraceStyle>,

    #[arg(long)]
    group_line_thickness: Option<f32>,

    #[arg(long)]
    group_square_thickness: Option<f32>,

    /// How far a square symbol's arms reach toward the system, in tenths.
    #[arg(long)]
    group_square_arm: Option<f32>,

    /// Font family for part / part-group names. Defaults to the app default
    /// (`serif`). Resolved and embedded like `--title-font` for `render pdf`.
    #[arg(long)]
    group_name_font: Option<String>,

    /// Font size in tenths for a part / part-group name.
    #[arg(long)]
    group_name_size: Option<f32>,

    /// Padding in tenths between a part / part-group name's right edge and the
    /// symbol it sits beside.
    #[arg(long)]
    group_name_padding: Option<f32>,

    /// Font family for titles / work-level text. Defaults to the app default
    /// (`serif`). For `render pdf` this family is resolved against the installed
    /// system fonts and embedded.
    #[arg(long)]
    title_font: Option<String>,

    /// Font family for lyrics. Defaults to the app default (`serif`). Resolved
    /// and embedded like `--title-font` for `render pdf`.
    #[arg(long)]
    lyric_font: Option<String>,
}

impl From<LayoutArgs> for UserLayout {
    fn from(args: LayoutArgs) -> UserLayout {
        UserLayout {
            page: PageLayout {
                color: args.page_color,
            },
            foreground: ForegroundLayout {
                color: args.foreground_color,
            },
            staff: StaffLayout {
                line_width: args.staff_line_width,
                ledger_line_width: args.staff_ledger_line_width,
            },
            barline: BarlineLayout {
                light: args.barline_light,
                heavy: args.barline_heavy,
            },
            beam: BeamLayout {
                thickness: args.beam_thickness,
                spacing: args.beam_spacing,
            },
            stem: StemLayout {
                thickness: args.stem_thickness,
            },
            note_size: NoteSizeLayout {
                grace: args.note_size_grace,
                cue: args.note_size_cue,
            },
            dot: DotLayout {
                radius: args.dot_radius,
                spacing: args.dot_spacing,
            },
            measure_start: MeasureStartLayout {
                clef_padding: args.measure_start_clef_padding,
                key_signature_padding: args.measure_start_key_signature_padding,
                time_signature_padding: args.measure_start_time_signature_padding,
            },
            tie: TieLayout {
                endpoint_thickness: args.tie_endpoint_thickness,
                midpoint_thickness: args.tie_midpoint_thickness,
                height_ratio: args.tie_height_ratio,
                height_min: args.tie_height_min,
                height_max: args.tie_height_max,
                note_gap: args.tie_note_gap,
                vertical_offset: args.tie_vertical_offset,
                break_inset: args.tie_break_inset,
                break_fragment: args.tie_break_fragment,
            },
            section: SectionLayout {
                symbol: args.section_symbol,
                symbol_gap: args.section_symbol_gap,
            },
            part_group: PartGroupLayout {
                symbol: args.part_group_symbol,
                symbol_gap: args.part_group_symbol_gap,
            },
            part: PartLayout {
                symbol: args.part_symbol,
                symbol_gap: args.part_symbol_gap,
            },
            group_bracket: GroupBracketLayout {
                thickness: args.group_bracket_thickness,
            },
            group_brace: GroupBraceLayout {
                style: args.group_brace_style,
            },
            group_line: GroupLineLayout {
                thickness: args.group_line_thickness,
            },
            group_square: GroupSquareLayout {
                thickness: args.group_square_thickness,
                arm: args.group_square_arm,
            },
            group_name: GroupNameLayout {
                font: args.group_name_font,
                size: args.group_name_size,
                padding: args.group_name_padding,
            },
            title: TitleLayout {
                font: args.title_font,
            },
            lyric: LyricLayout {
                font: args.lyric_font,
            },
        }
    }
}
