use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::staff_measure::StaffMeasure;
use std::collections::BTreeMap;

pub struct Staff {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    /// How many lines to draw, from `<staff-details><staff-lines>`, defaulting
    /// to [`Staff::DEFAULT_LINES`]. The lines run downwards from [`Staff::xy`],
    /// which stays the top line whatever the count: every element positioned
    /// against a staff -- clefs, key signatures, notes, rests -- is placed in
    /// half-spaces from that top line, so anchoring anywhere else would move
    /// the music relative to the lines it is written on.
    pub lines: usize,

    pub measures: BTreeMap<u32, StaffMeasure>,

    pub color: Color,
    pub line_thickness: f32,
    pub barline_thickness_light: f32,
    pub barline_thickness_heavy: f32,

    pub hidden: bool,

    pub scale: f32,

    pub distance_specified: Option<f32>,
    pub distance_final: f32,
}

impl Default for Staff {
    fn default() -> Staff {
        Staff {
            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            lines: Staff::DEFAULT_LINES,

            measures: Default::default(),

            color: Default::default(),
            line_thickness: Default::default(),
            barline_thickness_heavy: Default::default(),
            barline_thickness_light: Default::default(),

            hidden: Default::default(),

            scale: 1.,

            distance_specified: Default::default(),
            distance_final: Default::default(),
        }
    }
}

impl Staff {
    /// What a staff has unless `<staff-details><staff-lines>` says otherwise.
    pub const DEFAULT_LINES: usize = 5;

    /// The four spaces of a normal staff, which is the reference height every
    /// SMuFL glyph is drawn against: the font's em equals four staff spaces, so
    /// this is a property of the notation, not of any one staff. It stays four
    /// on a staff drawn with fewer or more lines -- a percussion staff's
    /// noteheads are the same size as everyone else's. Use [`Staff::height`] for
    /// how tall a particular staff actually is.
    pub const SPACES: usize = Staff::DEFAULT_LINES - 1;

    pub const DEFAULT_SPACE_SIZE: f32 = 10.;

    pub fn locate_measure_mut(&mut self, measure_number: &u32) -> Option<&mut StaffMeasure> {
        self.measures.get_mut(measure_number)
    }

    /// How much vertical room this staff takes: the distance from its top line
    /// to its bottom one, 10 tenths per space according to the MusicXML spec,
    /// but never less than a standard five-line staff's.
    ///
    /// A staff drawn with fewer lines than five is still a full staff -- it is
    /// read, barred and bracketed exactly as a five-line staff is, so the notes
    /// and rests around its lines have room and its neighbours do not collapse
    /// onto it. A staff of no lines at all, which `<staff-lines>0</staff-lines>`
    /// asks for, is nothing and takes no room.
    pub fn height(&self) -> f32 {
        if self.lines == 0 {
            0.
        } else {
            self.spaces().max(Staff::SPACES) as f32 * self.line_space()
        }
    }

    /// How far below [`Staff::xy`] -- the top of the standard-height block this
    /// staff occupies -- its first drawn line sits.
    ///
    /// Zero for a staff with its full five lines or more: the first line is the
    /// top of the block. A one-line staff is the exception the percussion
    /// convention makes: its single line stands in for the *middle* line of the
    /// five-line staff it isn't, so it is drawn two spaces down and everything
    /// pinned to it -- clef, time signature, rests, name, and the block's own
    /// barlines and brackets -- centres on it. Staves of two to four lines keep
    /// their lines at the top.
    pub fn top_line_offset(&self) -> f32 {
        if self.lines == 1 {
            Staff::SPACES as f32 / 2. * self.line_space()
        } else {
            0.
        }
    }

    /// How many spaces this staff's lines enclose.
    pub fn spaces(&self) -> usize {
        self.lines.saturating_sub(1)
    }

    pub fn line_space(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }

    pub fn set_lines(&mut self, lines: usize) {
        self.lines = lines;
        for measure in self.measures.values_mut() {
            measure.lines = lines;
        }
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        for measure in self.measures.values_mut() {
            measure.scale = scale;
        }
    }
}

impl Layoutable for Staff {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_thickness = user_layout
            .staff
            .or(score_defaults.appearance.staff)
            .unwrap_or(app_defaults.staff_line_thickness);

        self.barline_thickness_light = user_layout
            .light_barline
            .or(score_defaults.appearance.light_barline)
            .unwrap_or(app_defaults.barline_light);

        self.barline_thickness_heavy = user_layout
            .heavy_barline
            .or(score_defaults.appearance.heavy_barline)
            .unwrap_or(app_defaults.barline_heavy);

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }
    }

    fn measure(&mut self, _available: &XY, params: LayoutParams<'_>) {
        self.height = self.height();
        self.width = 0.;

        if self.hidden {
            self.height = 0.;
        }

        for measure in self.measures.values_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(available, params);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut _origin = self.xy;
        for measure in self.measures.values_mut() {
            measure.arrange(&_origin);

            _origin = _origin.mv(measure.width, 0.)
        }
    }
}
