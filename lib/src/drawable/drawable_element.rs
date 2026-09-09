use crate::drawable::elements::circle::Circle;
use crate::drawable::elements::glyph::Glyph;
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
    Glyph(Glyph<'a>),
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

impl<'a> From<Glyph<'a>> for DrawableElement<'a> {
    fn from(g: Glyph<'a>) -> Self {
        DrawableElement::Glyph(g)
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
            DrawableElement::Glyph(g) => g.scale(factor, pivot).into(),
            DrawableElement::Polygon(p) => p.scale(factor, pivot).into(),
        }
    }
}

/// Bounds accumulator: `(min_x, min_y, max_x, max_y)`, seeded so the first
/// element widens it from nothing.
type Bounds = (f32, f32, f32, f32);

const EMPTY_BOUNDS: Bounds = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);

fn accumulate_bounds(el: &DrawableElement<'_>, bounds: &mut Bounds) {
    let (min_x, min_y, max_x, max_y) = bounds;

    match el {
        DrawableElement::Line(l) => {
            for p in [l.start, l.end] {
                *min_x = min_x.min(p.x);
                *min_y = min_y.min(p.y);
                *max_x = max_x.max(p.x);
                *max_y = max_y.max(p.y);
            }
        }
        DrawableElement::Rect(r) => {
            *min_x = min_x.min(r.xy.x);
            *min_y = min_y.min(r.xy.y);
            *max_x = max_x.max(r.xy.x + r.width);
            *max_y = max_y.max(r.xy.y + r.height);
        }
        DrawableElement::Circle(c) => {
            *min_x = min_x.min(c.xy.x - c.radius);
            *min_y = min_y.min(c.xy.y - c.radius);
            *max_x = max_x.max(c.xy.x + c.radius);
            *max_y = max_y.max(c.xy.y + c.radius);
        }
        // Both boxes, arrived at from opposite ends: a text reserves the box it
        // is laid out in, a glyph reports the box its ink measures.
        DrawableElement::Text(t) => {
            *min_x = min_x.min(t.bounds.x_min());
            *min_y = min_y.min(t.bounds.y_min());
            *max_x = max_x.max(t.bounds.x_max());
            *max_y = max_y.max(t.bounds.y_max());
        }
        DrawableElement::Glyph(g) => {
            let bbox = g.bounds();
            *min_x = min_x.min(bbox.x_min());
            *min_y = min_y.min(bbox.y_min());
            *max_x = max_x.max(bbox.x_max());
            *max_y = max_y.max(bbox.y_max());
        }
        DrawableElement::Polygon(p) => {
            for xy in &p.pts {
                *min_x = min_x.min(xy.x);
                *min_y = min_y.min(xy.y);
                *max_x = max_x.max(xy.x);
                *max_y = max_y.max(xy.y);
            }
        }
    }
}

/// `(min_x, min_y, max_x, max_y)` over every element yielded. An empty iterator
/// yields the inverted sentinel `(f32::MAX, f32::MAX, f32::MIN, f32::MIN)`.
///
/// Takes any iterator of element references, so a caller whose elements are
/// split across several owners (e.g. one `Vec` per page) can pass
/// `pages.iter().flat_map(|p| &p.elements)` without concatenating first;
/// `&[DrawableElement]` and `&Vec<DrawableElement>` also coerce directly.
pub fn compute_bounds<'a>(
    elements: impl IntoIterator<Item = &'a DrawableElement<'a>>,
) -> (f32, f32, f32, f32) {
    let mut bounds = EMPTY_BOUNDS;
    for el in elements {
        accumulate_bounds(el, &mut bounds);
    }
    bounds
}
