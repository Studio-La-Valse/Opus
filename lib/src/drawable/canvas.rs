pub mod flat_buffer;
pub mod svg;

use crate::drawable::drawable_element::{DrawableElement, compute_bounds};
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::drawable::elements::rect::Rect;
use crate::drawable::elements::text::Text;

/// A sink that turns [`DrawableElement`]s into a concrete rendering (an SVG
/// string, a canvas-ready flat buffer, a PDF page, ...).
///
/// [`CanvasPainter`] drives the lifecycle: exactly one [`Canvas::begin`] with the
/// element bounds, then one `draw_*` call per element in order, then
/// [`Canvas::finish`].
pub trait Canvas {
    type Output;

    /// Bounds are `(min_x, min_y, max_x, max_y)` over every element, as produced
    /// by [`compute_bounds`].
    fn begin(&mut self, bounds: (f32, f32, f32, f32));

    fn draw_line(&mut self, line: &Line);
    fn draw_rect(&mut self, rect: &Rect);
    fn draw_text(&mut self, text: &Text<'_>);
    fn draw_polygon(&mut self, polygon: &Polygon);

    fn finish(self) -> Self::Output;
}

/// Walks a slice of [`DrawableElement`]s and dispatches each one to a [`Canvas`].
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
            match el {
                DrawableElement::Line(l) => self.canvas.draw_line(l),
                DrawableElement::Rect(r) => self.canvas.draw_rect(r),
                DrawableElement::Text(t) => self.canvas.draw_text(t),
                DrawableElement::Polygon(p) => self.canvas.draw_polygon(p),
            }
        }

        self.canvas.finish()
    }
}
