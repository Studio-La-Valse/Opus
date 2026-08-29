use crate::commands::print_issues;
use clap::Args;
use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::pdf::{self, PdfPage, PdfPageCanvas};
use lib::drawable::canvas::svg::SvgCanvas;
use lib::drawable::drawable_element::DrawableElement;
use lib::drawable::layoutable::Layoutable;
use lib::geometry::color::Color;
use lib::geometry::xy::XY;
use lib::score::app_defaults::AppDefaults;
use lib::score::layout::{Defaults, Layout};
use lib::score::layout_ctx::LayoutCtx;
use lib::score::page_orientation::PageOrientation;
use lib::score::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use lib::score::user_layout::UserLayout;
use lib::score::visual::layout_engine::{HorizontalPageLayout, LayoutEngine, VerticalPageLayout};
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_pass::{BaseRenderer, DebugRenderer};
use lib::score::visual::score::Score;
use lib::score::visual::score_element::ScoreElement;
use lib::smufl::smufl_font::SmuflFont;
use lib::xml::validate::ValidationCtx;
use lib::xml::visitor::{DefaultVisitor, Visitor};
use lib::xml::visitors::content_visitor::ContentVisitor;
use lib::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::xml::visitors::layout_visitor::LayoutVisitor;
use lib::xml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::xml::visitors::position_visitor::PositionVisitor;
use lib::xml::visitors::setup_visitor::SetupVisitor;
use lib::xml::walker::Walker;
use lib::xml::walker_ctx::WalkerCtx;
use roxmltree::{Document, ParsingOptions};
use std::collections::{HashMap, HashSet};
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

    /// Path to the SMuFL music font (e.g. `Bravura.otf`). Required for
    /// `--format pdf`, where it is embedded into the document; unused for SVG.
    #[arg(long)]
    font: Option<String>,

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
    let file = args.file;
    let out = args.out;
    let format = args.format;
    let meta = args.meta;
    let glyph_names = args.glyphs;
    let font_path = args.font;
    let debug = args.debug;
    let page_color = args.page_color;
    let foreground_color = args.foreground_color;
    let page_orientation = args.page_orientation;
    let horizontal_gutter_even = args.horizontal_gutter_even;
    let horizontal_gutter_uneven = args.horizontal_gutter_uneven;
    let vertical_gutter = args.vertical_gutter;

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
    time = Instant::now();

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

    let mut layout_ctx = LayoutCtx::default();
    let mut layout = Layout::default();
    let mut visual = Score::default();

    let visitor = DefaultVisitor {}
        .uses(LayoutContextVisitor {})
        .uses(SetupVisitor {})
        .uses(LayoutVisitor {
            encountered: HashSet::new(),
        });

    let mut ctx = WalkerCtx::new(
        &user_layout,
        &mut layout,
        &app_defaults,
        &mut layout_ctx,
        &mut visual,
        &font,
    );

    Walker::new(visitor).walk(&document, &mut ctx);

    println!(
        "First read pass: walking doc tree for layout: {}ms",
        time.elapsed().as_millis()
    );
    time = Instant::now();

    let visitor = DefaultVisitor {}
        .uses(LayoutContextVisitor {})
        .uses(ContentVisitor {
            clef_change: HashMap::new(),
        });

    let mut ctx = WalkerCtx::new(
        &user_layout,
        &mut layout,
        &app_defaults,
        &mut layout_ctx,
        &mut visual,
        &font,
    );

    Walker::new(visitor).walk(&document, &mut ctx);

    println!(
        "Second read pass: walking doc tree for content: {}ms",
        time.elapsed().as_millis()
    );
    time = Instant::now();

    visual.apply_layout(&layout, &user_layout, &app_defaults);

    println!("Applying user layout: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let strat_impl = Box::new(SimpleRebeamStrategy {});
    let strategy = Box::new(OnlyWhenRequiredRebeamStrategy { imp: strat_impl });

    visual.rebeam(strategy.as_ref());

    println!("Rebeaming: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    visual.measure(&XY::INFINITE);

    let orientation = user_layout
        .page_orientation
        .unwrap_or(app_defaults.page_orientation);
    let layout_engine: Box<dyn LayoutEngine> = match orientation {
        PageOrientation::Horizontal => Box::new(HorizontalPageLayout {
            gutter_even: user_layout
                .horizontal_gutter_even
                .unwrap_or(app_defaults.horizontal_gutter_even),
            gutter_uneven: user_layout
                .horizontal_gutter_uneven
                .unwrap_or(app_defaults.horizontal_gutter_uneven),
        }),
        PageOrientation::Vertical => Box::new(VerticalPageLayout {
            gutter: user_layout
                .vertical_gutter
                .unwrap_or(app_defaults.vertical_gutter),
        }),
    };
    layout_engine.arrange_pages(&mut visual, &XY::ZERO);

    println!("Layout pass: {}ms", time.elapsed().as_millis());

    match format {
        OutputFormat::Svg => write_svg(&visual, &font, debug, &out),
        OutputFormat::Pdf => write_pdf(&visual, &font, debug, font_path, &layout.defaults, &out),
    }

    println!("Written to: {}", out)
}

/// Renders `score` to a single SVG document and writes it to `out`.
fn write_svg(score: &Score, font: &SmuflFont, debug: bool, out: &str) {
    let mut time = Instant::now();

    let mut elements: Vec<DrawableElement<'_>> = RenderCompositor {
        pass: Box::new(BaseRenderer {}),
    }
    .walk(score, font);

    if debug {
        elements.extend(
            RenderCompositor {
                pass: Box::new(DebugRenderer {}),
            }
            .walk(score, font),
        );
    }

    println!("Render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let svg = CanvasPainter::new(SvgCanvas::new()).paint(&elements);
    fs::write(out, svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}

/// Renders `score` to a multi-page PDF -- one physical page per laid-out page --
/// with the SMuFL music font embedded, and writes it to `out`.
fn write_pdf(
    score: &Score,
    font: &SmuflFont,
    debug: bool,
    font_path: Option<String>,
    defaults: &Defaults,
    out: &str,
) {
    let font_path = font_path.expect("--format pdf requires --font <path to the SMuFL music font>");
    let font_otf = fs::read(&font_path)
        .unwrap_or_else(|err| panic!("Failed to read font '{font_path}': {err}"));
    let face = Face::parse(&font_otf, 0)
        .unwrap_or_else(|err| panic!("Font '{font_path}' is not valid OpenType: {err}"));

    // MusicXML tenths -> PDF points: the score's mm-per-tenth scaling times 72
    // points per inch over 25.4 mm per inch.
    let pt_per_tenth = defaults.scaling_millimeters / defaults.scaling_tenths * 72.0 / 25.4;

    let mut time = Instant::now();

    let mut pages = RenderCompositor {
        pass: Box::new(BaseRenderer {}),
    }
    .walk_pages(score, font);

    if debug {
        let overlay = RenderCompositor {
            pass: Box::new(DebugRenderer {}),
        }
        .walk_pages(score, font);
        for (page, debug_page) in pages.iter_mut().zip(overlay) {
            page.elements.extend(debug_page.elements);
        }
    }

    println!("Render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let pdf_pages: Vec<PdfPage> = pages
        .iter()
        .map(|page| {
            let canvas =
                PdfPageCanvas::new(page.origin, (page.width, page.height), pt_per_tenth, &face);
            CanvasPainter::new(canvas).paint(&page.elements)
        })
        .collect();

    fs::write(out, pdf::write_pdf(&pdf_pages, &font_otf)).unwrap();

    println!("Write to pdf: {}ms", time.elapsed().as_millis());
}
