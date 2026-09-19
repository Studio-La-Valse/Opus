use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::clef::Clef;
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::rest::Rest;
use crate::score::visual::staff::Staff;
use crate::score::visual::time_signature::TimeSignature;

/// The resolved gaps between a measure's three opening columns, folded once
/// per arrange pass rather than per staff measure.
///
/// Values are in tenths, unscaled -- each staff measure's own `scale` is
/// applied at the point of use, so one resolved instance serves every staff
/// of the score regardless of cue sizing. Deliberately not a field on
/// [`StaffMeasure`]: it is config the arranging caller resolves, not derived
/// state cached on a visual node.
///
/// Follows the two-source pattern `TieMetrics` uses: user override, else app
/// default.
#[derive(Copy, Clone, Debug)]
pub struct MeasureStartPaddings {
    pub clef: f32,
    pub key_signature: f32,
    pub time_signature: f32,
}

impl MeasureStartPaddings {
    pub fn resolve(params: LayoutParams<'_>) -> Self {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        Self::from_sources(user_layout, app_defaults)
    }

    fn from_sources(user: &UserLayout, app: &AppDefaults) -> Self {
        MeasureStartPaddings {
            clef: user
                .measure_start_clef_padding
                .unwrap_or(app.measure_start_clef_padding),
            key_signature: user
                .measure_start_key_signature_padding
                .unwrap_or(app.measure_start_key_signature_padding),
            time_signature: user
                .measure_start_time_signature_padding
                .unwrap_or(app.measure_start_time_signature_padding),
        }
    }
}

pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,

    /// The line count of the staff this measure belongs to, mirrored the same
    /// way `scale` is so that a measure can hand a complete
    /// [`StaffCtx`] to the elements it arranges.
    pub lines: usize,

    pub rests: Vec<Rest>,

    // Clef left side of measure
    pub clef_start: Option<Clef>,
    // Key signature at left side of measure
    pub key_signature_start: KeySignature,
    // Time signature left side of measure
    pub time_signature_start: Option<TimeSignature>,
    // Time signature right of measure
    pub time_signature_end: Option<TimeSignature>,
    // Clef right of measure
    pub clef_end: Option<Clef>,
}

impl Default for StaffMeasure {
    fn default() -> Self {
        Self {
            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            scale: 1.,
            lines: Staff::DEFAULT_LINES,

            rests: Default::default(),

            clef_start: Default::default(),
            key_signature_start: Default::default(),
            time_signature_start: Default::default(),
            time_signature_end: Default::default(),
            clef_end: Default::default(),
        }
    }
}

impl StaffMeasure {
    pub fn line_space(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }
}
impl StaffMeasure {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        if let Some(ref mut clef) = self.clef_start {
            clef.resolve_layout(params);
        }

        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.resolve_layout(params);
        }

        self.key_signature_start.resolve_layout(params);

        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            prepare_time_signature.resolve_layout(params);
        }

        if let Some(ref mut clef) = self.clef_end {
            clef.resolve_layout(params);
        }

        for rest in self.rests.iter_mut() {
            rest.resolve_layout(params);
        }
    }
}
