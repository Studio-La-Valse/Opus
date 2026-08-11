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
}
