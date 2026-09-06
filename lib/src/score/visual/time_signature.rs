use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::number::{Number, NumberDigit};

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

    /// World-space bounding box of one digit drawn at `at`, for callers that
    /// need the glyph's ink extent rather than its origin (the debug overlay).
    pub fn digit_box(&self, digit: &NumberDigit, at: XY) -> BoundingBox {
        let unit = self.unit();
        let scaled = BoundingBox {
            xy: digit.bbox.xy.scale(unit),
            size: digit.bbox.size.scale(unit),
        };

        scaled.mv(at.x, at.y)
    }

    /// World units per staff space at this element's current scale.
    fn unit(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }

    /// The numerator is drawn in the top half of the available height, the
    /// denominator in the bottom half.
    fn num_xy(&self) -> XY {
        self.xy.mv(0., self.height / 4.)
    }

    fn denom_xy(&self) -> XY {
        self.xy.mv(0., self.height / 4. * 3.)
    }

    /// Every digit of the numerator with the position to draw it at, laid out
    /// left to right and centred over the element's own width. See
    /// [`placed`](Self::placed).
    pub fn num_digits(&self) -> impl Iterator<Item = (&NumberDigit, XY)> {
        self.placed(&self.num, self.num_xy())
    }

    /// Every digit of the denominator, laid out the same way.
    pub fn denom_digits(&self) -> impl Iterator<Item = (&NumberDigit, XY)> {
        self.placed(&self.denom, self.denom_xy())
    }

    /// Steps `number`'s digits across from `origin`, centring the run within
    /// `self.width`.
    ///
    /// Centring is what makes a `12` over an `8` sit right; with both halves a
    /// single digit it only matters to the extent that the digits differ in
    /// width (a `1` is much narrower than a `4`).
    fn placed<'a>(
        &'a self,
        number: &'a Number,
        origin: XY,
    ) -> impl Iterator<Item = (&'a NumberDigit, XY)> {
        let unit = self.unit();
        let left = origin.x + (self.width - number.advance() * unit) / 2.;

        number.placed().map(move |(digit, offset)| {
            (
                digit,
                XY {
                    x: left + offset * unit,
                    y: origin.y,
                },
            )
        })
    }

    fn measure_width(&mut self) {
        let unit = self.unit();
        self.width = (self.num.advance() * unit).max(self.denom.advance() * unit);
    }

    /// Sets the scale and re-derives width from it, so callers that rescale
    /// a time signature after construction don't position against a stale,
    /// pre-rescale size.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
        self.measure_width();
    }
}

impl TimeSignature {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for TimeSignature {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = available.y;
        self.measure_width();
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
