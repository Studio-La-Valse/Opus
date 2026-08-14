use crate::drawable::drawable_element::{DrawableElement, compute_bounds};
use crate::drawable::elements::text::{HorizontalAlign, VerticalAlign};
use crate::geometry::color::Color;

/// Tags identifying each record in [`FlatBuffer::geometry`]. Keep in sync with the
/// decoder in `web/main.js`.
pub const TAG_LINE: f32 = 0.0;
pub const TAG_RECT: f32 = 1.0;
pub const TAG_TEXT: f32 = 2.0;
pub const TAG_POLYGON: f32 = 3.0;

/// Separates entries in [`FlatBuffer::text_blob`]. Text elements never contain this
/// control character in practice (SMuFL glyphs are single codepoints).
pub const TEXT_DELIMITER: char = '\u{1F}';

/// A canvas-ready encoding of a render pass: a flat, tagged numeric stream
/// (`geometry`) plus the text content of every `Text` record, in the order
/// encountered, joined by [`TEXT_DELIMITER`] (`text_blob`).
///
/// Record layout per tag (all fields are `f32`, colors are `r, g, b, a`,
/// `strokeWidth < 0` means "no stroke"):
///
/// - `TAG_LINE`:    `x1, y1, x2, y2, r, g, b, a, strokeWidth`
/// - `TAG_RECT`:    `x, y, w, h, r, g, b, a, strokeWidth, sr, sg, sb, sa`
/// - `TAG_TEXT`:    `x, y, fontSize, r, g, b, a, hAlign, vAlign` (text pulled
///   from `text_blob` in order; `hAlign`/`vAlign` are `0/1/2`)
/// - `TAG_POLYGON`: `nPts, r, g, b, a, strokeWidth, sr, sg, sb, sa, x0, y0, x1, y1, ...`
pub struct FlatBuffer {
    pub bounds: (f32, f32, f32, f32), // min_x, min_y, width, height
    pub geometry: Vec<f32>,
    pub text_blob: String,
}

pub fn to_flat_buffer(elements: Vec<DrawableElement<'_>>) -> FlatBuffer {
    let (min_x, min_y, max_x, max_y) = compute_bounds(&elements);
    let bounds = (min_x, min_y, max_x - min_x, max_y - min_y);

    let mut geometry = Vec::new();
    let mut text_blob = String::new();
    let mut has_text = false;

    for el in elements {
        match el {
            DrawableElement::Line(l) => {
                geometry.push(TAG_LINE);
                geometry.push(l.start.x);
                geometry.push(l.start.y);
                geometry.push(l.end.x);
                geometry.push(l.end.y);
                push_color(&mut geometry, l.stroke_color);
                geometry.push(l.stroke_width);
            }
            DrawableElement::Rect(r) => {
                geometry.push(TAG_RECT);
                geometry.push(r.xy.x);
                geometry.push(r.xy.y);
                geometry.push(r.width);
                geometry.push(r.height);
                push_color(&mut geometry, r.color);
                geometry.push(r.stroke_width.unwrap_or(-1.0));
                push_color(&mut geometry, r.stroke_color.unwrap_or(Color::TRANSPARENT));
            }
            DrawableElement::Text(t) => {
                geometry.push(TAG_TEXT);
                geometry.push(t.xy.x);
                geometry.push(t.xy.y);
                geometry.push(t.font_size);
                push_color(&mut geometry, t.color);
                geometry.push(horizontal_align_tag(t.horizontal_alignment));
                geometry.push(vertical_align_tag(t.vertical_alignment));

                if has_text {
                    text_blob.push(TEXT_DELIMITER);
                }
                has_text = true;
                text_blob.push_str(t.text);
            }
            DrawableElement::Polygon(p) => {
                geometry.push(TAG_POLYGON);
                geometry.push(p.pts.len() as f32);
                push_color(&mut geometry, p.color);
                geometry.push(p.stroke_width.unwrap_or(-1.0));
                push_color(&mut geometry, p.stroke_color.unwrap_or(Color::TRANSPARENT));
                for pt in &p.pts {
                    geometry.push(pt.x);
                    geometry.push(pt.y);
                }
            }
        }
    }

    FlatBuffer {
        bounds,
        geometry,
        text_blob,
    }
}

fn push_color(out: &mut Vec<f32>, color: Color) {
    out.push(color.r as f32);
    out.push(color.g as f32);
    out.push(color.b as f32);
    out.push(color.a);
}

fn horizontal_align_tag(align: HorizontalAlign) -> f32 {
    match align {
        HorizontalAlign::Left => 0.0,
        HorizontalAlign::Center => 1.0,
        HorizontalAlign::Right => 2.0,
    }
}

fn vertical_align_tag(align: VerticalAlign) -> f32 {
    match align {
        VerticalAlign::Top => 0.0,
        VerticalAlign::Middle => 1.0,
        VerticalAlign::Bottom => 2.0,
    }
}
