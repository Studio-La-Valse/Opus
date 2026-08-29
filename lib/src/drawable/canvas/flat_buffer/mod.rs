use crate::drawable::canvas::Canvas;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
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

/// A [`Canvas`] that encodes elements into a [`FlatBuffer`].
#[derive(Default)]
pub struct FlatBufferCanvas {
    bounds: (f32, f32, f32, f32),
    geometry: Vec<f32>,
    text_blob: String,
    has_text: bool,
}

impl FlatBufferCanvas {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Canvas for FlatBufferCanvas {
    type Output = FlatBuffer;

    fn begin(&mut self, bounds: (f32, f32, f32, f32)) {
        let (min_x, min_y, max_x, max_y) = bounds;
        self.bounds = (min_x, min_y, max_x - min_x, max_y - min_y);
    }

    fn draw_line(&mut self, l: &Line) {
        self.geometry.push(TAG_LINE);
        self.geometry.push(l.start.x);
        self.geometry.push(l.start.y);
        self.geometry.push(l.end.x);
        self.geometry.push(l.end.y);
        push_color(&mut self.geometry, l.stroke_color);
        self.geometry.push(l.stroke_width);
    }

    fn draw_rect(&mut self, r: &Rect) {
        self.geometry.push(TAG_RECT);
        self.geometry.push(r.xy.x);
        self.geometry.push(r.xy.y);
        self.geometry.push(r.width);
        self.geometry.push(r.height);
        push_color(&mut self.geometry, r.color);
        self.geometry.push(r.stroke_width.unwrap_or(-1.0));
        push_color(
            &mut self.geometry,
            r.stroke_color.unwrap_or(Color::TRANSPARENT),
        );
    }

    fn draw_text(&mut self, t: &Text<'_>) {
        self.geometry.push(TAG_TEXT);
        self.geometry.push(t.xy.x);
        self.geometry.push(t.xy.y);
        self.geometry.push(t.font_size);
        push_color(&mut self.geometry, t.color);
        self.geometry
            .push(horizontal_align_tag(t.horizontal_alignment));
        self.geometry.push(vertical_align_tag(t.vertical_alignment));

        if self.has_text {
            self.text_blob.push(TEXT_DELIMITER);
        }
        self.has_text = true;
        self.text_blob.push_str(t.text);
    }

    fn draw_polygon(&mut self, p: &Polygon) {
        self.geometry.push(TAG_POLYGON);
        self.geometry.push(p.pts.len() as f32);
        push_color(&mut self.geometry, p.color);
        self.geometry.push(p.stroke_width.unwrap_or(-1.0));
        push_color(
            &mut self.geometry,
            p.stroke_color.unwrap_or(Color::TRANSPARENT),
        );
        for pt in &p.pts {
            self.geometry.push(pt.x);
            self.geometry.push(pt.y);
        }
    }

    fn finish(self) -> FlatBuffer {
        FlatBuffer {
            bounds: self.bounds,
            geometry: self.geometry,
            text_blob: self.text_blob,
        }
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
