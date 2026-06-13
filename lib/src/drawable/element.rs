use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
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

pub fn to_svg(elements: &[Element]) -> String {
    let (min_x, min_y, max_x, max_y) = compute_bounds(elements);

    let width  = max_x - min_x;
    let height = max_y - min_y;

    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x} {min_y} {w} {h}">"#,
        min_x = min_x,
        min_y = min_y,
        w = width,
        h = height,
    ));

    for el in elements {
        match el {
            Element::Line(l) => out.push_str(&format!(
                r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{}" stroke-width="{}" />"#,
                l.stroke_color.to_hex(),
                l.stroke_width,
                x1 = l.start.x,
                y1 = l.start.y,
                x2 = l.end.x,
                y2 = l.end.y,
            )),

            Element::Rect(r) => out.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{}" stroke="{}" stroke-width="{}" />"#,
                r.color.to_hex(),
                r.stroke_color.to_hex(),
                r.stroke_width,
                x = r.xy.x,
                y = r.xy.y,
                w = r.width,
                h = r.height,
            )),
        }
    }

    out.push_str("</svg>");
    out
}

pub fn compute_bounds(elements: &[Element]) -> (f32, f32, f32, f32) {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;

    for el in elements {
        match el {
            Element::Line(l) => {
                for p in [l.start, l.end] {
                    min_x = min_x.min(p.x);
                    min_y = min_y.min(p.y);
                    max_x = max_x.max(p.x);
                    max_y = max_y.max(p.y);
                }
            }
            Element::Rect(r) => {
                min_x = min_x.min(r.xy.x);
                min_y = min_y.min(r.xy.y);
                max_x = max_x.max(r.xy.x + r.width);
                max_y = max_y.max(r.xy.y + r.height);
            }
        }
    }

    (min_x, min_y, max_x, max_y)
}

