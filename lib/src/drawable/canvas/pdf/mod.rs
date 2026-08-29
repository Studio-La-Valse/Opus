//! A [`Canvas`] that renders one page of [`DrawableElement`]s into a PDF content
//! stream, plus [`write_pdf`] which stitches the per-page streams into a single
//! multi-page PDF file with the music font embedded once.
//!
//! # Why this canvas is shaped differently from the SVG / flat-buffer ones
//!
//! SVG and the flat buffer each describe a single drawing surface, so they
//! implement [`Canvas`] once and [`CanvasPainter`](super::CanvasPainter) drives
//! them over the whole score at once. A PDF is inherently multi-page and shares
//! one embedded font across every page, so the flow is:
//!
//! 1. [`RenderCompositor::walk_pages`](crate::score::visual::render_compositor::RenderCompositor::walk_pages)
//!    hands you one [`RenderedPage`](crate::score::visual::render_compositor::RenderedPage)
//!    per physical page (elements still in global tenths).
//! 2. For each page, run `CanvasPainter::new(PdfPageCanvas::new(..)).paint(..)`
//!    to get a [`PdfPage`] (a media box + a content stream).
//! 3. Pass the whole `Vec<PdfPage>` and the raw font bytes to [`write_pdf`].
//!
//! # Coordinate system
//!
//! Score coordinates are MusicXML tenths, x-right / y-down, with every page
//! placed at its global `origin`. PDF user space is points, x-right / y-up, with
//! each page's media box starting at `(0, 0)`. [`PdfPageCanvas::begin`] emits a
//! single CTM (`cm`) that folds all three differences -- tenths->points scale,
//! the page's global offset, and the y-flip -- into one matrix, so every
//! `draw_*` method can then emit raw score coordinates with no per-element math.
//! Text additionally carries a y-flip in its text matrix so glyphs come out
//! upright after the CTM flips the page.
//!
//! # Adding a wasm PDF export later
//!
//! `wasm/src/lib.rs` currently exposes `render` -> flat buffer. A PDF export
//! would mirror it: take the cached `Score` + `SmuflFont`, call
//! `compositor.walk_pages(&score, &font)`, build a `PdfPage` per entry exactly
//! as `cli/src/commands/render.rs` does, then return `write_pdf(&pages,
//! font_bytes)` as a `Vec<u8>` (wasm-bindgen marshals it to a `Uint8Array` the
//! browser can offer as a download). The font bytes need to reach wasm too --
//! either bundled at build time with `include_bytes!` or passed through
//! `load_score` alongside the existing metadata JSON.

mod document;

pub use document::{PdfPage, write_pdf};

use std::collections::BTreeSet;

use pdf_writer::{Content, Name, Str};
use ttf_parser::{Face, GlyphId};

use crate::drawable::canvas::Canvas;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::{HorizontalAlign, Text, VerticalAlign};
use crate::geometry::color::Color;
use crate::geometry::xy::XY;

/// Resource name the embedded music font is registered under in every page.
pub(crate) const FONT_NAME: Name<'static> = Name(b"F0");

/// A [`Canvas`] that renders a single page into a PDF content stream.
///
/// Construct one per page via [`PdfPageCanvas::new`], drive it with
/// [`CanvasPainter`](super::CanvasPainter), and collect the [`PdfPage`] it
/// produces for [`write_pdf`].
pub struct PdfPageCanvas<'f> {
    content: Content,
    face: &'f Face<'f>,

    /// Points per tenth: the MusicXML `scaling` ratio times 72/25.4.
    pt_per_tenth: f32,
    /// The page's global top-left in tenths, subtracted out by the CTM.
    origin: XY,
    /// Page height in points, the y-flip pivot in the CTM.
    page_height_pt: f32,
    page_width_pt: f32,

    used_glyphs: BTreeSet<u16>,
    /// Distinct fill / stroke alphas seen, in permille, so [`write_pdf`] can
    /// emit a matching `ExtGState` for each (see [`gs_fill_name`] /
    /// [`gs_stroke_name`]).
    used_alphas: BTreeSet<u16>,
    current_fill_alpha: f32,
    current_stroke_alpha: f32,
}

