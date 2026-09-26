use crate::drawable::drawable_element::Scale;
use crate::drawable::elements::rect::Rect;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use serde::Serialize;

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}

impl HorizontalAlign {
    pub fn to_svg(self) -> &'static str {
        match self {
            HorizontalAlign::Left => "start",
            HorizontalAlign::Center => "middle",
            HorizontalAlign::Right => "end",
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize)]
pub enum VerticalAlign {
    #[default]
    Top,
    Middle,
    Bottom,
}

impl VerticalAlign {
    pub fn to_svg(self) -> &'static str {
        match self {
            VerticalAlign::Top => "hanging",
            VerticalAlign::Middle => "middle",
            VerticalAlign::Bottom => "baseline",
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum FontWeight {
    #[default]
    Normal,
    Bold,
}

impl FontWeight {
    pub fn to_css(self) -> &'static str {
        match self {
            FontWeight::Normal => "normal",
            FontWeight::Bold => "bold",
        }
    }
}

#[derive(Default, Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
}

impl FontStyle {
    pub fn to_css(self) -> &'static str {
        match self {
            FontStyle::Normal => "normal",
            FontStyle::Italic => "italic",
        }
    }
}

/// Which typeface a [`Text`] element is drawn in. The `family` is a CSS
/// `font-family` value: one name, or a comma-separated list to fall through,
/// multi-word names quoted. It carries no meaning of its own to this crate -- a
/// consumer picks the names it wants and each
/// [`Canvas`](crate::drawable::canvas::Canvas) resolves them to its own target
/// (an SVG `font-family` and a canvas `ctx.font` take the list as it is; a PDF
/// writer walks [`families`](Self::families) for the first one installed).
#[derive(Debug, Clone, Copy, Serialize)]
pub struct FontSpec<'a> {
    pub family: &'a str,
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl<'a> FontSpec<'a> {
    /// A regular-weight, upright face of `family`.
    pub fn plain(family: &'a str) -> FontSpec<'a> {
        FontSpec {
            family,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }

    /// The names in `family`, most preferred first, with their CSS quotes
    /// stripped: `Edwin, 'Century Schoolbook', serif` yields `Edwin`,
    /// `Century Schoolbook`, `serif`.
    pub fn families(&self) -> impl Iterator<Item = &'a str> {
        self.family
            .split(',')
            .map(|name| name.trim().trim_matches(|c| c == '\'' || c == '"'))
            .filter(|name| !name.is_empty())
    }
}

/// A run of text laid out inside a box.
///
/// The box is the element's geometry -- there is no separate anchor point. The
/// alignments say where in the box the text sits, and the anchor a sink draws
/// from falls out of the two ([`anchor`](Self::anchor)): horizontally at the
/// box's left edge, middle or right edge, vertically hanging from its top line,
/// centred on its middle line, or sitting on its bottom line as a baseline.
///
/// The box is a *layout* box, not a measurement of the ink. This crate has no
/// font metrics for a text face -- each [`Canvas`](crate::drawable::canvas::Canvas)
/// resolves a [`FontSpec`] against a different target, a browser resolving a
/// family or a PDF writer's embedded face -- so the box is what a producer
/// reserves for the text, and it is the producer's business whether the text
/// fills it. That is what a text box means in any editor that has one, and it is
/// what makes a text element measurable at all: a run whose extent nothing here
/// can compute still occupies a rectangle somebody chose.
///
/// A [`Glyph`](super::glyph::Glyph) is the other case and works the other way
/// round: its box *is* measured, straight from the font metadata, and its
/// position is the origin the metadata registers it against.
#[derive(Serialize, Clone)]
pub struct Text<'a> {
    pub text: &'a str,
    pub color: Color,
    pub font_size: f32,
    pub font: FontSpec<'a>,
    /// The box the text is laid out in, in world space.
    pub bounds: BoundingBox,
    pub vertical_alignment: VerticalAlign,
    pub horizontal_alignment: HorizontalAlign,
    /// Filled over `bounds`, behind the text. `None` leaves whatever is under
    /// the box showing through, which is what every text does today.
    pub background: Option<Color>,
}

impl<'a> Text<'a> {
    /// The point a sink draws the text from, picked out of the box by the
    /// alignments. Paired with the matching baseline / anchor mode -- which is
    /// what [`VerticalAlign::to_svg`] and [`HorizontalAlign::to_svg`] name --
    /// this places the text within the box.
    ///
    /// A zero-sized box collapses every alignment onto its own corner, so a
    /// producer with nothing to reserve can still position text as a bare point.
    pub fn anchor(&self) -> XY {
        let x = match self.horizontal_alignment {
            HorizontalAlign::Left => self.bounds.x_min(),
            HorizontalAlign::Center => self.bounds.x_min() + self.bounds.width() / 2.0,
            HorizontalAlign::Right => self.bounds.x_max(),
        };
        let y = match self.vertical_alignment {
            VerticalAlign::Top => self.bounds.y_min(),
            VerticalAlign::Middle => self.bounds.y_min() + self.bounds.height() / 2.0,
            VerticalAlign::Bottom => self.bounds.y_max(),
        };

        XY { x, y }
    }

    /// The fill behind the text as an ordinary [`Rect`], for a sink to paint
    /// before the run. A background *is* a filled rectangle, so every canvas
    /// draws it with the rectangle painting it already has rather than growing
    /// a second way to fill one.
    pub fn background_rect(&self) -> Option<Rect> {
        self.background.map(|color| Rect {
            xy: self.bounds.xy,
            width: self.bounds.width(),
            height: self.bounds.height(),
            color,
            stroke_color: None,
            stroke_width: None,
        })
    }
}

impl<'a> Scale for Text<'a> {
    fn scale(&self, factor: f32, pivot: XY) -> Text<'a> {
        Text {
            bounds: self.bounds.scale_about(factor, pivot),
            font_size: self.font_size * factor,
            ..self.clone()
        }
    }
}
