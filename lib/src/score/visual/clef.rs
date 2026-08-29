use crate::drawable::layoutable::Layoutable;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::layout::Layout;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::clef::Clef as SmuflClef;

pub struct Clef {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub scale: f32,

    pub clef: SmuflClef,
}

impl Clef {
    /// Scale factor applied to courtesy / mid-measure clef changes relative to
    /// the staff scale.
    pub const COURTESY_SCALE: f32 = 0.8;

    pub fn new(clef: crate::smufl::glyphs::clef::Clef) -> Clef {
        let mut result = Clef {
            xy: Default::default(),
            color: Color::BLACK,
            scale: 1.,

            width: 0.,
            height: 0.,

            clef,
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

    fn measure_size(&mut self) {
        let bbox = self.scale_box(&self.clef.bbox);
        self.width = bbox.width();
        self.height = bbox.height();
    }

    /// Sets the scale and re-derives width/height from it, so callers that
    /// rescale a clef after construction (e.g. courtesy clefs at 0.8x) don't
    /// end up positioning against a stale, pre-rescale size.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
        self.measure_size();
    }
}

impl ScoreElement for Clef {
    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Clef {
    fn measure(&mut self, _available: &XY) {
        self.measure_size();
    }

    /// Supplied origin x coordinate is left of clef, y coordinate is the line in the staff.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
