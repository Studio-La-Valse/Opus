use crate::commands::{print_issues, read_musicxml};
use crate::smufl_fonts::{SmuflRoots, discover};
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
use lib::score::engrave::{Stage, arrange_score, walk_document};
use lib::score::layout_options::UserLayout;
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::smufl::font_choice::choose_music_font;
use lib::smufl::smufl_font::SmuflFont;
use roxmltree::{Document, ParsingOptions};
use std::fs::read_to_string;
use std::time::Instant;

use self::layout_args::LayoutArgs;
use self::paths::OutputTarget;

pub mod layout_args;
pub mod layout_file;
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

    /// The SMuFL music font to engrave in, by name (`Bravura`, `Leland`, ...).
    /// Overrides the document's `<music-font>`; without either, Bravura. The
    /// font has to be installed -- see `opus font list` and
    /// `opus font install`.
    #[arg(long)]
    music_font: Option<String>,

    #[arg(long, short, action)]
    debug: bool,

    /// A TOML layout file: one table per option group, e.g. `[tie]` with
    /// `height_max = 14`. Optional; every option it leaves out falls back to
    /// the document, then the SMuFL font's engravingDefaults, then the app
    /// default. The layout flags below override it, option by option.
    #[arg(long = "layout")]
    layout_file: Option<String>,

    #[command(flatten)]
    layout_args: LayoutArgs,
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
        music_font,
        debug,
        layout_file,
        layout_args,
    } = args;

    let mut time = Instant::now();

    let data = read_musicxml(&file);

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

    // The flags are the more specific of the two, so they win option by option.
    let file_layout = layout_file.as_deref().map(layout_file::read);
    let user_layout = file_layout
        .unwrap_or_default()
        .overlay(UserLayout::from(layout_args));

    // The pipeline itself does no timing -- `std::time::Instant` is
    // unimplemented on wasm32, so `lib` stays clock-free and the caller that
    // wants durations measures the gaps between stage callbacks.
    let mut stage_time = Instant::now();

    let mut progress = |stage| {
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
    };

    // The walk does not need the font, so the document's own `<music-font>`
    // can weigh in on which one to load before anything is arranged.
    let (mut visual, layout, messages) = walk_document(&document, &mut progress);
    let font = load_music_font(music_font.as_deref(), &layout.music_font);
    arrange_score(&mut visual, &layout, &font, &user_layout, &mut progress);

    // Guarded because `print_issues` announces an empty list as "no issues
    // found", which would read as a validation verdict rather than as the walk
    // simply having had nothing to say.
    if !messages.is_empty() {
        print_issues(&document, &messages);
    }

    let time = Instant::now();

    // One walk feeds every format: the score's pages, each carrying its
    // elements in page-local tenths. `fonts` must outlive `pages`, which
    // borrows glyph data from it.
    let fonts = RenderFonts::resolve(&font, &user_layout);
    let pages = RenderCompositor::compose(&visual, &fonts, debug);

    println!("Render pass: {}ms", time.elapsed().as_millis());

    let target = OutputTarget::resolve(out.as_deref(), &file, overwrite);

    if is_pdf {
        pdf::write(&pages, &fonts, &layout.defaults, &target);
    } else {
        svg::write(&pages, &target);
    }
}

/// Finds the installed SMuFL fonts, picks one by `user` → `document` →
/// Bravura, and loads its metadata.
fn load_music_font(user: Option<&str>, document: &[String]) -> SmuflFont {
    let installed = discover(&SmuflRoots::for_this_system());
    let names: Vec<String> = installed.iter().map(|font| font.name.clone()).collect();

    let name = choose_music_font(user, document, &names).unwrap_or_else(|err| {
        panic!("{err}; see `opus font list`, or install one with `opus font install <folder>`")
    });
    let font = installed.iter().find(|font| font.name == name).unwrap();

    println!("Music font: {name} ({})", font.metadata.display());

    let json = read_to_string(&font.metadata).unwrap_or_else(|err| {
        panic!(
            "Failed to read metadata '{}': {err}",
            font.metadata.display()
        )
    });
    SmuflFont::load(&json)
}
