//! `StaffDetailsVisitor`: the malformed `<staff-details>` a document walk
//! panics on, described in plain words first.
//!
//! Same bargain as `test_page_layout_validation`: the reader is strict because
//! a staff it cannot count lines for or size is a number it would have to
//! invent, and every cause reported here is checked to be a cause the walk
//! really does panic on.

#[cfg(test)]
mod tests {
    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::Severity;
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::validators::staff_details_visitor::StaffDetailsVisitor;
    use lib::musicxml::walker::Walker;
    use lib::score::engrave::walk_document;
    use lib::score::layout_options::UserLayout;
    use lib::smufl::smufl_font::SmuflFont;
    use roxmltree::Document;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| {
            SmuflFont::load(
                &asset("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json"),
                &asset("assets/smufl/metadata/glyphnames.json"),
            )
        })
    }

    /// Wraps a `<staff-details>` in the `<attributes>` of a one-note measure,
    /// the only place it may appear.
    fn in_attributes(staff_details: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef><sign>G</sign><line>2</line></clef>{staff_details}</attributes>\
             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type></note>\
             </measure></part></score-partwise>"
        )
    }

    fn errors(xml: &str) -> Vec<String> {
        let document = Document::parse(xml).expect("test document does not parse");

        let mut ctx = ValidationCtx::default();
        Walker::new(DefaultVisitor {}.uses(StaffDetailsVisitor::default()))
            .walk(&document, &mut ctx);

        ctx.issues
            .iter()
            .inspect(|issue| {
                assert_eq!(
                    issue.severity,
                    Severity::Error,
                    "a staff cannot be sized by guesswork, so nothing here is a warning"
                );
            })
            .map(|issue| issue.message.clone())
            .collect()
    }

    #[test]
    fn a_well_formed_staff_details_is_quiet() {
        let details = "<staff-details number=\"1\" print-object=\"yes\">\
             <staff-lines>1</staff-lines>\
             <staff-size scaling=\"80\">75</staff-size></staff-details>";

        assert!(errors(&in_attributes(details)).is_empty());
    }

    /// Everything in a `<staff-details>` is optional, the element included.
    #[test]
    fn an_absent_or_empty_staff_details_is_quiet() {
        assert!(errors(&in_attributes("")).is_empty());
        assert!(errors(&in_attributes("<staff-details/>")).is_empty());
    }

    /// The malformations `WalkCursorVisitor::enter_staff_details` panics on.
    fn malformed_cases() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            (
                "a staff number that is not a number",
                "<staff-details number=\"upper\"><staff-lines>5</staff-lines></staff-details>",
                "upper",
            ),
            (
                "a line count that is not a number",
                "<staff-details><staff-lines>lots</staff-lines></staff-details>",
                "staff-lines",
            ),
            (
                "a negative line count",
                "<staff-details><staff-lines>-1</staff-lines></staff-details>",
                "staff-lines",
            ),
            (
                "a staff size that is not a number",
                "<staff-details><staff-size>small</staff-size></staff-details>",
                "staff-size",
            ),
            (
                "a content scaling that is not a number",
                "<staff-details><staff-size scaling=\"tiny\">75</staff-size></staff-details>",
                "tiny",
            ),
        ]
    }

    #[test]
    fn every_malformation_is_reported() {
        for (what, details, expected) in malformed_cases() {
            let found = errors(&in_attributes(details));

            assert_eq!(found.len(), 1, "{what} gave {found:?}");
            assert!(
                found[0].contains(expected),
                "{what}: {:?} does not name {expected}",
                found[0]
            );
        }
    }

    /// The rule this file exists for: every cause reported here is a cause the
    /// render walk actually panics on. A rule describing something the walk
    /// tolerates would be noise, and a panic no rule describes would be a crash
    /// with nothing said about it first.
    #[test]
    fn every_reported_cause_is_one_the_walk_panics_on() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        for (what, details, _) in malformed_cases() {
            assert_eq!(errors(&in_attributes(details)).len(), 1, "{what}");

            let xml = in_attributes(details);
            let walked = std::panic::catch_unwind(move || {
                let document = Document::parse(&xml).expect("test document does not parse");
                walk_document(&document, font(), &UserLayout::default(), &mut |_stage| {});
            });

            assert!(
                walked.is_err(),
                "{what} is reported by validation but walks without panicking, \
                 so the two have drifted apart"
            );
        }

        std::panic::set_hook(previous);
    }
}
