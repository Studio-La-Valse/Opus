use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::placed::Placed;
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

impl Placed for TimeSignature {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
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
    /// Unlike the other [`Placed`] elements, a time signature is several glyphs
    /// rather than one, so a digit is boxed against where that digit was laid
    /// out rather than against the element's own origin.
    pub fn digit_box(&self, digit: &NumberDigit, at: XY) -> BoundingBox {
        digit.bbox.placed(at, self.unit())
    }

    /// The numerator sits one space above the middle of the staff, the
    /// denominator one space below it -- so on the usual five lines they land on
    /// the second and fourth, the halves each digit is drawn centred in.
    ///
    /// Measured from the middle outwards rather than as quarters of the staff's
    /// height, because a staff of one line is zero tenths tall: quartering that
    /// puts both halves of the signature in the same place, on top of each
    /// other.
    fn num_xy(&self) -> XY {
        self.xy.mv(0., self.height / 2. - self.unit())
    }

    fn denom_xy(&self) -> XY {
        self.xy.mv(0., self.height / 2. + self.unit())
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