impl<'f> PdfPageCanvas<'f> {
    /// * `origin` / `size_tenths` -- the page's global placement and size, in
    ///   tenths, straight from
    ///   [`RenderedPage`](crate::score::visual::render_compositor::RenderedPage).
    /// * `pt_per_tenth` -- `scaling_millimeters / scaling_tenths * 72.0 / 25.4`.
    /// * `face` -- the same music font [`write_pdf`] will embed; used here only
    ///   to measure glyph advances and vertical metrics for text alignment.
    pub fn new(origin: XY, size_tenths: (f32, f32), pt_per_tenth: f32, face: &'f Face<'f>) -> Self {
        Self {
            content: Content::new(),
            face,
            pt_per_tenth,
            origin,
            page_width_pt: size_tenths.0 * pt_per_tenth,
            page_height_pt: size_tenths.1 * pt_per_tenth,
            used_glyphs: BTreeSet::new(),
            used_alphas: BTreeSet::new(),
            current_fill_alpha: 1.0,
            current_stroke_alpha: 1.0,
        }
    }

    /// Emits an `ExtGState` switch (`/GSf<permille> gs`) if the non-stroking
    /// alpha actually changed since the last paint op.
    fn set_fill_alpha(&mut self, alpha: f32) {
        if (alpha - self.current_fill_alpha).abs() < f32::EPSILON {
            return;
        }
        self.current_fill_alpha = alpha;
        let permille = alpha_permille(alpha);
        self.used_alphas.insert(permille);
        self.content
            .set_parameters(Name(gs_fill_name(permille).as_bytes()));
    }

    fn set_stroke_alpha(&mut self, alpha: f32) {
        if (alpha - self.current_stroke_alpha).abs() < f32::EPSILON {
            return;
        }
        self.current_stroke_alpha = alpha;
        let permille = alpha_permille(alpha);
        self.used_alphas.insert(permille);
        self.content
            .set_parameters(Name(gs_stroke_name(permille).as_bytes()));
    }
}

