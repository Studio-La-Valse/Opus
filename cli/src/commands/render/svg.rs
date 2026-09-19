use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::svg::SvgCanvas;
use lib::score::visual::render_compositor::RenderedPage;
use std::time::Instant;

use super::paths::OutputTarget;

/// Renders each page to its own SVG document (`<stem>-p{n}.svg`) under
/// `target`'s directory. Each file is sized to its page rectangle, with a
/// `viewBox` origin of `(0, 0)` since elements are already page-local.
pub(super) fn write(pages: &[RenderedPage<'_>], target: &OutputTarget) {
    let time = Instant::now();

    for page in pages {
        let canvas = SvgCanvas::new((0., 0., page.width, page.height));
        let svg = CanvasPainter::new(canvas).paint(&page.elements);

        let path = target.page(page.number, "svg");
        target.write(&path, svg);
        println!("Written to: {}", path.display());
    }

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
