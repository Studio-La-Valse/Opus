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

    // No `note_size_grace` / `note_size_cue`. `<note-size>` is resolved on the
    // document walk, whose result is cached and re-arranged for whatever layout
    // a later render asks for -- so a value read from here would be baked in at
    // the wrong moment and then never revisited. Exposing it as an override
    // means resolving note size in `measure` instead; see docs/roadmap.md.
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
