use crate::commands::{print_issues, read_musicxml};
use clap::{Args, Subcommand};
use lib::musicxml::validate::ValidationCtx;
use lib::musicxml::visitor::{DefaultVisitor, Visitor};
use lib::musicxml::visitors::validators::beam_group_visitor::BeamGroupVisitor;
use lib::musicxml::visitors::validators::group_symbol_visitor::GroupSymbolVisitor;
use lib::musicxml::visitors::validators::page_layout_visitor::PageLayoutVisitor;
use lib::musicxml::visitors::validators::part_consistency_visitor::PartConsistencyVisitor;
use lib::musicxml::visitors::validators::position_visitor::PositionVisitor;
use lib::musicxml::visitors::validators::staff_details_visitor::StaffDetailsVisitor;
use lib::musicxml::walker::Walker;
use lib::score::engrave::{EngravedScore, Stage, engrave};
use lib::score::layout_options::{APP_DEFAULTS, UserLayout};
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::smufl::smufl_font::SmuflFont;
use roxmltree::{Document, ParsingOptions};
use std::fs::read_to_string;
use std::time::Instant;

use self::layout_args::LayoutArgs;
use self::paths::OutputTarget;

pub mod layout_args;
pub mod paths;
mod pdf;
mod svg;

#[derive(Args, Debug)]
pub struct RenderArgs {
    #[arg(long)]
    file: String,

    /// Output directory for the rendered file(s). Created if it doesn't exist.
    /// When omitted, output is written next to `--file`. Filenames reuse the
    /// input file's stem: `render pdf` writes `<stem>.pdf`; `render svg` writes
    /// one file per page, `<stem>-p1.svg`, `<stem>-p2.svg`, ...
    #[arg(long)]
    out: Option<String>,

    /// Replace output files that already exist. Without this, `render` panics
    /// rather than overwrite a file (guarding against a stray `--out` or the
    /// write-beside-the-input default clobbering something unrelated).
    #[arg(long, action)]
    overwrite: bool,

    #[arg(long)]
    meta: String,

    #[arg(long)]
    glyphs: String,

    #[arg(long, short, action)]
    debug: bool,

    #[command(flatten)]
    layout: LayoutArgs,
}

/// Output format for `opus render`, chosen as a subcommand: `render svg` or
/// `render pdf`. There is deliberately no default and no inference from any
/// file extension.
///
/// Both formats are driven the same way -- engrave once, walk the score into
/// per-page [`RenderedPage`](lib::score::visual::render_compositor::RenderedPage)s,
/// then hand those to the format writer. They differ only in output shape: SVG
/// emits one self-contained file per page, PDF a single multi-page document.
///
/// Each variant carries its own [`RenderArgs`] rather than sharing one set via
/// `#[command(flatten)]` so that svg-only or pdf-only options can be added later
/// without disturbing the other subcommand.
#[derive(Subcommand, Debug)]
pub enum RenderCommand {
    /// Render to SVG, one file per laid-out page (`<stem>-p1.svg`, ...).
    Svg(RenderArgs),
    /// Render to a single multi-page PDF, one physical page per laid-out page,
    /// with every font it draws text in embedded once.
    Pdf(RenderArgs),
}

pub fn run(format: RenderCommand) {
    let (args, is_pdf) = match format {
        RenderCommand::Svg(args) => (args, false),
        RenderCommand::Pdf(args) => (args, true),
    };

    let RenderArgs {
        file,
        out,
        overwrite,
        meta,
        glyphs: glyph_names,
        debug,
        layout: layout_args,
    } = args;

    let mut time = Instant::now();

    let data = read_musicxml(&file);
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
        .uses(PositionVisitor::default())
        .uses(BeamGroupVisitor::default())
        .uses(PageLayoutVisitor::default())
        .uses(StaffDetailsVisitor::default())
        .uses(GroupSymbolVisitor::default());
    Walker::new(visitor).walk(&document, &mut validation_ctx);
    print_issues(&document, &validation_ctx.issues);

    println!("Validation: {}ms", time.elapsed().as_millis());

    let user_layout = UserLayout::from(layout_args);

    // The pipeline itself does no timing -- `std::time::Instant` is
    // unimplemented on wasm32, so `lib` stays clock-free and the caller that
    // wants durations measures the gaps between stage callbacks.
    let mut stage_time = Instant::now();

    let EngravedScore {
        score: visual,
        layout,
        messages,
    } = engrave(&document, &font, &user_layout, &mut |stage| {
        let elapsed = stage_time.elapsed().as_millis();
        stage_time = Instant::now();

        match stage {
            Stage::FirstPass => {
                println!("First read pass: walking doc tree for layout: {elapsed}ms")
            }
            Stage::SecondPass => {
                println!("Second read pass: walking doc tree for content: {elapsed}ms")
            }
            Stage::Rebeam => println!("Rebeaming: {elapsed}ms"),
            Stage::ResolveLayout => println!("Resolving layout: {elapsed}ms"),
            Stage::LayoutPass => println!("Layout pass: {elapsed}ms"),
        }
    });

    // Guarded because `print_issues` announces an empty list as "no issues
    // found", which would read as a validation verdict rather than as the walk
    // simply having had nothing to say.
    if !messages.is_empty() {
        print_issues(&document, &messages);
    }

    let title_font = user_layout
        .title
        .font
        .as_deref()
        .unwrap_or(APP_DEFAULTS.title.font);
    let lyric_font = user_layout
        .lyric
        .font
        .as_deref()
        .unwrap_or(APP_DEFAULTS.lyric.font);
    let group_name_font = user_layout
        .group_name
        .font
        .as_deref()
        .unwrap_or(APP_DEFAULTS.group_name.font);

    let time = Instant::now();

    // One walk feeds every format: the score's pages, each carrying its
    // elements in page-local tenths. `fonts` must outlive `pages`, which
    // borrows glyph data from it.
    let fonts = RenderFonts::create(&font, title_font, lyric_font, group_name_font);
    let pages = RenderCompositor::compose(&visual, &fonts, debug);

    println!("Render pass: {}ms", time.elapsed().as_millis());

    let target = OutputTarget::resolve(out.as_deref(), &file, overwrite);

    if is_pdf {
        pdf::write(&pages, &fonts, &layout.defaults, &target);
    } else {
        svg::write(&pages, &target);
    }
}
