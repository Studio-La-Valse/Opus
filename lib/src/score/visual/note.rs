use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::dot::Dot;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::smufl::glyphs::notehead::Notehead;

/// Identifies one [`Note`] within a [`Score`](crate::score::visual::score::Score).
///
/// The visual tree is a pure containment hierarchy, so it cannot express a
/// relation between two notes that sit in different branches of it -- which is
/// exactly what a tie is. Ids let [`Tie`](crate::score::visual::tie::Tie) name
/// its two endpoints without needing a pointer into the tree.
///
/// Handed out by
/// [`WalkCursor`](crate::score::walk_cursor::WalkCursor), so that every visitor
/// in the chain agrees on which note it is looking at. They are assigned during
/// the cached `walk_document` half of the pipeline, so they stay stable across
/// the repeated `arrange_score` calls the wasm render path makes. Only
/// uniqueness is meaningful -- never read anything into the values themselves.
#[derive(Default, Ord, PartialOrd, Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub struct NoteId(u32);

impl NoteId {
    /// The next id in sequence.
    pub fn next(self) -> Self {
        NoteId(self.0 + 1)
    }
}

impl From<u32> for NoteId {
    fn from(value: u32) -> Self {
        NoteId(value)
    }
}

pub struct Note {
    pub id: NoteId,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub default_x: f32,
    pub staff_line: i32,
    pub staff: StaffIdx,

    pub scale: f32,

    pub color: Color,

    pub dots: Vec<Dot>,
    /// Resolved centre-to-centre dot step (and notehead-edge-to-first-dot gap),
    /// in world units; see [`AppDefaults::dot_spacing`](crate::score::app_defaults::AppDefaults).
    pub dot_spacing: f32,

    pub glyph: Notehead,
    pub accidental: Option<Accidental>,
}

impl Note {
    pub fn new(
        id: NoteId,
        glyph: Notehead,
        default_x: f32,
        staff: StaffIdx,
        staff_line: i32,
        scale: f32,
        dots: u8,
    ) -> Self {
        Note {
            id,
            glyph,
            accidental: None,

            default_x,
            staff,
            staff_line,

            dots: (0..dots).map(|_| Dot::new(scale)).collect(),
            dot_spacing: 0.,

            scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        let staff_top = origin.mv(0., staff_ctx.distance_from_top);
        let note_dy =
            self.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        let note_top = staff_top.mv(0., note_dy);
        self.xy = note_top.mv(self.default_x, 0.);

        self.arrange_accidental();
    }

    fn arrange_accidental(&mut self) {
        if let Some(accidental) = &mut self.accidental {
            accidental.arrange(&self.xy.mv(-2., 0.));
        }
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled: BoundingBox = BoundingBox {
            xy: bbox.xy.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
            size: bbox.size.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }

    /// Scales a normalized point to current position and scale.
    pub fn scale_pt(&self, xy: &XY) -> XY {
        let scaled = XY {
            x: xy.x * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            y: xy.y * (Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }
}

impl Note {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
        self.dot_spacing = user_layout.dot_spacing.unwrap_or(app_defaults.dot_spacing);
    }
}

impl Note {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        self.width = bbox.width();

        if let Some(accidental) = &mut self.accidental {
            accidental.measure(available, params);
        }

        for dot in &mut self.dots {
            dot.measure(available, params);
        }
    }
}
