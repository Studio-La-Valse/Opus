use crate::commands::print_issues;
use clap::{Args, Subcommand};
use lib::geometry::color::Color;
use lib::musicxml::validate::ValidationCtx;
use lib::musicxml::visitor::{DefaultVisitor, Visitor};
use lib::musicxml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::musicxml::visitors::position_visitor::PositionVisitor;
use lib::musicxml::walker::Walker;
use lib::score::app_defaults::AppDefaults;
use lib::score::engrave::{EngravedScore, Stage, engrave};
use lib::score::page_orientation::PageOrientation;
use lib::score::user_layout::UserLayout;
use lib::smufl::smufl_font::SmuflFont;
use roxmltree::{Document, ParsingOptions};
use std::fs::read_to_string;
use std::time::Instant;

mod pdf;
mod svg;

#[derive(Args, Debug)]
pub struct RenderArgs {
    #[arg(long)]
    file: String,

    #[arg(long)]
    out: String,

    #[arg(long)]
    meta: String,

    #[arg(long)]
    glyphs: String,

    /// Font family for titles / work-level text. Defaults to the app default
    /// (`serif`). For `render pdf` this family is resolved against the installed
    /// system fonts and embedded.
    #[arg(long)]
    title_font: Option<String>,

    /// Font family for lyrics. Defaults to the app default (`serif`). Resolved
    /// and embedded like `--title-font` for `render pdf`.
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

/// Output format for `opus render`, chosen as a subcommand: `render svg` or
/// `render pdf`. There is deliberately no default and no inference from the
/// `--out` file extension.
///
/// Each variant carries its own [`RenderArgs`] rather than sharing one set via
/// `#[command(flatten)]` so that svg-only or pdf-only options can be added later
/// without disturbing the other subcommand.
#[derive(Subcommand, Debug)]
pub enum RenderCommand {
    /// Render to a single SVG document.
    Svg(RenderArgs),
    /// Render to a multi-page PDF, one physical page per laid-out page, with
    /// every font it draws text in embedded once.
    Pdf(RenderArgs),
}

pub fn run(format: RenderCommand) {
    let (args, pdf) = match format {
        RenderCommand::Svg(args) => (args, false),
        RenderCommand::Pdf(args) => (args, true),
    };

    let RenderArgs {
        file,
        out,
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
            Stage::Rebeam(d) => println!("Rebeaming: {}ms", d.as_millis()),
            Stage::LayoutPass(d) => println!("Layout pass: {}ms", d.as_millis()),
        },
    );

    let title_font = title_font.as_deref().unwrap_or(&app_defaults.title_font);
    let lyric_font = lyric_font.as_deref().unwrap_or(&app_defaults.lyric_font);

    if pdf {
        pdf::write(
            &visual,
            &font,
            title_font,
            lyric_font,
            debug,
            &layout.defaults,
            &out,
        );
    } else {
        svg::write(&visual, &font, title_font, lyric_font, debug, &out);
    }

    println!("Written to: {}", out)
}
