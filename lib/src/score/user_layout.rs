use crate::geometry::color::Color;
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

    pub staff: Option<f32>,
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
    pub dot_color: Option<Color>,

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
}
