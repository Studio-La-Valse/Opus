pub mod render;
pub mod validate;

use lib::xml::validation_issue::{Severity, ValidationIssue};
use roxmltree::Document;

pub fn print_issues(document: &Document, issues: &[ValidationIssue]) {
    if issues.is_empty() {
        println!("validate: no issues found");
        return;
    }

    for issue in issues {
        let pos = document.text_pos_at(issue.at);
        let tag = match issue.severity {
            Severity::Info => "INFO",
            Severity::Warning => "WARN",
            Severity::Error => "ERROR",
        };
        println!("[{tag}] {}:{}: {}", pos.row, pos.col, issue.message);
    }
}
