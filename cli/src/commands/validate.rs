use crate::commands::{print_issues, read_musicxml};
use clap::Args;
use lib::musicxml::validate::ValidationCtx;
use lib::musicxml::visitor::{DefaultVisitor, Visitor};
use lib::musicxml::visitors::beam_group_visitor::BeamGroupVisitor;
use lib::musicxml::visitors::group_symbol_visitor::GroupSymbolVisitor;
use lib::musicxml::visitors::page_layout_visitor::PageLayoutVisitor;
use lib::musicxml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::musicxml::visitors::position_visitor::PositionVisitor;
use lib::musicxml::visitors::staff_details_visitor::StaffDetailsVisitor;
use lib::musicxml::visitors::tie_orientation_visitor::TieOrientationVisitor;
use lib::musicxml::walker::Walker;
use roxmltree::{Document, ParsingOptions};

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long)]
    file: String,
}

pub fn run(args: ValidateArgs) {
    println!("Reading {}", args.file);
    let data = read_musicxml(&args.file);
    println!(
        "Read {} successfully ({} lines, {} bytes)",
        args.file,
        data.lines().count(),
        data.len()
    );

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(&data, options).unwrap();
    println!("Parsed {} successfully", args.file);

    let mut ctx = ValidationCtx::default();

    let visitor = DefaultVisitor {}
        .uses(PartConsistencyVisitor::default())
        .uses(PositionVisitor::default())
        .uses(BeamGroupVisitor::default())
        .uses(PageLayoutVisitor::default())
        .uses(StaffDetailsVisitor::default())
        .uses(GroupSymbolVisitor::default())
        .uses(TieOrientationVisitor::default());

    Walker::new(visitor).walk(&document, &mut ctx);

    print_issues(&document, &ctx.issues);
}
