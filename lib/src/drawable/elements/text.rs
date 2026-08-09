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

#[derive(Serialize, Clone)]
pub struct Text<'a> {
    pub text: &'a str,
    pub color: Color,
    pub font_size: f32,
    pub font: &'a str,
    pub xy: XY,
    pub vertical_alignment: VerticalAlign,
    pub horizontal_alignment: HorizontalAlign,
}
