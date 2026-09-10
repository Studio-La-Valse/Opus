use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
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
        let mut result = Flag {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,

            size,
            scale: size.content_scale,

            color: Color::BLACK,

            glyph,
        };

        result.measure_size();

        result
    }

    /// World-space position of the glyph's own SMuFL stem-attachment anchor.
    /// After `arrange`, this coincides exactly with the stem corner it was
    /// aligned to.
    pub fn stem_anchor_world(&self) -> XY {
        self.scale_pt(&self.glyph.stem_anchor)
    }

    /// The same anchor as an offset from the glyph origin, which is what
    /// `arrange` subtracts to put the origin where the anchor lands on the stem.
    fn scaled_stem_anchor(&self) -> XY {
        self.glyph.stem_anchor.scale(self.unit())
    }

    fn measure_size(&mut self) {
        let bbox = self.scale_box(&self.glyph.bbox);
        self.width = bbox.width();
        self.height = bbox.height();
    }
}

impl Layoutable for Flag {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.scale = self.size.resolve(params);

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }

    fn measure(&mut self, _available: &XY, _params: LayoutParams<'_>) {
        self.measure_size();
    }

    /// Supplied origin is the point on the stem (its nw/sw corner) that the
    /// flag's own SMuFL stem-attachment anchor should land on - not the
    /// flag's own top-left corner.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin - self.scaled_stem_anchor();
    }
}
