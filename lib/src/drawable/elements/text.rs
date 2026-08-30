use crate::drawable::drawable_element::Scale;
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

/// Which typeface a [`Text`] element is drawn in. The `family` is a plain
/// font-family name (as understood by CSS / a PDF `BaseFont`); it carries no
/// meaning of its own to this crate -- a consumer picks the names it wants and
/// each [`Canvas`](crate::drawable::canvas::Canvas) resolves them to its own
/// target (an SVG `font-family`, a canvas `ctx.font`, an embedded PDF font).
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
}

#[derive(Serialize, Clone)]
pub struct Text<'a> {
    pub text: &'a str,
    pub color: Color,
    pub font_size: f32,
    pub font: FontSpec<'a>,
    pub xy: XY,
    pub vertical_alignment: VerticalAlign,
    pub horizontal_alignment: HorizontalAlign,
}

impl<'a> Scale for Text<'a> {
    fn scale(&self, factor: f32, pivot: XY) -> Text<'a> {
        Text {
            xy: self.xy.scale_about(factor, pivot),
            font_size: self.font_size * factor,
            ..self.clone()
        }
    }
}
