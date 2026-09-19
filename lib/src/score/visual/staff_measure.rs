use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::clef::{Clef, ClefAnchor};
use crate::score::visual::key_signature::KeySignature;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::rest::Rest;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
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

    /// Places the opening clef `dx` right of this measure's left edge, and
    /// answers how far right its ink then reaches -- or `None` when the measure
    /// opens without a clef, which is every measure but the first of a system.
    ///
    /// The three `arrange_*_start` methods share this shape so that
    /// [`System::arrange_measure_starts`](crate::score::visual::system::System)
    /// can place each of them the same way: it decides the offset, they report
    /// what the next column has to clear.
    pub fn arrange_clef_start(&mut self, dx: f32) -> Option<f32> {
        let origin = self.xy;
        let scaling = self.scale;

        let clef = self.clef_start.as_mut()?;
        clef.place(ClefAnchor::LeftEdgeAt(origin.x + dx), origin.y, scaling);

        // The columns are worked out as offsets from the measure's own left
        // edge, so the edge reported back is one too.
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
        let origin = self.xy;
        let scaling = self.scale;
        let measure_right = origin.x + self.width;

        if let Some(clef) = self.clef_end.as_mut() {
            clef.place(ClefAnchor::GapBefore(measure_right), origin.y, scaling);
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
            // A whole-measure rest carries no position of its own and is centred
            // in whatever width the measure ended up with.
            let dx: f32 = rest.default_x.unwrap_or(self.width / 2.);

            let glyph_origin = self.xy.mv(dx, 0.);
            rest.arrange_ctx(&glyph_origin, &staff_ctx);
        }
    }

    /// Places everything this measure carries other than its three opening
    /// columns, which
    /// [`System::arrange_measure_starts`](crate::score::visual::system::System)
    /// places once every staff measure of the system is placed. Run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)
    /// after the container pass has placed this measure itself.
    pub fn arrange_content(&mut self) {
        self.arrange_time_signature_end();
        self.arrange_clef_end();
        self.arrange_rests();
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

    /// Sizes the measure's elements, scaling each to the staff first: a clef or
    /// time signature drawn at anything other than full size has to be the
    /// right size *here*, because its width is what the system's shared opening
    /// columns are worked out from, and a stale one would put every staff's
    /// elements in the wrong place rather than just its own.
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = available.y;

        if let Some(ref mut clef) = self.clef_start {
            clef.rescale(self.scale);
            clef.measure(available, params);
        }

        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.rescale(self.scale);
            time_signature.measure(available, params);
        }

        self.key_signature_start.rescale(self.scale);
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

    /// Places this measure's own position. Everything it draws is placed
    /// afterwards by [`arrange_content`](Self::arrange_content), which the
    /// container pass does not call: content placement is
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger)'s
    /// job, run once every container in the tree -- this measure's opening
    /// columns included -- has its final position.
    pub fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
