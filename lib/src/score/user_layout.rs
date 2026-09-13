use crate::geometry::color::Color;
use crate::score::core::group_symbol::GroupSymbol;
use crate::score::page_orientation::PageOrientation;
use serde::Deserialize;

/// Caller-supplied overrides, each falling back to
/// [`AppDefaults`](crate::score::app_defaults::AppDefaults) when `None`.
///
/// This struct is the single source of truth for the layout option set: it
/// deserializes directly (camelCase, every field optional), so a front end --
/// the wasm bindings today -- exposes a new knob by nothing more than the field
/// being added here. Don't mirror it into a parallel options struct somewhere
/// else; that only creates two lists to keep in step.
#[derive(Copy, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct UserLayout {
    pub page_color: Option<Color>,
    pub foreground_color: Option<Color>,

    pub page_orientation: Option<PageOrientation>,
    pub horizontal_gutter_even: Option<f32>,
    pub horizontal_gutter_uneven: Option<f32>,
    pub vertical_gutter: Option<f32>,

    pub staff_line_width: Option<f32>,
    pub light_barline: Option<f32>,
    pub heavy_barline: Option<f32>,

    pub beam_thickness: Option<f32>,
    pub beam_spacing: Option<f32>,
    pub stem_thickness: Option<f32>,

    /// Fraction of full size a grace or cue note is drawn at, overriding what
    /// the document declared in `<defaults><appearance><note-size>`. `0.5` is
    /// half size; `1.` draws them like any other note.
    ///
    /// Applies to everything the note owns -- notehead, dots, stem, flag and the
    /// beams joining it to its group -- because all of them resolve through
    /// [`LayoutParams::note_size`](crate::score::visual::layoutable::LayoutParams::note_size).
    pub note_size_grace: Option<f32>,
    pub note_size_cue: Option<f32>,

    pub dot_radius: Option<f32>,
    pub dot_spacing: Option<f32>,

    /// Overrides for the tie knobs of the same name on
    /// [`AppDefaults`](crate::score::app_defaults::AppDefaults), all in tenths
    /// except `tie_height_ratio`.
    pub tie_endpoint_thickness: Option<f32>,
    pub tie_midpoint_thickness: Option<f32>,
    pub tie_height_ratio: Option<f32>,
    pub tie_height_min: Option<f32>,
    pub tie_height_max: Option<f32>,
    pub tie_note_gap: Option<f32>,
    pub tie_vertical_offset: Option<f32>,
    pub tie_break_inset: Option<f32>,
    pub tie_break_fragment: Option<f32>,

    /// Force one symbol on every section / part-group / part in the score,
    /// overriding what each `<part-group>` declared. Set one to
    /// [`GroupSymbol::None`](crate::score::core::group_symbol::GroupSymbol) to
    /// suppress that level's symbols entirely.
    ///
    /// Nothing stops a part being given a `Bracket`: its tips reach past the
    /// system's left edge and would sit over the clef. That is the caller's
    /// choice, not a case to guard against.
    pub section_symbol: Option<GroupSymbol>,
    pub part_group_symbol: Option<GroupSymbol>,
    pub part_symbol: Option<GroupSymbol>,

    /// Overrides for the group-symbol knobs of the same name on
    /// [`AppDefaults`](crate::score::app_defaults::AppDefaults), all in tenths.
    /// The gaps are per level, the thicknesses per shape.
    pub section_symbol_gap: Option<f32>,
    pub part_group_symbol_gap: Option<f32>,
    pub part_symbol_gap: Option<f32>,
    pub group_bracket_thickness: Option<f32>,
    pub group_line_thickness: Option<f32>,
    pub group_square_thickness: Option<f32>,
    pub group_square_arm: Option<f32>,

    /// Overrides for the part / part-group name knobs of the same name on
    /// [`AppDefaults`](crate::score::app_defaults::AppDefaults), in tenths. The
    /// font family stays out: `UserLayout` is `Copy`, so `title_font` /
    /// `lyric_font` are excluded for the same reason.
    pub group_name_size: Option<f32>,
    pub group_name_padding: Option<f32>,
}
