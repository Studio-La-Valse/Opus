use crate::color::Color;

pub struct AppDefaults {
    pub page_color: Color,
    pub foreground_color: Color,
    pub staff_line_thickness: f32,
    pub stem_thickness: f32,
    pub beam_thickness: f32,
    pub beam_spacing: f32,
}

impl Default for AppDefaults {
    fn default() -> Self {
        AppDefaults {
            page_color: Color::WHITE,
            foreground_color: Color::BLACK,
            staff_line_thickness: 1.1,
            stem_thickness: 1.,
            beam_thickness: 5.,
            beam_spacing: 1.5,
        }
    }
}
