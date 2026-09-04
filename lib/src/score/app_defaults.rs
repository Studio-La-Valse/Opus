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

    /// Default font family for titles / work-level text. A generic CSS family so
    /// both a browser and a system font database can resolve it.
    pub title_font: String,
    /// Default font family for lyrics.
    pub lyric_font: String,
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
            title_font: "serif".to_string(),
            lyric_font: "serif".to_string(),
        }
    }
}
