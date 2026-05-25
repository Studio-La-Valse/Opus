use clap::Parser;
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
    let layout_ctx_visitor = LayoutContextVisitor {
        layout_ctx: &mut layout_ctx,
    };

    let mut user_layout = UserLayout::default();
    let mut layout = Layout::default();
    let layout_visitor = LayoutVisitor {
        layout: &mut layout,
        user_layout: &mut user_layout,
    };

    let visitor = DefaultVisitor {}
        .add_callback(layout_ctx_visitor)
        .add_callback(layout_visitor);

    Walker { visitor }.walk(&document);

    let elapsed_time = time.elapsed();
    println!("Elapsed: {}ms", elapsed_time.as_millis())
}
