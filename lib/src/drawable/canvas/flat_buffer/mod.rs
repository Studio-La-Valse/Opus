use crate::drawable::canvas::Canvas;
use crate::drawable::elements::circle::Circle;
use crate::drawable::elements::glyph::Glyph;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::{
    FontStyle, FontWeight, HorizontalAlign, Text, VerticalAlign,
};
use crate::geometry::color::Color;

/// Tags identifying each record in [`FlatBuffer::geometry`]. Keep in sync with the
/// decoder in `web/music-xml.js` (the tag constants, and [`FlatBuffer::page_table`]'s
/// layout).
pub const TAG_LINE: f32 = 0.0;
pub const TAG_RECT: f32 = 1.0;
pub const TAG_TEXT: f32 = 2.0;
pub const TAG_POLYGON: f32 = 3.0;
pub const TAG_CIRCLE: f32 = 4.0;
pub const TAG_GLYPH: f32 = 5.0;

/// Separates entries in [`FlatBuffer::text_blob`] and [`FlatBuffer::font_blob`].
/// Neither text elements nor font-family names contain this control character in
/// practice (SMuFL glyphs are single codepoints; family names are plain text).
pub const TEXT_DELIMITER: char = '\u{1F}';

/// `FontWeight::Bold` bit in a [`FlatBuffer::font_styles`] entry.
pub const FONT_STYLE_BOLD: u32 = 1;
/// `FontStyle::Italic` bit in a [`FlatBuffer::font_styles`] entry.
pub const FONT_STYLE_ITALIC: u32 = 2;

/// A canvas-ready encoding of a render pass: a flat, tagged numeric stream
/// (`geometry`) plus the text content of every `Text` record (`text_blob`) and
/// the distinct fonts they use (`font_blob` / `font_styles`), each in the order
/// first encountered and joined by [`TEXT_DELIMITER`].
///
/// Record layout per tag (all fields are `f32`, colors are `r, g, b, a`,
/// `strokeWidth < 0` means "no stroke"):
///
/// - `TAG_LINE`:    `x1, y1, x2, y2, r, g, b, a, strokeWidth`
/// - `TAG_RECT`:    `x, y, w, h, r, g, b, a, strokeWidth, sr, sg, sb, sa`
/// - `TAG_TEXT`:    `x, y, fontSize, r, g, b, a, hAlign, vAlign, fontIndex`
///   (text pulled from `text_blob` in order; `hAlign`/`vAlign` are `0/1/2`;
///   `fontIndex` selects an entry in `font_blob` / `font_styles`)
/// - `TAG_GLYPH`:   `x, y, fontSize, r, g, b, a, fontIndex`
///   (a single glyph, drawn from its origin: left-aligned on the alphabetic
///   baseline, so it carries no alignment fields. Its codepoint comes from
///   `text_blob` in the same order as a `TAG_TEXT` record's content does)
/// - `TAG_POLYGON`: `nPts, r, g, b, a, strokeWidth, sr, sg, sb, sa, x0, y0, x1, y1, ...`
/// - `TAG_CIRCLE`:  `cx, cy, radius, r, g, b, a, strokeWidth, sr, sg, sb, sa`
pub struct FlatBuffer {
    pub bounds: (f32, f32, f32, f32), // min_x, min_y, width, height
    pub geometry: Vec<f32>,
    /// Parallel page table, 5 `f32`s per page:
    /// `[geometry_start_index, origin_x, origin_y, width, height]`. Page `i`'s
    /// records span `geometry[page_table[5i] .. page_table[5(i+1)]]`, with the
    /// last page running to `geometry.len()`. Empty when the buffer was built
    /// via [`CanvasPainter::paint`](crate::drawable::canvas::CanvasPainter::paint)
    /// rather than
    /// [`paint_pages`](crate::drawable::canvas::CanvasPainter::paint_pages).
    /// Coordinates are in the same space as `geometry`.
    pub page_table: Vec<f32>,
    pub text_blob: String,
    /// The distinct font families every `TAG_TEXT` record's `fontIndex` points
    /// into, `TEXT_DELIMITER`-joined.
    pub font_blob: String,
    /// Parallel to `font_blob`: each entry is [`FONT_STYLE_BOLD`] /
    /// [`FONT_STYLE_ITALIC`] bit-or'd together (as an `f32`).
    pub font_styles: Vec<f32>,
}

