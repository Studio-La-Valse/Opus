use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::svg::SvgCanvas;
use lib::drawable::drawable_element::DrawableElement;
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::score::visual::score::Score;
use lib::smufl::smufl_font::SmuflFont;
use std::fs;
use std::time::Instant;

/// Renders `score` to a single SVG document and writes it to `out`.
pub(super) fn write(
    score: &Score,
    font: &SmuflFont,
    title_font: &str,
    lyric_font: &str,
    debug: bool,
    out: &str,
) {
    let mut time = Instant::now();

    let fonts = RenderFonts::create(font, title_font, lyric_font);

    let mut elements: Vec<DrawableElement<'_>> = RenderCompositor::base().walk(score, &fonts);

    if debug {
        elements.extend(RenderCompositor::debug().walk(score, &fonts));
    }

    println!("Render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let svg = CanvasPainter::new(SvgCanvas::new()).paint(&elements);
    fs::write(out, svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
