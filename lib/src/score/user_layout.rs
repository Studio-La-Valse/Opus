use crate::color::Color;

pub struct UserLayout {
    pub page_color: Option<Color>,
    pub foreground_color: Option<Color>,

    pub staff: Option<f32>,
    pub light_barline: Option<f32>,
    pub heavy_barline: Option<f32>,

    pub beam_thickness: Option<f32>,
    pub beam_spacing: Option<f32>,
    pub stem_thickness: Option<f32>,
}

impl Default for UserLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl UserLayout {
    pub fn new() -> UserLayout {
        UserLayout {
            page_color: None,
            foreground_color: None,

            staff: None,
            light_barline: None,
            heavy_barline: None,

            beam_thickness: None,
            beam_spacing: None,
            stem_thickness: None,
        }
    }
}
