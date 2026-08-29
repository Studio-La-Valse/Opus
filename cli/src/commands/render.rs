use crate::commands::print_issues;
use clap::Args;
use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::pdf::{self, EmbeddedFont, FontSet, PdfPage, PdfPageCanvas};
use lib::drawable::canvas::svg::SvgCanvas;
use lib::drawable::drawable_element::DrawableElement;
use lib::drawable::elements::text::{FontStyle, FontWeight};
use lib::geometry::color::Color;
use lib::score::app_defaults::AppDefaults;
use lib::score::engrave::{EngravedScore, Stage, engrave};
use lib::score::layout::Defaults;
use lib::score::page_orientation::PageOrientation;
use lib::score::user_layout::UserLayout;
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::score::visual::score::Score;
use lib::smufl::smufl_font::SmuflFont;
use lib::xml::validate::ValidationCtx;
use lib::xml::visitor::{DefaultVisitor, Visitor};
use lib::xml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::xml::visitors::position_visitor::PositionVisitor;
use lib::xml::walker::Walker;
use roxmltree::{Document, ParsingOptions};
use std::fs;
use std::fs::read_to_string;
use std::time::Instant;
use ttf_parser::Face;

/// Concrete output the `render` command should produce. There is deliberately
/// no default and no inference from the `--out` file extension: the caller must
/// say `--format svg` or `--format pdf` explicitly.
#[derive(Clone, Copy, Debug, clap::ValueEnum)]
pub enum OutputFormat {
    Svg,
    Pdf,
}

#[derive(Args, Debug)]
pub struct RenderArgs {
    #[arg(long)]
    file: String,

    #[arg(long)]
    out: String,

    /// Output format. Required, no default -- pass `svg` or `pdf`.
    #[arg(long, value_enum)]
    format: OutputFormat,

    #[arg(long)]
    meta: String,

    #[arg(long)]
    glyphs: String,

    /// Font family for titles / work-level text. Defaults to the app default
    /// (`serif`). For `--format pdf` this family is resolved against the
    /// installed system fonts and embedded.
    #[arg(long)]
    title_font: Option<String>,

    /// Font family for lyrics. Defaults to the app default (`serif`). Resolved
    /// and embedded like `--title-font` for `--format pdf`.
    #[arg(long)]
    lyric_font: Option<String>,

    #[arg(long, short, action)]
    debug: bool,

    #[arg(long)]
    page_color: Option<Color>,

    #[arg(long)]
    foreground_color: Option<Color>,

    #[arg(long)]
    page_orientation: Option<PageOrientation>,

    #[arg(long)]
    horizontal_gutter_even: Option<f32>,

    #[arg(long)]
    horizontal_gutter_uneven: Option<f32>,

    #[arg(long)]
    vertical_gutter: Option<f32>,
}

