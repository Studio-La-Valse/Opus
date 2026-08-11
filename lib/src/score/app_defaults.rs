use crate::geometry::color::Color;
use crate::score::page_orientation::PageOrientation;

pub struct AppDefaults {
    pub page_color: Color,
    pub foreground_color: Color,
    pub page_orientation: PageOrientation,
    pub horizontal_gutter_even: f32,
    pub horizontal_gutter_uneven: f32,
    pub vertical_gutter: f32,
    pub staff_line_thickness: f32,
    pub stem_thickness: f32,
    pub beam_thickness: f32,
    pub beam_spacing: f32,
    pub barline_light: f32,
    pub barline_heavy: f32,

    pub note_size_grace: f32,
    pub note_size_cue: f32,
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
            staff_line_thickness: 1.1,
            stem_thickness: 1.,
            beam_thickness: 5.,
            beam_spacing: 1.5,
            barline_light: 1.875,
            barline_heavy: 5.,
            note_size_grace: 0.66,
            note_size_cue: 0.66,
        }
    }
}
