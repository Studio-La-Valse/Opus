use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::dot::Dot;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::note_scale::NoteScale;
use crate::score::visual::placed::Placed;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::tie::{Tie, TieAnchor};
use crate::smufl::glyphs::notehead::Notehead;

pub struct Note {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub default_x: f32,
    pub staff_line: i32,
    pub staff: StaffIdx,

    /// What this note's size is derived from: the staff's content scaling and
    /// whether the note is normal, grace or cue. Kept unresolved so that the
    /// note-size factor can be re-decided on every render; see [`NoteScale`].
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful once that pass has run.
    pub scale: f32,

    pub color: Color,

    pub dots: Vec<Dot>,
    /// Resolved centre-to-centre dot step (and notehead-edge-to-first-dot gap),
    /// in world units; see [`AppDefaults::dot_spacing`](crate::score::app_defaults::AppDefaults).
    pub dot_spacing: f32,

    pub glyph: Notehead,
    pub accidental: Option<Accidental>,

    /// The tie leaving this note for the next one of its pitch, set by the
    /// content walk straight from the `<note>`'s own `<tie>` / `<tied>`. `None`
    /// for a note nothing is tied from -- which is most of them.
    ///
    /// Lives on the note it leaves rather than in a list of pairs beside the
    /// tree because that is what a tie *is*: a relation between a note and the
    /// one that follows it, in the same voice of the same part. Which note that
    /// turns out to be is left to
    /// [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties), which
    /// simply looks ahead for it.
    pub tie: Option<Tie>,
}

impl Note {
    /// Gap in tenths between a notehead's left edge and the right edge of its
    /// accidental. Scaled with the note, like everything else it owns.
    const ACCIDENTAL_GAP: f32 = 2.;

    pub fn new(
        glyph: Notehead,
        default_x: f32,
        staff: StaffIdx,
        staff_line: i32,
        size: NoteScale,
        dots: u8,
    ) -> Self {
        Note {
            glyph,
            accidental: None,

            tie: None,

            default_x,
            staff,
            staff_line,

            dots: (0..dots).map(|_| Dot::new(size)).collect(),
            dot_spacing: 0.,

            size,
            scale: size.content_scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    /// This note as one end of a tie -- see [`TieAnchor`]. Only meaningful once
    /// [`arrange_ctx`](Self::arrange_ctx) has placed it.
    pub fn tie_anchor(&self) -> TieAnchor {
        TieAnchor {
            left: self.xy,
            width: self.width,
            scale: self.scale,
            color: self.color,
            staff: self.staff,
            staff_line: self.staff_line,
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
        let gap = Note::ACCIDENTAL_GAP * self.scale;

        if let Some(accidental) = &mut self.accidental {
            accidental.arrange(&self.xy.mv(-gap, 0.));
        }
    }
}

impl Placed for Note {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Note {
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

        if let Some(accidental) = self.accidental.as_mut() {
            accidental.resolve_layout(params);
        }

        for dot in self.dots.iter_mut() {
            dot.resolve_layout(params);
        }
    }
}

impl Note {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        self.width = bbox.width();

        if let Some(accidental) = &mut self.accidental {
            // Sized with the note, not the staff: a cue or grace note's
            // accidental is reduced by the same factor its notehead is.
            accidental.rescale(self.scale);
            accidental.measure(available, params);
        }

        for dot in &mut self.dots {
            dot.measure(available, params);
        }
    }
}
