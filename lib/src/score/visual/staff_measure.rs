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

    /// The gap this measure wants between one opening element and the next.
    /// Scaled with the staff, so a cue-sized staff asks for a smaller one.
    pub fn padding(&self) -> f32 {
        ELEMENT_PADDING * self.scale
    }

    /// Places the opening clef `dx` right of this measure's left edge, and
    /// answers how far right its ink then reaches -- or `None` when the measure
    /// opens without a clef, which is every measure but the first of a system.
    ///
    /// The three `arrange_*_start` methods share this shape so that
    /// [`System::arrange_measure_starts`](crate::score::visual::system::System)
    /// can place each of them the same way: it decides the offset, they report
    /// what the next column has to clear.
    pub fn arrange_clef_start(&mut self, dx: f32) -> Option<f32> {
        let xy = self.xy;
        let line_space = self.line_space() / 2.;

        let clef = self.clef_start.as_mut()?;
        let dy: f32 = clef.clef.line as f32 * line_space;
        clef.arrange(&xy.mv(dx, dy));

        Some(dx + clef.width)
    }

    /// Places the opening key signature. Always answers an edge, even for a
    /// staff carrying no accidentals: an empty key signature is zero wide, so
    /// the time signature still lands a padding right of the shared column
    /// rather than crowding whatever came before it.
    pub fn arrange_key_signature_start(&mut self, dx: f32) -> Option<f32> {
        self.key_signature_start.arrange(&self.xy.mv(dx, 0.));

        Some(dx + self.key_signature_start.width)
    }

    /// Places the opening time signature, which only the measures that open a
    /// score or announce a change carry.
    pub fn arrange_time_signature_start(&mut self, dx: f32) -> Option<f32> {
        let xy = self.xy;

        let time_signature = self.time_signature_start.as_mut()?;
        time_signature.arrange(&xy.mv(dx, 0.));

        Some(dx + time_signature.width)
    }
    fn arrange_time_signature_end(&mut self) {
        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            let pos = self
                .xy
                .mv(self.width - prepare_time_signature.width - 5., 0.);
            prepare_time_signature.arrange(&pos);
        }
    }
    fn arrange_clef_end(&mut self) {
        if let Some(ref mut clef) = self.clef_end {
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
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
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

    /// Sizes the measure's elements, scaling each to the staff first: a clef or
    /// time signature drawn at anything other than full size has to be the
    /// right size *here*, because its width is what the system's shared opening
    /// columns are worked out from, and a stale one would put every staff's
    /// elements in the wrong place rather than just its own.
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = available.y;

        if let Some(ref mut clef) = self.clef_start {
            clef.rescale(self.scale);
            clef.measure(available, params);
        }

        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.rescale(self.scale);
            time_signature.measure(available, params);
        }

        self.key_signature_start.measure(available, params);

        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            prepare_time_signature.rescale(self.scale);
            prepare_time_signature.measure(available, params);
        }

        if let Some(ref mut clef) = self.clef_end {
            clef.rescale(self.scale * Clef::COURTESY_SCALE);
            clef.measure(available, params);
        }

        for rest in self.rests.iter_mut() {
            rest.measure(available, params);
        }
    }

    /// Places everything this measure can place on its own. The three opening
    /// elements are not among them: where they go is decided across the whole
    /// system by
    /// [`System::arrange_measure_starts`](crate::score::visual::system::System),
    /// which runs once the staves have been placed -- the same way a tie, whose
    /// two ends may be systems apart, is left to
    /// [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties).
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        self.arrange_time_signature_end();
        self.arrange_clef_end();
        self.arrange_rests();
    }
}
