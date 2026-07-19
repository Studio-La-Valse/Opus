use clap::Parser;
use lib::app_defaults::AppDefaults;
use lib::drawable::bfs_iter::bfs_elements;
use lib::drawable::drawable_element::{DrawableElement, scale_elem, to_svg};
use lib::layout::Layout;
use lib::layout_ctx::LayoutCtx;
use lib::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use lib::smufl::smufl_font::SmuflFont;
use lib::user_layout::UserLayout;
use lib::visitor::{DefaultVisitor, Visitor};
use lib::visitors::content_visitor::ContentVisitor;
use lib::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::visitors::layout_visitor::LayoutVisitor;
use lib::visual::element::ScoreElement;
use lib::visual::layoutable::Layoutable;
use lib::visual::score::Score;
use lib::walker::Walker;
use lib::walker_ctx::WalkerCtx;
use lib::xy::XY;
use roxmltree::{Document, ParsingOptions};
use std::fs;
use std::fs::read_to_string;
use std::time::Instant;

#[derive(Parser, Debug)]
struct Args {
    #[arg(long)]
    file: String,

    #[arg(long)]
    out: String,

    #[arg(long)]
    font: String,

    #[arg(long)]
    meta: String,

    #[arg(long)]
    glyphs: String,
}

fn main() {
    let args = Args::parse();
    let file = args.file;
    let out = args.out;
    let font = args.font;
    let meta = args.meta;
    let glyph_names = args.glyphs;

    let mut time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");
    let font = SmuflFont::load(font, &meta, &glyph_names);

    println!("Reading to string: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    let document = Document::parse_with_options(&data, options).unwrap();

    println!("Parsing doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let mut layout_ctx = LayoutCtx::default();
    let mut layout = Layout::default();
    let mut visual = Score::default();

    let visitor = DefaultVisitor {}
        .add_callback(LayoutVisitor {})
        .add_callback(LayoutContextVisitor {})
        .add_callback(ContentVisitor { part_measure: None });

    let mut ctx = WalkerCtx::new(&mut layout, &mut layout_ctx, &mut visual, &font);

    Walker::new(visitor).walk(&document, &mut ctx);

    println!("Walking doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let user_layout = UserLayout::new();

    let app_defaults: AppDefaults = Default::default();

    visual.apply_layout(&layout, &user_layout, &app_defaults);

    println!("Applying user layout: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let strat_impl = Box::new(SimpleRebeamStrategy {});
    let strategy = Box::new(OnlyWhenRequiredRebeamStrategy { imp: strat_impl });

    visual.rebeam(strategy.as_ref());

    println!("Rebeaming: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    visual.measure(&XY::INFINITE);
    visual.arrange(&XY::ZERO);

    println!("Layout pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let elements: Vec<DrawableElement> = bfs_elements(&visual).collect();

    let svg = to_svg(&elements.iter().map(|e| scale_elem(e, 0.01)).collect());
    fs::write(out, svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