pub fn run(args: RenderArgs) {
    let RenderArgs {
        file,
        out,
        format,
        meta,
        glyphs: glyph_names,
        title_font,
        lyric_font,
        debug,
        page_color,
        foreground_color,
        page_orientation,
        horizontal_gutter_even,
        horizontal_gutter_uneven,
        vertical_gutter,
    } = args;

    let mut time = Instant::now();

    let data = read_to_string(&file).unwrap_or_else(|err| panic!("Failed to read '{file}': {err}"));
    let meta_content = read_to_string(&meta)
        .unwrap_or_else(|err| panic!("Failed to read metadata '{meta}': {err}"));
    let glyph_names_content = read_to_string(&glyph_names)
        .unwrap_or_else(|err| panic!("Failed to read glyph names '{glyph_names}': {err}"));
    let font = SmuflFont::load(&meta_content, &glyph_names_content);

    println!("Reading to string: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(&data, options).unwrap();

    println!("Parsing doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let mut validation_ctx = ValidationCtx::default();
    let visitor = DefaultVisitor {}
        .uses(PartConsistencyVisitor::default())
        .uses(PositionVisitor::default());
    Walker::new(visitor).walk(&document, &mut validation_ctx);
    print_issues(&document, &validation_ctx.issues);

    println!("Validation: {}ms", time.elapsed().as_millis());

    let user_layout = UserLayout {
        page_color,
        foreground_color,
        page_orientation,
        horizontal_gutter_even,
        horizontal_gutter_uneven,
        vertical_gutter,
        ..Default::default()
    };
    let app_defaults: AppDefaults = Default::default();

    let EngravedScore {
        score: visual,
        layout,
    } = engrave(
        &document,
        &font,
        &user_layout,
        &app_defaults,
        &mut |stage| match stage {
            Stage::FirstPass(d) => println!(
                "First read pass: walking doc tree for layout: {}ms",
                d.as_millis()
            ),
            Stage::SecondPass(d) => println!(
                "Second read pass: walking doc tree for content: {}ms",
                d.as_millis()
            ),
            Stage::ApplyLayout(d) => println!("Applying user layout: {}ms", d.as_millis()),
            Stage::Rebeam(d) => println!("Rebeaming: {}ms", d.as_millis()),
            Stage::LayoutPass(d) => println!("Layout pass: {}ms", d.as_millis()),
        },
    );

    let title_font = title_font.as_deref().unwrap_or(&app_defaults.title_font);
    let lyric_font = lyric_font.as_deref().unwrap_or(&app_defaults.lyric_font);

    match format {
        OutputFormat::Svg => write_svg(&visual, &font, title_font, lyric_font, debug, &out),
        OutputFormat::Pdf => write_pdf(
            &visual,
            &font,
            title_font,
            lyric_font,
            debug,
            &layout.defaults,
            &out,
        ),
    }

    println!("Written to: {}", out)
}

/// Renders `score` to a single SVG document and writes it to `out`.
fn write_svg(
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

/// Renders `score` to a multi-page PDF -- one physical page per laid-out page --
/// with every font it draws text in embedded once, and writes it to `out`.
fn write_pdf(
    score: &Score,
    font: &SmuflFont,
    title_font: &str,
    lyric_font: &str,
    debug: bool,
    defaults: &Defaults,
    out: &str,
) {
    let fonts = RenderFonts::create(font, title_font, lyric_font);
    let music_family = fonts.music.family;
    let title_family = fonts.title.family;
    let lyric_family = fonts.lyric.family;

    let mut db = fontdb::Database::new();
    db.load_system_fonts();

    // Own the bytes here so the parsed `Face`s and the slices `write_pdf` embeds
    // can borrow them for the rest of this function.
    let music_bytes = load_family_bytes(&db, music_family).unwrap_or_else(|| {
        panic!("music font '{music_family}' not found in the system fonts; install it")
    });
    let title_bytes = load_family_bytes(&db, title_family);
    let lyric_bytes = load_family_bytes(&db, lyric_family);

    let mut embedded = vec![embedded_font(music_family, &music_bytes)];
    if title_family != music_family
        && let Some(bytes) = &title_bytes
    {
        embedded.push(embedded_font(title_family, bytes));
    }
    if lyric_family != music_family
        && lyric_family != title_family
        && let Some(bytes) = &lyric_bytes
    {
        embedded.push(embedded_font(lyric_family, bytes));
    }
    // The music font is entry 0 and the fallback for any family that didn't
    // resolve (e.g. a lyric font that isn't installed).
    let font_set = FontSet::new(embedded, 0);

    // MusicXML tenths -> PDF points: the score's mm-per-tenth scaling times 72
    // points per inch over 25.4 mm per inch.
    let pt_per_tenth = defaults.scaling_millimeters / defaults.scaling_tenths * 72.0 / 25.4;

    let mut time = Instant::now();

    let mut pages = RenderCompositor::base().walk_pages(score, &fonts);

    if debug {
        let overlay = RenderCompositor::debug().walk_pages(score, &fonts);
        for (page, debug_page) in pages.iter_mut().zip(overlay) {
            page.elements.extend(debug_page.elements);
        }
    }

    println!("Render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let pdf_pages: Vec<PdfPage> = pages
        .iter()
        .map(|page| {
            let canvas = PdfPageCanvas::new(
                page.origin,
                (page.width, page.height),
                pt_per_tenth,
                &font_set,
            );
            CanvasPainter::new(canvas).paint(&page.elements)
        })
        .collect();

    fs::write(out, pdf::write_pdf(&pdf_pages, &font_set)).unwrap();

    println!("Write to pdf: {}ms", time.elapsed().as_millis());
}

/// Copies out the raw bytes of the installed font matching `family` at regular
/// weight / upright style, or `None` if nothing matches. The CSS generic
/// keywords resolve through fontdb's generic families rather than as literal
/// names.
fn load_family_bytes(db: &fontdb::Database, family: &str) -> Option<Vec<u8>> {
    let family = match family {
        "serif" => fontdb::Family::Serif,
        "sans-serif" => fontdb::Family::SansSerif,
        "monospace" => fontdb::Family::Monospace,
        "cursive" => fontdb::Family::Cursive,
        "fantasy" => fontdb::Family::Fantasy,
        name => fontdb::Family::Name(name),
    };
    let id = db.query(&fontdb::Query {
        families: &[family],
        weight: fontdb::Weight::NORMAL,
        stretch: fontdb::Stretch::Normal,
        style: fontdb::Style::Normal,
    })?;
    db.with_face_data(id, |data, _index| data.to_vec())
}

/// Builds an [`EmbeddedFont`] borrowing `bytes`, deriving the PDF `BaseFont`
/// name from the font's own PostScript name (falling back to `family`).
fn embedded_font<'f>(family: &str, bytes: &'f [u8]) -> EmbeddedFont<'f> {
    let face = Face::parse(bytes, 0)
        .unwrap_or_else(|err| panic!("font '{family}' is not valid OpenType: {err}"));
    let base_font = face
        .names()
        .into_iter()
        .find(|n| n.name_id == ttf_parser::name_id::POST_SCRIPT_NAME)
        .and_then(|n| n.to_string())
        .unwrap_or_else(|| family.replace(' ', ""));

    EmbeddedFont {
        family: family.to_string(),
        weight: FontWeight::Normal,
        style: FontStyle::Normal,
        base_font,
        face,
        program: bytes,
    }
}
