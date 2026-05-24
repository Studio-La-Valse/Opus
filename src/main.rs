use clap::Parser;
use std::fs::read_to_string;
use roxmltree::ParsingOptions;
use roxmltree::Document;
use opus::score::layout::{UserLayout, Layout};
use opus::xml::layout_ctx::LayoutCtx;
use opus::xml::visitor::{DefaultVisitor, Visitor};
use opus::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use opus::xml::visitors::layout_visitor::LayoutVisitor;
use opus::xml::walker::Walker;

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

    let data = read_to_string(file).expect("Something went wrong reading the file");

    let document = Document::parse_with_options(&data, options).unwrap();

    let mut layout_ctx = LayoutCtx::default();
    let layout_ctx_visitor = LayoutContextVisitor { layout_ctx: &mut layout_ctx };

    let mut user_layout = UserLayout::default();
    let mut layout = Layout::default();
    let layout_visitor = LayoutVisitor { layout: &mut layout, user_layout: &mut user_layout };

    let visitor = DefaultVisitor { }
        .add_callback(layout_ctx_visitor)
        .add_callback(layout_visitor);
    
    Walker { visitor }
        .walk(&document);
}
