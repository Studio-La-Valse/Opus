use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::LayoutParams;
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
        TimeSignature {
            xy: XY::ZERO,
            height: 0.,
            width: 0.,
            scale: 1.,
            color: Color::BLACK,
            num,
            denom,
        }
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
    /// `placed`.
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

    /// Sets the scale. The width is not re-derived here: it is the measure
    /// pass's to settle, so a caller that positions against it straight after
    /// rescaling has to
    /// [`measure_time_signature`](crate::score::visual::arranger::ScoreMeasurement::measure_time_signature)
    /// first.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
    }
}

impl TimeSignature {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.color = params.foreground_color();
    }
}
