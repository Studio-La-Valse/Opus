use clap::Parser;
use opus::score::visual::visual_score::VisualScore;
use opus::xml::visitor::DefaultVisitor;
use opus::xml::visitors::content_visitor::ContentVisitor;
use opus::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use opus::xml::visitors::layout_visitor::LayoutVisitor;
use opus::xml::walker_ctx::WalkerCtx;
use opus::*;
use roxmltree::Document;
use roxmltree::ParsingOptions;
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

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    let time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");
    let document = Document::parse_with_options(&data, options).unwrap();

    let mut layout_ctx = LayoutCtx::default();
    let mut user_layout = UserLayout::default();
    let mut layout = Layout::default();
    let mut visual = VisualScore {};

    let visitor = DefaultVisitor {}
        .add_callback(LayoutVisitor {})
        .add_callback(LayoutContextVisitor {})
        .add_callback(ContentVisitor {});

    let mut ctx = WalkerCtx::new(&mut user_layout, &mut layout, &mut layout_ctx, &mut visual);

    Walker::new(visitor, &mut ctx).walk(&document);

    let elapsed_time = time.elapsed();
    println!("Elapsed: {}ms", elapsed_time.as_millis())
}
