use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::dot::Dot;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::NoteId;
use crate::score::visual::note_scale::NoteScale;
use crate::score::visual::placed::Placed;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::smufl::glyphs::rest::Rest as SmuflRest;

pub struct Rest {
    /// Identity within the score, so that something outside the tree can name
    /// this rest -- a mid-measure clef change drawn in front of it. A rest is a
    /// `<note>` in MusicXML and is counted as one; see [`NoteId`].
    pub id: NoteId,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    /// Where in its measure the document put this rest, or `None` for a
    /// whole-measure rest.
    ///
    /// The two cases used to be a `default_x: Option<f32>` and an `is_measure:
    /// bool` that always agreed with it, since a `<rest measure="yes">` is
    /// exactly the rest that carries no `default-x`: it stands for the measure
    /// rather than for a moment in it, and so is centred in whatever width the
    /// measure ends up with. One field cannot fall out of step with itself.
    pub default_x: Option<f32>,
    pub staff_line: i32,
    pub staff: StaffIdx,

    /// What this rest's size is derived from: the staff's content scaling and
    /// whether the rest is normal, grace or cue -- a cue passage's rests are
    /// reduced along with its notes. See [`NoteScale`].
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful once that pass has run.
    pub scale: f32,

    pub color: Color,

    pub glyph: SmuflRest,

    pub dots: Vec<Dot>,
    /// Resolved centre-to-centre dot step (and rest-edge-to-first-dot gap), in
    /// world units; see [`AppDefaults::dot_spacing`](crate::score::app_defaults::AppDefaults).
    pub dot_spacing: f32,
}

impl Rest {
    /// Half-space index, from the top of the block the staff occupies, of the
    /// line a rest is vertically centred on: the middle of the drawn lines. For
    /// the usual five that is the middle line (index 4). A one-line staff's
    /// single line stands in for that middle line and is drawn two spaces down,
    /// so a rest on it is centred at index 4 too -- the same place the clef and
    /// time signature land.
    pub fn center_line(staff_lines: usize) -> i32 {
        if staff_lines == 1 {
            4
        } else {
            staff_lines.saturating_sub(1) as i32
        }
    }

    pub fn new(
        id: NoteId,
        glyph: SmuflRest,
        default_x: Option<f32>,
        staff: StaffIdx,
        staff_line: i32,
        size: NoteScale,
        dots: u8,
    ) -> Self {
        Rest {
            id,
            glyph,

            default_x,
            staff,
            staff_line,

            size,
            scale: size.content_scale,

            dots: (0..dots).map(|_| Dot::new(size)).collect(),
            dot_spacing: 0.,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    /// here, origin is the origin of the staff measure, so adjust y coordinate for staff distance.
    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        self.arrange_glyph(origin, staff_ctx);
        self.arrange_dots(staff_ctx);
    }

    /// Rests carry no stem, so a dot landing on a staff line is nudged up (the
    /// default direction per engraving convention).
    fn arrange_dots(&mut self, staff_ctx: &StaffCtx) {
        let on_staff_line = self.staff_line.rem_euclid(2) == 0;
        let dy = if on_staff_line {
            -(Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling
        } else {
            0.
        };

        let base = self.xy.mv(self.width, dy);
        for (i, dot) in self.dots.iter_mut().enumerate() {
            let center = base.mv((i as f32 + 1.) * self.dot_spacing, 0.);
            dot.arrange(&center);
        }
    }

    fn arrange_glyph(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        let mut dy = staff_ctx.distance_from_top;
        dy += self.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        self.xy = XY {
            x: origin.x,
            y: origin.y + dy,
        };
    }
}

impl Placed for Rest {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Rest {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.scale = self.size.resolve(params);

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
        self.dot_spacing = user_layout.dot_spacing.unwrap_or(app_defaults.dot_spacing);

        for dot in self.dots.iter_mut() {
            dot.resolve_layout(params);
        }
    }
}

impl Rest {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        for dot in &mut self.dots {
            dot.measure(available, params);
        }

        self.width = bbox.width();
    }
}