/// A [`Canvas`] that encodes elements into a [`FlatBuffer`].
#[derive(Default)]
pub struct FlatBufferCanvas {
    bounds: (f32, f32, f32, f32),
    geometry: Vec<f32>,
    /// See [`FlatBuffer::page_table`]. Grows by one 5-`f32` record per
    /// [`Canvas::begin_page`] call; stays empty for a non-paged paint.
    page_table: Vec<f32>,
    text_blob: String,
    has_text: bool,
    /// `(family, style-flags)` for every distinct font seen, in first-seen order.
    fonts: Vec<(String, u32)>,
}

impl FlatBufferCanvas {
    pub fn new() -> Self {
        Self::default()
    }

    /// Index of `(family, flags)` in [`Self::fonts`], appending it if new.
    fn font_index(&mut self, family: &str, flags: u32) -> f32 {
        let idx = self
            .fonts
            .iter()
            .position(|(f, s)| f == family && *s == flags)
            .unwrap_or_else(|| {
                self.fonts.push((family.to_string(), flags));
                self.fonts.len() - 1
            });
        idx as f32
    }

    /// Appends one entry to [`Self::text_blob`], delimited from the last.
    /// Shared by text and glyph records, which draw from the one blob in the
    /// order the sink saw them.
    fn push_text(&mut self, content: &str) {
        if self.has_text {
            self.text_blob.push(TEXT_DELIMITER);
        }
        self.has_text = true;
        self.text_blob.push_str(content);
    }
}

impl Canvas for FlatBufferCanvas {
    type Output = FlatBuffer;

    fn begin(&mut self, bounds: (f32, f32, f32, f32)) {
        let (min_x, min_y, max_x, max_y) = bounds;
        self.bounds = (min_x, min_y, max_x - min_x, max_y - min_y);
    }

    fn begin_page(&mut self, origin_x: f32, origin_y: f32, width: f32, height: f32) {
        self.page_table.extend_from_slice(&[
            self.geometry.len() as f32,
            origin_x,
            origin_y,
            width,
            height,
        ]);
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

    fn draw_circle(&mut self, c: &Circle) {
        self.geometry.push(TAG_CIRCLE);
        self.geometry.push(c.xy.x);
        self.geometry.push(c.xy.y);
        self.geometry.push(c.radius);
        push_color(&mut self.geometry, c.color);
        self.geometry.push(c.stroke_width.unwrap_or(-1.0));
        push_color(
            &mut self.geometry,
            c.stroke_color.unwrap_or(Color::TRANSPARENT),
        );
    }

    fn draw_text(&mut self, t: &Text<'_>) {
        let flags = font_style_flags(t.font.weight, t.font.style);
        let font_index = self.font_index(t.font.family, flags);

        self.geometry.push(TAG_TEXT);
        self.geometry.push(t.xy.x);
        self.geometry.push(t.xy.y);
        self.geometry.push(t.font_size);
        push_color(&mut self.geometry, t.color);
        self.geometry
            .push(horizontal_align_tag(t.horizontal_alignment));
        self.geometry.push(vertical_align_tag(t.vertical_alignment));
        self.geometry.push(font_index);

        self.push_text(t.text);
    }

    fn draw_glyph(&mut self, g: &Glyph<'_>) {
        let flags = font_style_flags(g.font.weight, g.font.style);
        let font_index = self.font_index(g.font.family, flags);

        self.geometry.push(TAG_GLYPH);
        self.geometry.push(g.origin.x);
        self.geometry.push(g.origin.y);
        self.geometry.push(g.font_size);
        push_color(&mut self.geometry, g.color);
        self.geometry.push(font_index);

        self.push_text(g.glyph);
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
        let font_blob = self
            .fonts
            .iter()
            .map(|(family, _)| family.as_str())
            .collect::<Vec<_>>()
            .join(&TEXT_DELIMITER.to_string());
        let font_styles = self.fonts.iter().map(|(_, flags)| *flags as f32).collect();

        FlatBuffer {
            bounds: self.bounds,
            geometry: self.geometry,
            page_table: self.page_table,
            text_blob: self.text_blob,
            font_blob,
            font_styles,
        }
    }
}

fn font_style_flags(weight: FontWeight, style: FontStyle) -> u32 {
    let mut flags = 0;
    if weight == FontWeight::Bold {
        flags |= FONT_STYLE_BOLD;
    }
    if style == FontStyle::Italic {
        flags |= FONT_STYLE_ITALIC;
    }
    flags
}

fn push_color(out: &mut Vec<f32>, color: Color) {
    out.push(color.r() as f32);
    out.push(color.g() as f32);
    out.push(color.b() as f32);
    out.push(color.a());
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
