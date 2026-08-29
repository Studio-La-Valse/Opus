use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;

pub enum DrawableElement<'a> {
    Line(Line),
    Rect(Rect),
    Text(Text<'a>),
    Polygon(Polygon),
}

impl<'a> From<Line> for DrawableElement<'a> {
    fn from(value: Line) -> Self {
        DrawableElement::Line(value)
    }
}

impl<'a> From<Rect> for DrawableElement<'a> {
    fn from(r: Rect) -> Self {
        DrawableElement::Rect(r)
    }
}

impl<'a> From<Text<'a>> for DrawableElement<'a> {
    fn from(t: Text<'a>) -> Self {
        DrawableElement::Text(t)
    }
}

impl<'a> From<Polygon> for DrawableElement<'a> {
    fn from(p: Polygon) -> Self {
        DrawableElement::Polygon(p)
    }
}

pub fn scale_elem<'a>(element: &DrawableElement<'a>, scale: f32) -> DrawableElement<'a> {
    match element {
        DrawableElement::Line(l) => l.scale(scale).into(),
        DrawableElement::Rect(r) => r.scale(scale).into(),
        DrawableElement::Text(t) => t.scale(scale).into(),
        DrawableElement::Polygon(p) => p.scale(scale).into(),
    }
}

pub fn compute_bounds(elements: &[DrawableElement<'_>]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for el in elements {
        match el {
            DrawableElement::Line(l) => {
                for p in [l.start, l.end] {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
            DrawableElement::Rect(r) => {
                min_x = min_x.min(r.xy.x);
                min_y = min_y.min(r.xy.y);
                max_x = max_x.max(r.xy.x + r.width);
                max_y = max_y.max(r.xy.y + r.height);
            }
            DrawableElement::Text(t) => {
                min_x = min_x.min(t.xy.x);
                min_y = min_y.min(t.xy.y);
                max_x = max_x.max(t.xy.x);
                max_y = max_y.max(t.xy.y);
            }
            DrawableElement::Polygon(p) => {
                for xy in &p.pts {
                    min_x = min_x.min(xy.x);
                    min_y = min_y.min(xy.y);
                    max_x = max_x.max(xy.x);
                    max_y = max_y.max(xy.y);
                }
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}