impl Canvas for PdfPageCanvas<'_> {
    type Output = PdfPage;

    fn begin(&mut self, _bounds: (f32, f32, f32, f32)) {
        // score (x, y)  ->  device (x', y'):
        //   x' = (x - origin.x) * s
        //   y' = page_height_pt - (y - origin.y) * s
        // as a PDF matrix [a b c d e f]:
        let s = self.pt_per_tenth;
        self.content.transform([
            s,
            0.0,
            0.0,
            -s,
            -self.origin.x * s,
            self.page_height_pt + self.origin.y * s,
        ]);
    }

    fn draw_line(&mut self, l: &Line) {
        self.set_stroke_alpha(l.stroke_color.a);
        set_stroke_rgb(&mut self.content, l.stroke_color);
        self.content.set_line_width(l.stroke_width);
        self.content.move_to(l.start.x, l.start.y);
        self.content.line_to(l.end.x, l.end.y);
        self.content.stroke();
    }

    fn draw_rect(&mut self, r: &Rect) {
        let stroke = stroke_of(r.stroke_color, r.stroke_width);

        self.set_fill_alpha(r.color.a);
        set_fill_rgb(&mut self.content, r.color);
        if let Some((color, width)) = stroke {
            self.set_stroke_alpha(color.a);
            set_stroke_rgb(&mut self.content, color);
            self.content.set_line_width(width);
        }

        self.content.rect(r.xy.x, r.xy.y, r.width, r.height);
        if stroke.is_some() {
            self.content.fill_nonzero_and_stroke();
        } else {
            self.content.fill_nonzero();
        }
    }

    fn draw_text(&mut self, t: &Text<'_>) {
        let units_per_em = f32::from(self.face.units_per_em());
        let glyph_scale = t.font_size / units_per_em;

        let mut gid_bytes: Vec<u8> = Vec::with_capacity(t.text.len() * 2);
        let mut advance = 0.0_f32;
        for ch in t.text.chars() {
            let gid = self.face.glyph_index(ch).map_or(0, |g| g.0);
            self.used_glyphs.insert(gid);
            gid_bytes.extend_from_slice(&gid.to_be_bytes());
            advance +=
                f32::from(self.face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0)) * glyph_scale;
        }

        let tx = match t.horizontal_alignment {
            HorizontalAlign::Left => t.xy.x,
            HorizontalAlign::Center => t.xy.x - advance / 2.0,
            HorizontalAlign::Right => t.xy.x - advance,
        };
        // In score space (y-down) the glyph is drawn upright with its ascender
        // above the baseline, so aligning the visual top means dropping the
        // baseline by the ascent. TODO: `ascender`/`descender` are the font-wide
        // hhea metrics; for a single SMuFL glyph the per-glyph bounding box
        // (`face.glyph_bounding_box`) would track the SVG `dominant-baseline`
        // rendering more closely -- revisit once text is checked against SVG.
        let ascent = f32::from(self.face.ascender()) * glyph_scale;
        let descent = f32::from(self.face.descender()) * glyph_scale; // negative
        let ty = match t.vertical_alignment {
            VerticalAlign::Bottom => t.xy.y,
            VerticalAlign::Top => t.xy.y + ascent,
            VerticalAlign::Middle => t.xy.y + (ascent + descent) / 2.0,
        };

        self.set_fill_alpha(t.color.a);
        set_fill_rgb(&mut self.content, t.color);
        self.content.begin_text();
        self.content.set_font(FONT_NAME, t.font_size);
        // Counter-flip against the CTM's y-flip so glyphs render upright.
        self.content.set_text_matrix([1.0, 0.0, 0.0, -1.0, tx, ty]);
        self.content.show(Str(&gid_bytes));
        self.content.end_text();
    }

    fn draw_polygon(&mut self, p: &Polygon) {
        let Some((first, rest)) = p.pts.split_first() else {
            return;
        };
        let stroke = stroke_of(p.stroke_color, p.stroke_width);

        self.set_fill_alpha(p.color.a);
        set_fill_rgb(&mut self.content, p.color);
        if let Some((color, width)) = stroke {
            self.set_stroke_alpha(color.a);
            set_stroke_rgb(&mut self.content, color);
            self.content.set_line_width(width);
        }

        self.content.move_to(first.x, first.y);
        for pt in rest {
            self.content.line_to(pt.x, pt.y);
        }
        self.content.close_path();
        if stroke.is_some() {
            self.content.fill_nonzero_and_stroke();
        } else {
            self.content.fill_nonzero();
        }
    }

    fn finish(self) -> PdfPage {
        PdfPage {
            media_box: [0.0, 0.0, self.page_width_pt, self.page_height_pt],
            content: self.content.finish().to_vec(),
            used_glyphs: self.used_glyphs,
            used_alphas: self.used_alphas,
        }
    }
}

/// Normalises a `stroke_color` / `stroke_width` pair to "stroke this, that
/// wide", or `None` when the element has no visible stroke -- matching the SVG
/// canvas, which only strokes when a color is set.
fn stroke_of(color: Option<Color>, width: Option<f32>) -> Option<(Color, f32)> {
    let color = color?;
    let width = width.unwrap_or(0.0);
    (width > 0.0).then_some((color, width))
}

fn set_fill_rgb(content: &mut Content, color: Color) {
    let [r, g, b] = rgb_unit(color);
    content.set_fill_rgb(r, g, b);
}

fn set_stroke_rgb(content: &mut Content, color: Color) {
    let [r, g, b] = rgb_unit(color);
    content.set_stroke_rgb(r, g, b);
}

fn rgb_unit(color: Color) -> [f32; 3] {
    [
        color.r as f32 / 255.0,
        color.g as f32 / 255.0,
        color.b as f32 / 255.0,
    ]
}

pub(crate) fn alpha_permille(alpha: f32) -> u16 {
    (alpha.clamp(0.0, 1.0) * 1000.0).round() as u16
}

pub(crate) fn gs_fill_name(permille: u16) -> String {
    format!("GSf{permille}")
}

pub(crate) fn gs_stroke_name(permille: u16) -> String {
    format!("GSs{permille}")
}
