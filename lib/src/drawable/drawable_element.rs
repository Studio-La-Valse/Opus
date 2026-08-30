use crate::drawable::elements::circle::Circle;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;
use crate::geometry::xy::XY;

pub enum DrawableElement<'a> {
    Line(Line),
    Rect(Rect),
    Circle(Circle),
    Text(Text<'a>),
    Polygon(Polygon),
}

/// Scales a drawable by `factor` about a `pivot` point, which stays fixed.
/// Implemented by every element and by [`DrawableElement`] itself.
pub trait Scale {
    fn scale(&self, factor: f32, pivot: XY) -> Self;
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

impl<'a> From<Circle> for DrawableElement<'a> {
    fn from(c: Circle) -> Self {
        DrawableElement::Circle(c)
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

impl<'a> Scale for DrawableElement<'a> {
    fn scale(&self, factor: f32, pivot: XY) -> DrawableElement<'a> {
        match self {
            DrawableElement::Line(l) => l.scale(factor, pivot).into(),
            DrawableElement::Rect(r) => r.scale(factor, pivot).into(),
            DrawableElement::Circle(c) => c.scale(factor, pivot).into(),
            DrawableElement::Text(t) => t.scale(factor, pivot).into(),
            DrawableElement::Polygon(p) => p.scale(factor, pivot).into(),
        }
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
            DrawableElement::Circle(c) => {
                min_x = min_x.min(c.xy.x - c.radius);
                min_y = min_y.min(c.xy.y - c.radius);
                max_x = max_x.max(c.xy.x + c.radius);
                max_y = max_y.max(c.xy.y + c.radius);
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
