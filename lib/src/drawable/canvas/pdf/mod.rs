//! A [`Canvas`] that renders one page of
//! [`DrawableElement`](crate::drawable::drawable_element::DrawableElement)s into
//! a PDF content stream, plus [`write_pdf`] which stitches the per-page streams
//! into a single multi-page PDF file with every font it uses embedded once.
//!
//! # Why this canvas is shaped differently from the SVG / flat-buffer ones
//!
//! Every sink is now fed one
//! [`RenderedPage`](crate::score::visual::render_compositor::RenderedPage) at a
//! time from
//! [`RenderCompositor::walk_pages`](crate::score::visual::render_compositor::RenderCompositor::walk_pages).
//! The SVG writer builds an independent document per page and the flat buffer
//! streams every page into one blob (with a page table), so both implement
//! [`Canvas`] once and let [`CanvasPainter`](super::CanvasPainter) drive them.
//! A PDF differs in two ways that this module owns rather than the painter:
//! each page is its own drawing surface (own media box + content stream), and
//! one set of embedded fonts is subset across *all* pages and written once. So
//! the flow is:
//!
//! 1. [`RenderCompositor::walk_pages`](crate::score::visual::render_compositor::RenderCompositor::walk_pages)
//!    hands you one [`RenderedPage`](crate::score::visual::render_compositor::RenderedPage)
//!    per physical page (elements still in global tenths).
//! 2. For each page, run `CanvasPainter::new(PdfPageCanvas::new(..)).paint(..)`
//!    to get a [`PdfPage`] (a media box + a content stream).
//! 3. Pass the whole `Vec<PdfPage>` and the [`FontSet`] to [`write_pdf`], which
//!    unions the per-page glyph sets and embeds each font once.
//!
//! # Fonts
//!
//! A `Text` element names a [`FontSpec`] (family + weight + style). The canvas
//! is constructed with a [`FontSet`] -- the fonts the caller has made available,
//! each carrying a parsed [`Face`] (for measuring advances / vertical metrics)
//! and the raw program bytes [`write_pdf`] subsets and embeds. A `Text` whose
//! `FontSpec` isn't in the set falls back to [`FontSet`]'s default entry.
//! Sourcing the bytes -- a bundled asset, a system font database -- is the
//! caller's job; this module only consumes what it's handed.
//!
//! ## `FontSource` seam (not yet built)
//!
//! The CLI builds its [`FontSet`] from `fontdb` + the installed system fonts.
//! The wasm target has no system font database, so a browser-side PDF export
//! would need the font programs bundled (`include_bytes!` of Bravura plus a
//! text face) or threaded through `load_score` and stashed in the score cache.
//! The clean shape is a `trait FontSource { fn resolve(&self, family: &str) ->
//! Option<&[u8]> }` with a `SystemFontSource` (CLI) and a `BundledFontSource`
//! (wasm) implementation, handed to whatever builds the [`FontSet`]. Until a
//! wasm `render_pdf` actually exists this stays a note rather than a trait.
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

mod document;

pub use document::{PdfPage, write_pdf};

use std::collections::{BTreeMap, BTreeSet};

use pdf_writer::{Content, Name, Str};
use ttf_parser::{Face, GlyphId};

use crate::drawable::canvas::Canvas;
use crate::drawable::elements::circle::Circle;
use crate::drawable::elements::glyph::Glyph;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::{
    FontSpec, FontStyle, FontWeight, HorizontalAlign, Text, VerticalAlign,
};
use crate::geometry::color::Color;
use crate::geometry::xy::XY;

/// One embeddable font: a parsed face for measurement plus the raw program
/// bytes [`write_pdf`] subsets and embeds.
pub struct EmbeddedFont<'f> {
    /// Family name, matched against a [`FontSpec::family`].
    pub family: String,
    pub weight: FontWeight,
    pub style: FontStyle,
    /// PDF `BaseFont` / PostScript name used in the font dictionaries.
    pub base_font: String,
    pub face: Face<'f>,
    /// The full OpenType/sfnt program.
    pub program: &'f [u8],
}

/// The fonts a [`PdfPageCanvas`] may draw text in. A [`FontSpec`] that matches
/// no entry falls back to `default`.
pub struct FontSet<'f> {
    fonts: Vec<EmbeddedFont<'f>>,
    default: usize,
}

