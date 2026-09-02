pub mod flat_buffer;
pub mod pdf;
pub mod svg;

use crate::drawable::drawable_element::{DrawableElement, compute_bounds};
use crate::drawable::elements::circle::Circle;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;
use crate::score::visual::render_compositor::RenderedPage;

/// A sink that turns [`DrawableElement`]s into a concrete rendering (an SVG
/// string, a canvas-ready flat buffer, a PDF page, ...).
///
/// [`CanvasPainter`] drives the lifecycle. Whole-score:
/// [`Canvas::begin`] with the element bounds, one `draw_*` call per element in
/// order, then [`Canvas::finish`]. Page-aware ([`CanvasPainter::paint_pages`]):
/// [`Canvas::begin`] with the bounds over *every* page, then per page a
/// [`Canvas::begin_page`] followed by that page's `draw_*` calls, then one
/// [`Canvas::finish`].
pub trait Canvas {
    type Output;

    /// Bounds are `(min_x, min_y, max_x, max_y)` over every element, as produced
    /// by [`compute_bounds`].
    fn begin(&mut self, bounds: (f32, f32, f32, f32));

    /// Called once before each page's elements when driven via
    /// [`CanvasPainter::paint_pages`], with the page's global origin and size in
    /// the same coordinate space as the elements. Default: no-op. Sinks that
    /// track page boundaries within one continuous stream (the flat buffer)
    /// override this; SVG and PDF ignore it because they build one surface per
    /// page instead.
    fn begin_page(&mut self, _origin_x: f32, _origin_y: f32, _width: f32, _height: f32) {}

    fn draw_line(&mut self, line: &Line);
    fn draw_rect(&mut self, rect: &Rect);
    fn draw_circle(&mut self, circle: &Circle);
    fn draw_text(&mut self, text: &Text<'_>);
    fn draw_polygon(&mut self, polygon: &Polygon);

    fn finish(self) -> Self::Output;
}

fn draw_one<C: Canvas>(canvas: &mut C, el: &DrawableElement<'_>) {
    match el {
        DrawableElement::Line(l) => canvas.draw_line(l),
        DrawableElement::Rect(r) => canvas.draw_rect(r),
        DrawableElement::Circle(c) => canvas.draw_circle(c),
        DrawableElement::Text(t) => canvas.draw_text(t),
        DrawableElement::Polygon(p) => canvas.draw_polygon(p),
    }
}

/// Walks [`DrawableElement`]s and dispatches each one to a [`Canvas`], either as
/// one flat slice ([`paint`](Self::paint)) or grouped into pages
/// ([`paint_pages`](Self::paint_pages)).
pub struct CanvasPainter<C: Canvas> {
    canvas: C,
}

impl<C: Canvas> CanvasPainter<C> {
    pub fn new(canvas: C) -> Self {
        Self { canvas }
    }

    pub fn paint(mut self, elements: &[DrawableElement<'_>]) -> C::Output {
        self.canvas.begin(compute_bounds(elements));

        for el in elements {
            draw_one(&mut self.canvas, el);
        }

        self.canvas.finish()
    }

    /// Drives the canvas over a sequence of [`RenderedPage`]s: one
    /// [`Canvas::begin`] with the bounds over every page's elements, then per
    /// page a [`Canvas::begin_page`] followed by that page's elements, then one
    /// [`Canvas::finish`]. Elements keep their global coordinates; the page
    /// origin/size passed to [`Canvas::begin_page`] is what lets a sink recover
    /// page boundaries from the otherwise-continuous stream.
    pub fn paint_pages(mut self, pages: &[RenderedPage<'_>]) -> C::Output {
        self.canvas
            .begin(compute_bounds(pages.iter().flat_map(|p| &p.elements)));

        for page in pages {
            self.canvas
                .begin_page(page.origin.x, page.origin.y, page.width, page.height);
            for el in &page.elements {
                draw_one(&mut self.canvas, el);
            }
        }

        self.canvas.finish()
    }
}
