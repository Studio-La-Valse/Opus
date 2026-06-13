use crate::score::drawable::elements::line::Line;
use crate::score::drawable::elements::rect::Rect;
use serde::Serialize;

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Line(Line),
    Rect(Rect),
}

impl From<Line> for Element {
    fn from(value: Line) -> Self {
        Element::Line(value)
    }
}

impl From<Rect> for Element {
    fn from(r: Rect) -> Self {
        Element::Rect(r)
    }
}
