pub mod render;
pub mod validate;

use lib::musicxml::validation_issue::{Severity, ValidationIssue};
use roxmltree::Document;

/// Reads a MusicXML file into a `String`, honouring a byte-order mark.
///
/// `fs::read_to_string` would do for the common case, but UTF-16 is a perfectly
/// ordinary MusicXML encoding -- Finale writes it, and two of the bundled
/// samples are in it -- and that rejects the file outright as "not valid UTF-8".
/// `roxmltree` parses a `&str`, so the transcode has to happen here rather than
/// in the parser.
///
/// The BOM is the only signal used. The `encoding="..."` pseudo-attribute in the
/// XML declaration is deliberately ignored: by the time it can be read the bytes
/// have already had to be decoded, and a UTF-16 document without a BOM is
/// outside what this is trying to be.
pub fn read_musicxml(path: &str) -> String {
    let bytes = std::fs::read(path).unwrap_or_else(|err| panic!("Failed to read '{path}': {err}"));

    match bytes.as_slice() {
        [0xFF, 0xFE, rest @ ..] => decode_utf16(path, rest, u16::from_le_bytes),
        [0xFE, 0xFF, rest @ ..] => decode_utf16(path, rest, u16::from_be_bytes),
        [0xEF, 0xBB, 0xBF, rest @ ..] => decode_utf8(path, rest),
        _ => decode_utf8(path, &bytes),
    }
}

fn decode_utf8(path: &str, bytes: &[u8]) -> String {
    String::from_utf8(bytes.to_vec())
        .unwrap_or_else(|err| panic!("Failed to read '{path}': not valid UTF-8: {err}"))
}

fn decode_utf16(path: &str, bytes: &[u8], to_unit: fn([u8; 2]) -> u16) -> String {
    if !bytes.len().is_multiple_of(2) {
        panic!("Failed to read '{path}': UTF-16 content has an odd number of bytes");
    }

    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| to_unit([pair[0], pair[1]]))
        .collect();

    String::from_utf16(&units)
        .unwrap_or_else(|err| panic!("Failed to read '{path}': not valid UTF-16: {err}"))
}

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
