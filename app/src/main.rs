use clap::Parser;
use lib::drawable::bfs_iter::bfs_elements;
use lib::drawable::element::{Element, to_svg};
use lib::layout::{Layout, UserLayout};
use lib::layout_ctx::LayoutCtx;
use lib::visitor::{DefaultVisitor, Visitor};
use lib::visitors::content_visitor::ContentVisitor;
use lib::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::visitors::layout_visitor::LayoutVisitor;
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

    let time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");

    let _elements = run(data);

    let svg = to_svg(&_elements);

    fs::write("./svg.svg", svg).unwrap();

    let elapsed_time = time.elapsed();
    println!("Elapsed: {}ms", elapsed_time.as_millis())
}

pub fn run(xml_string: String) -> Vec<Element> {
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    let document = Document::parse_with_options(&xml_string, options).unwrap();

    let mut layout_ctx = LayoutCtx::default();
    let mut user_layout = UserLayout::default();
    let mut layout = Layout::default();
    let mut visual = Score::default();

    let visitor = DefaultVisitor {}
        .add_callback(LayoutVisitor {})
        .add_callback(LayoutContextVisitor {})
        .add_callback(ContentVisitor {
            staff_measures: Default::default(),
        });

    let mut ctx = WalkerCtx::new(&mut user_layout, &mut layout, &mut layout_ctx, &mut visual);

    Walker::new(visitor).walk(&document, &mut ctx);

    visual.measure(&XY::INFINITE);
    visual.arrange(&XY::ZERO);

    let elements: Vec<Element> = bfs_elements(&visual).collect();
    elements
}
