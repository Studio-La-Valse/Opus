use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::flag::Flag as SmuflFlag;

pub struct Flag {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
    pub color: Color,

    pub glyph: SmuflFlag,
}

impl Flag {
    pub fn new(glyph: SmuflFlag, scale: f32) -> Self {
        let mut result = Flag {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,

            scale,
            color: Color::BLACK,

            glyph,
        };

        result.measure_size();

        result
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled: BoundingBox = BoundingBox {
            xy: bbox.xy.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
            size: bbox.size.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }

    /// World-space position of the glyph's own SMuFL stem-attachment anchor.
    /// After `arrange`, this coincides exactly with the stem corner it was
    /// aligned to.
    pub fn stem_anchor_world(&self) -> XY {
        self.xy + self.scaled_stem_anchor()
    }

    fn scaled_stem_anchor(&self) -> XY {
        self.glyph
            .stem_anchor
            .scale(Staff::DEFAULT_SPACE_SIZE * self.scale)
    }

    fn measure_size(&mut self) {
        let bbox = self.scale_box(&self.glyph.bbox);
        self.width = bbox.width();
        self.height = bbox.height();
    }
}

impl ScoreElement for Flag {
    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Flag {
    fn measure(&mut self, _available: &XY, params: LayoutParams<'_>) {
        self._apply_layout(
            params.score_defaults,
            params.user_layout,
            params.app_defaults,
        );

        self.measure_size();
    }

    /// Supplied origin is the point on the stem (its nw/sw corner) that the
    /// flag's own SMuFL stem-attachment anchor should land on - not the
    /// flag's own top-left corner.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin - self.scaled_stem_anchor();
    }
}
