use crate::geometry::xy::XY;
use crate::score::visual::clef::Clef;
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::rest::Rest;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::time_signature::TimeSignature;

/// Element (Clef, time signature, key signature) spacing
const ELEMENT_PADDING: f32 = 5.;

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

    fn arrange_clef_start(&mut self) {
        let line_space = self.line_space() / 2.;
        if let Some(ref mut clef) = self.clef_start {
            clef.rescale(self.scale);

            let dy: f32 = clef.clef.line as f32 * line_space;
            let dx = ELEMENT_PADDING * self.scale;
            clef.arrange(&self.xy.mv(dx, dy));
        }
    }
    fn arrange_key_signature_start(&mut self) {
        let mut pos = self.xy;

        if let Some(clef) = &self.clef_start {
            // Take the right side of the clef if it exists.
            pos.x = clef.xy.x + clef.width;
        }

        let dx = ELEMENT_PADDING * self.scale;
        pos = pos.mv(dx, 0.);

        self.key_signature_start.arrange(&pos);
    }
    fn arrange_time_signature_start(&mut self) {
        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.rescale(self.scale);

            // ignore self.xy, take key signature xy + key signature width.
            let mut pos = self
                .key_signature_start
                .xy
                .mv(self.key_signature_start.width, 0.);

            let dx: f32 = ELEMENT_PADDING * self.scale;
            pos = pos.mv(dx, 0.);

            time_signature.arrange(&pos);
        }
    }
    fn arrange_time_signature_end(&mut self) {
        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            prepare_time_signature.rescale(self.scale);

            let pos = self
                .xy
                .mv(self.width - prepare_time_signature.width - 5., 0.);
            prepare_time_signature.arrange(&pos);
        }
    }
    fn arrange_clef_end(&mut self) {
        if let Some(ref mut clef) = self.clef_end {
            clef.rescale(self.scale * Clef::COURTESY_SCALE);

            let clef_origin = self.xy;
            let measure_right = clef_origin.mv(self.width, 0.);
            let arrange_left = measure_right.mv(-5. - clef.width, 0.);

            let dy = clef.clef.line as f32 * Staff::DEFAULT_SPACE_SIZE / 2. * self.scale;

            let origin = arrange_left.mv(0., dy);

            clef.arrange(&origin);
        }
    }
    fn arrange_rests(&mut self) {
        let staff_ctx = StaffCtx {
            hidden: false,
            distance_from_top: 0.,
            scaling: self.scale,
            lines: self.lines,
        };
        for rest in self.rests.iter_mut() {
            let dx: f32 = if rest.is_measure {
                self.width / 2.
            } else {
                rest.default_x.unwrap()
            };

            let glyph_origin = self.xy.mv(dx, 0.);
            rest.arrange_ctx(&glyph_origin, &staff_ctx);
        }
    }
}
impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = available.y;

        if let Some(ref mut clef) = self.clef_start {
            clef.measure(available, params);
        }

        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.measure(available, params);
        }

        self.key_signature_start.measure(available, params);

        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            prepare_time_signature.measure(available, params);
        }

        if let Some(ref mut clef) = self.clef_end {
            clef.measure(available, params);
        }

        for rest in self.rests.iter_mut() {
            rest.measure(available, params);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        self.arrange_clef_start();
        self.arrange_key_signature_start();
        self.arrange_time_signature_start();
        self.arrange_time_signature_end();
        self.arrange_clef_end();
        self.arrange_rests();
    }
}
