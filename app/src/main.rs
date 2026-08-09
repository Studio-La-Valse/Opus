use clap::Parser;
use lib::app_defaults::AppDefaults;
use lib::drawable::drawable_element::{DrawableElement, to_svg};
use lib::drawable::layoutable::Layoutable;
use lib::layout::Layout;
use lib::layout_ctx::LayoutCtx;
use lib::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use lib::smufl::smufl_font::SmuflFont;
use lib::user_layout::UserLayout;
use lib::visitor::{DefaultVisitor, Visitor};
use lib::visitors::content_visitor::ContentVisitor;
use lib::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::visitors::layout_visitor::LayoutVisitor;
use lib::visitors::setup_visitor::SetupVisitor;
use lib::visual::render_compositor::RenderCompositor;
use lib::visual::render_pass::{BaseRenderer, DebugRenderer};
use lib::visual::score::Score;
use lib::visual::score_element::ScoreElement;
use lib::walker::Walker;
use lib::walker_ctx::WalkerCtx;
use lib::xy::XY;
use roxmltree::{Document, ParsingOptions};
use std::collections::{HashMap, HashSet};
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
    meta: String,

    #[arg(long)]
    glyphs: String,

    #[arg(long, short, action)]
    debug: bool,
}

fn main() {
    println!(
        "Size of DrawableElement: {}",
        std::mem::size_of::<DrawableElement<'_>>()
    );

    let args = Args::parse();
    let file = args.file;
    let out = args.out;
    let meta = args.meta;
    let glyph_names = args.glyphs;
    let debug = args.debug;

    let mut time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");
    let font = SmuflFont::load(&meta, &glyph_names);

    println!("Reading to string: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    let document = Document::parse_with_options(&data, options).unwrap();

    println!("Parsing doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let user_layout: UserLayout = Default::default();
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
    visual.arrange(&XY::ZERO);

    println!("Layout pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let pass = BaseRenderer {};
    let compositor = RenderCompositor {
        pass: Box::new(pass),
    };
    let mut elements: Vec<DrawableElement<'_>> = compositor.walk(&visual, &font);

    println!("First render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    if debug {
        let pass = DebugRenderer {};
        let compositor = RenderCompositor {
            pass: Box::new(pass),
        };
        elements.extend(compositor.walk(&visual, &font));

        println!("Second render pass: {}ms", time.elapsed().as_millis());
        time = Instant::now();
    }

    let svg = to_svg(elements);
    fs::write(out, svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
