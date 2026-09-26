use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note_scale::NoteScale;
use crate::score::visual::placed::Placed;
use crate::smufl::glyphs::flag::Flag as SmuflFlag;

pub struct Flag {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    /// What this flag's size is derived from -- the same [`NoteScale`] as the
    /// stem it hangs off.
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful once that pass has run.
    pub scale: f32,

    pub color: Color,

    pub glyph: SmuflFlag,
}

impl Placed for Flag {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Flag {
    pub fn new(glyph: SmuflFlag, size: NoteScale) -> Self {
        Flag {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,

            size,
            scale: size.content_scale,

            color: Color::BLACK,

            glyph,
        }
    }

    /// World-space position of the glyph's own SMuFL stem-attachment anchor.
    /// After [`place`](Self::place), this coincides exactly with the stem corner it was
    /// aligned to.
    pub fn stem_anchor_world(&self) -> XY {
        self.scale_pt(&self.glyph.stem_anchor)
    }
}

impl Flag {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.scale = self.size.resolve(params);

        self.color = params.foreground_color();
    }
}

// ---- placement ----

impl Flag {
    /// Supplied origin is the point on the stem (its nw/sw corner) that the
    /// flag's own SMuFL stem-attachment anchor should land on - not the
    /// flag's own top-left corner.
    pub fn place(&mut self, origin: &XY) {
        // The offset from the glyph's origin to its stem-attachment anchor,
        // subtracted to put the origin where the anchor lands on the stem.
        let anchor_offset = self.glyph.stem_anchor.scale(self.unit());
        self.xy = *origin - anchor_offset;
    }
}
