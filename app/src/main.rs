use clap::Parser;
use lib::color::Color;
use lib::drawable::bfs_iter::bfs_elements;
use lib::drawable::element::{Element, to_svg};
use lib::layout::{Layout, UserLayout};
use lib::layout_ctx::LayoutCtx;
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
}

fn main() {
    let args = Args::parse();
    let file = args.file;

    let mut time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");

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
        .add_callback(ContentVisitor {
            staff_measures: Default::default(),
        });

    let mut ctx = WalkerCtx::new(&mut layout, &mut layout_ctx, &mut visual);

    Walker::new(visitor).walk(&document, &mut ctx);

    println!("Walking doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let user_layout = UserLayout {
        page_color: Some(Color {
            a: 1.,
            r: 255,
            g: 200,
            b: 100,
        }),

        foreground_color: Some(Color {
            a: 1.,
            r: 200,
            g: 100,
            b: 150,
        }),

        .. UserLayout::default()
    };
    visual.apply_layout(&layout, &user_layout);

    println!("Applying user layout: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    visual.measure(&XY::INFINITE);
    visual.arrange(&XY::ZERO);

    println!("Layout pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let elements: Vec<Element> = bfs_elements(&visual).collect();

    let svg = to_svg(&elements);
    fs::write("./svg.svg", svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
