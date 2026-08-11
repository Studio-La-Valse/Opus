use crate::drawable::layoutable::Layoutable;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::layout::Layout;
use crate::score::user_layout::UserLayout;
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::number::Number;

pub struct TimeSignature {
    pub xy: XY,
    pub height: f32,
    pub width: f32,

    pub scale: f32,

    pub color: Color,

    pub num: Number,
    pub denom: Number,
}

impl TimeSignature {
    pub fn new(num: Number, denom: Number) -> TimeSignature {
        let mut result = TimeSignature {
            xy: XY::ZERO,
            height: 0.,
            width: 0.,
            scale: 1.,
            color: Color::BLACK,
            num,
            denom,
        };

        result.measure_width();

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

    /// The numerator is drawn in the top half of the available height, the
    /// denominator in the bottom half.
    pub fn num_xy(&self) -> XY {
        self.xy.mv(0., self.height / 4.)
    }

    pub fn denom_xy(&self) -> XY {
        self.xy.mv(0., self.height / 4. * 3.)
    }

    fn measure_width(&mut self) {
        let num_box = self.scale_box(&self.num.bbox);
        let denom_box = self.scale_box(&self.denom.bbox);
        self.width = num_box.width().max(denom_box.width());
    }

    /// Sets the scale and re-derives width from it, so callers that rescale
    /// a time signature after construction don't position against a stale,
    /// pre-rescale size.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
        self.measure_width();
    }
}

impl ScoreElement for TimeSignature {
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

impl Layoutable for TimeSignature {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
        self.measure_width();
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
