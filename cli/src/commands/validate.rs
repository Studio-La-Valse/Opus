use crate::commands::print_issues;
use clap::Args;
use lib::xml::validate::ValidationCtx;
use lib::xml::validation_issue::{Severity, ValidationIssue};
use lib::xml::visitor::{DefaultVisitor, Visitor};
use lib::xml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::xml::visitors::position_visitor::PositionVisitor;
use lib::xml::walker::Walker;
use roxmltree::{Document, ParsingOptions};
use std::fs::read_to_string;

#[derive(Args, Debug)]
pub struct ValidateArgs {
    #[arg(long)]
    file: String,
}

pub fn run(args: ValidateArgs) {
    let data = read_to_string(args.file).expect("Something went wrong reading the file");

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(&data, options).unwrap();

    let mut ctx = ValidationCtx::default();

    let root = document.root_element();
    if root.tag_name().name() != "score-partwise" {
        ctx.issues.push(ValidationIssue {
            severity: Severity::Error,
            message: format!(
                "Expected root element <score-partwise>, found <{}>",
                root.tag_name().name()
            ),
            at: root.range().start,
        });
    } else {
        let visitor = DefaultVisitor {}
            .uses(PartConsistencyVisitor::default())
            .uses(PositionVisitor::default());

        Walker::new(visitor).walk(&document, &mut ctx);
    }

    print_issues(&document, &ctx.issues);
}
