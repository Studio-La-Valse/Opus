use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;

pub enum DrawableElement<'a> {
    Line(Line),
    Rect(Rect),
    Text(Text<'a>),
    Polygon(Box<Polygon>),
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
        DrawableElement::Polygon(Box::new(p))
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

pub fn to_svg<'a>(elements: Vec<DrawableElement<'a>>) -> String {
    let (min_x, min_y, max_x, max_y) = compute_bounds(&elements);

    let width = max_x - min_x;
    let height = max_y - min_y;

    let mut out = String::new();

    out.push_str(&format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{min_x} {min_y} {w} {h}" width="{w}" height="{h}">"#,
        min_x = min_x,
        min_y = min_y,
        w = width,
        h = height,
    ));
    out.push_str("\r\n");

    for el in elements {
        match el {
            DrawableElement::Line(l) => out.push_str(&format!(
                r#"<line x1="{x1}" y1="{y1}" x2="{x2}" y2="{y2}" stroke="{}" stroke-width="{}" />"#,
                l.stroke_color.to_hex(),
                l.stroke_width,
                x1 = l.start.x,
                y1 = l.start.y,
                x2 = l.end.x,
                y2 = l.end.y,
            )),

            DrawableElement::Rect(r) => out.push_str(&format!(
                r#"<rect x="{x}" y="{y}" width="{w}" height="{h}" fill="{}" stroke="{}" stroke-width="{}" />"#,
                r.color.to_hex(),
                r.stroke_color.map_or("none".to_string(), |s| s.to_hex()),
                r.stroke_width.unwrap_or(0.0),
                x = r.xy.x,
                y = r.xy.y,
                w = r.width,
                h = r.height,
            )),
            DrawableElement::Text(t) => out.push_str(&format!(
                r#"<text x="{x}" y="{y}" fill="{fill}" font-size="{fs}" font-family="{ff}" text-anchor="{ha}" dominant-baseline="{va}">{content}</text>"#,
                x = t.xy.x,
                y = t.xy.y,
                fill = t.color.to_hex(),
                fs = t.font_size,
                ff = t.font,
                ha = t.horizontal_alignment.to_svg(),
                va = t.vertical_alignment.to_svg(),
                content = xml_escape(t.text),
            )),
            DrawableElement::Polygon(p) => {
                let pts = p.pts
                    .iter()
                    .map(|xy| format!("{},{}", xy.x, xy.y))
                    .collect::<Vec<_>>()
                    .join(" ");

                out.push_str(&format!(
                    r#"<polygon points="{pts}" fill="{fill}" stroke="{stroke}" stroke-width="{sw}" />"#,
                    pts = pts,
                    fill = p.color.to_hex(),
                    stroke = p.stroke_color.as_ref().map_or("none".to_string(), |c| c.to_hex()),
                    sw = p.stroke_width.unwrap_or(0.0),
                ));
            }
        }

        out.push_str("\r\n")
    }

    out.push_str("</svg>");
    out
}

pub fn compute_bounds(elements: &Vec<DrawableElement<'_>>) -> (f32, f32, f32, f32) {
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

// Very small XML escape helper
fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}
