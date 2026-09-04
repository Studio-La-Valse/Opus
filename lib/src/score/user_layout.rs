use crate::geometry::color::Color;
use crate::score::page_orientation::PageOrientation;

#[derive(Copy, Clone, Default)]
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