impl<'f> FontSet<'f> {
    /// `fonts` must be non-empty; `default` indexes the fallback entry.
    pub fn new(fonts: Vec<EmbeddedFont<'f>>, default: usize) -> Self {
        assert!(
            default < fonts.len(),
            "FontSet default index {default} out of range for {} fonts",
            fonts.len()
        );
        Self { fonts, default }
    }

    /// A single-font set -- the common case where the only text is music
    /// glyphs, all in one font.
    pub fn single(font: EmbeddedFont<'f>) -> Self {
        Self::new(vec![font], 0)
    }

    /// Index of the entry `spec` selects, or the default entry's index.
    pub fn resolve(&self, spec: FontSpec<'_>) -> usize {
        self.fonts
            .iter()
            .position(|f| {
                f.family == spec.family && f.weight == spec.weight && f.style == spec.style
            })
            .unwrap_or(self.default)
    }

    pub(crate) fn get(&self, index: usize) -> &EmbeddedFont<'f> {
        &self.fonts[index]
    }
}

/// PDF resource name a font is registered under on every page (`/F0`, `/F1`, ...).
pub(crate) fn font_resource_name(index: usize) -> String {
    format!("F{index}")
}

/// A [`Canvas`] that renders a single page into a PDF content stream.
///
/// Construct one per page via [`PdfPageCanvas::new`], drive it with
/// [`CanvasPainter`](super::CanvasPainter), and collect the [`PdfPage`] it
/// produces for [`write_pdf`].
pub struct PdfPageCanvas<'f> {
    content: Content,
    fonts: &'f FontSet<'f>,

    /// Points per tenth: the MusicXML `scaling` ratio times 72/25.4.
    pt_per_tenth: f32,
    /// The page's global top-left in tenths, subtracted out by the CTM.
    origin: XY,
    /// Page height in points, the y-flip pivot in the CTM.
    page_height_pt: f32,
    page_width_pt: f32,

    /// Glyph ids referenced, keyed by the [`FontSet`] index that drew them.
    used_glyphs: BTreeMap<usize, BTreeSet<u16>>,
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
    /// * `fonts` -- the same [`FontSet`] [`write_pdf`] will embed; used here to
    ///   measure glyph advances and vertical metrics for text alignment.
    pub fn new(
        origin: XY,
        size_tenths: (f32, f32),
        pt_per_tenth: f32,
        fonts: &'f FontSet<'f>,
    ) -> Self {
        Self {
            content: Content::new(),
            fonts,
            pt_per_tenth,
            origin,
            page_width_pt: size_tenths.0 * pt_per_tenth,
            page_height_pt: size_tenths.1 * pt_per_tenth,
            used_glyphs: BTreeMap::new(),
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
        self.set_stroke_alpha(l.stroke_color.a());
        set_stroke_rgb(&mut self.content, l.stroke_color);
        self.content.set_line_width(l.stroke_width);
        self.content.move_to(l.start.x, l.start.y);
        self.content.line_to(l.end.x, l.end.y);
        self.content.stroke();
    }

    fn draw_rect(&mut self, r: &Rect) {
        let stroke = stroke_of(r.stroke_color, r.stroke_width);

        self.set_fill_alpha(r.color.a());
        set_fill_rgb(&mut self.content, r.color);
        if let Some((color, width)) = stroke {
            self.set_stroke_alpha(color.a());
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

    fn draw_circle(&mut self, c: &Circle) {
        let stroke = stroke_of(c.stroke_color, c.stroke_width);

        self.set_fill_alpha(c.color.a());
        set_fill_rgb(&mut self.content, c.color);
        if let Some((color, width)) = stroke {
            self.set_stroke_alpha(color.a());
            set_stroke_rgb(&mut self.content, color);
            self.content.set_line_width(width);
        }

        // Four cubic Bézier quadrants; `k` is the standard circle-approximation
        // control-point offset (4/3 * tan(pi/8)).
        let (cx, cy, r) = (c.xy.x, c.xy.y, c.radius);
        let k = 0.552_284_8 * r;
        self.content.move_to(cx + r, cy);
        self.content
            .cubic_to(cx + r, cy + k, cx + k, cy + r, cx, cy + r);
        self.content
            .cubic_to(cx - k, cy + r, cx - r, cy + k, cx - r, cy);
        self.content
            .cubic_to(cx - r, cy - k, cx - k, cy - r, cx, cy - r);
        self.content
            .cubic_to(cx + k, cy - r, cx + r, cy - k, cx + r, cy);
        self.content.close_path();
        if stroke.is_some() {
            self.content.fill_nonzero_and_stroke();
        } else {
            self.content.fill_nonzero();
        }
    }

    fn draw_text(&mut self, t: &Text<'_>) {
        if let Some(background) = t.background_rect() {
            self.draw_rect(&background);
        }

        let font_index = self.fonts.resolve(t.font);
        let face = &self.fonts.get(font_index).face;

        let units_per_em = f32::from(face.units_per_em());
        let glyph_scale = t.font_size / units_per_em;

        let mut gid_bytes: Vec<u8> = Vec::with_capacity(t.text.len() * 2);
        let mut advance = 0.0_f32;
        for ch in t.text.chars() {
            let gid = face.glyph_index(ch).map_or(0, |g| g.0);
            self.used_glyphs.entry(font_index).or_default().insert(gid);
            gid_bytes.extend_from_slice(&gid.to_be_bytes());
            advance += f32::from(face.glyph_hor_advance(GlyphId(gid)).unwrap_or(0)) * glyph_scale;
        }

        // The anchor is where the box's alignments put it; the shifts below are
        // what SVG's `text-anchor` and `dominant-baseline` then do to the run
        // relative to that anchor, done by hand because a PDF has no such
        // notion.
        let anchor = t.anchor();

        let tx = match t.horizontal_alignment {
            HorizontalAlign::Left => anchor.x,
            HorizontalAlign::Center => anchor.x - advance / 2.0,
            HorizontalAlign::Right => anchor.x - advance,
        };
        // In score space (y-down) the glyph is drawn upright with its ascender
        // above the baseline, so aligning the visual top means dropping the
        // baseline by the ascent. TODO: `ascender`/`descender` are the font-wide
        // hhea metrics; for a single SMuFL glyph the per-glyph bounding box
        // (`face.glyph_bounding_box`) would track the SVG `dominant-baseline`
        // rendering more closely -- revisit once text is checked against SVG.
        let ascent = f32::from(face.ascender()) * glyph_scale;
        let descent = f32::from(face.descender()) * glyph_scale; // negative
        let ty = match t.vertical_alignment {
            VerticalAlign::Bottom => anchor.y,
            VerticalAlign::Top => anchor.y + ascent,
            VerticalAlign::Middle => anchor.y + (ascent + descent) / 2.0,
        };

        self.set_fill_alpha(t.color.a());
        set_fill_rgb(&mut self.content, t.color);
        self.content.begin_text();
        self.content
            .set_font(Name(font_resource_name(font_index).as_bytes()), t.font_size);
        // Counter-flip against the CTM's y-flip so glyphs render upright.
        self.content.set_text_matrix([1.0, 0.0, 0.0, -1.0, tx, ty]);
        self.content.show(Str(&gid_bytes));
        self.content.end_text();
    }

    fn draw_glyph(&mut self, g: &Glyph<'_>) {
        let font_index = self.fonts.resolve(g.font);
        let face = &self.fonts.get(font_index).face;

        // A glyph arrives placed on its own origin, so unlike `draw_text` there
        // are no advances to accumulate and no ascender/descender to consult:
        // the origin is the baseline, and the text matrix takes it verbatim.
        let mut gid_bytes: Vec<u8> = Vec::with_capacity(g.glyph.len() * 2);
        for ch in g.glyph.chars() {
            let gid = face.glyph_index(ch).map_or(0, |gid| gid.0);
            self.used_glyphs.entry(font_index).or_default().insert(gid);
            gid_bytes.extend_from_slice(&gid.to_be_bytes());
        }

        self.set_fill_alpha(g.color.a());
        set_fill_rgb(&mut self.content, g.color);
        self.content.begin_text();
        self.content
            .set_font(Name(font_resource_name(font_index).as_bytes()), g.font_size);
        // Counter-flip against the CTM's y-flip so glyphs render upright.
        self.content
            .set_text_matrix([1.0, 0.0, 0.0, -1.0, g.origin().x, g.origin().y]);
        self.content.show(Str(&gid_bytes));
        self.content.end_text();
    }

    fn draw_polygon(&mut self, p: &Polygon) {
        let Some((first, rest)) = p.pts.split_first() else {
            return;
        };
        let stroke = stroke_of(p.stroke_color, p.stroke_width);

        self.set_fill_alpha(p.color.a());
        set_fill_rgb(&mut self.content, p.color);
        if let Some((color, width)) = stroke {
            self.set_stroke_alpha(color.a());
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
        color.r() as f32 / 255.0,
        color.g() as f32 / 255.0,
        color.b() as f32 / 255.0,
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
