//! `GroupSymbolVisitor`: the `<group-symbol>` values a document walk panics on,
//! described in plain words first.
//!
//! Same bargain as `test_staff_details_validation`: `GroupSymbol::from_mxml` is
//! strict because a symbol it cannot read is one it would have to invent, and an
//! invented one silently redraws the score's structure. Every cause reported
//! here is checked to be a cause the walk really does panic on.

#[cfg(test)]
mod tests {
    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::Severity;
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::validators::group_symbol_visitor::GroupSymbolVisitor;
    use lib::musicxml::walker::Walker;
    use lib::score::engrave::walk_document;

    use roxmltree::Document;

    /// Wraps `part_groups` around a single one-note part, so the `<part-list>`
    /// is well formed in every respect but the symbols under test.
    fn in_part_list(part_groups: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\"><part-list>{part_groups}\
             <score-part id=\"P1\"><part-name>P</part-name></score-part>\
             <part-group type=\"stop\"/></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef><sign>G</sign><line>2</line></clef></attributes>\
             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type></note>\
             </measure></part></score-partwise>"
        )
    }

    /// One `<part-group type="start">` carrying `symbol` as its `<group-symbol>`.
    fn start_with(symbol: &str) -> String {
        format!("<part-group type=\"start\"><group-symbol>{symbol}</group-symbol></part-group>")
    }

    fn errors(xml: &str) -> Vec<String> {
        let document = Document::parse(xml).expect("test document does not parse");

        let mut ctx = ValidationCtx::default();
        Walker::new(DefaultVisitor {}.uses(GroupSymbolVisitor::default()))
            .walk(&document, &mut ctx);

        ctx.issues
            .iter()
            .inspect(|issue| {
                assert_eq!(
                    issue.severity,
                    Severity::Error,
                    "a symbol cannot be guessed at, so nothing here is a warning"
                );
            })
            .map(|issue| issue.message.clone())
            .collect()
    }

    #[test]
    fn every_value_the_format_defines_is_quiet() {
        for symbol in ["none", "brace", "bracket", "line", "square"] {
            let found = errors(&in_part_list(&start_with(symbol)));
            assert!(found.is_empty(), "'{symbol}' gave {found:?}");
        }
    }

    /// `<group-symbol>` is optional, and so is the `<part-group>` carrying it.
    #[test]
    fn an_absent_symbol_is_quiet() {
        assert!(errors(&in_part_list("<part-group type=\"start\"/>")).is_empty());
        assert!(
            errors(&in_part_list(
                "<part-group type=\"start\"><group-name>Brass</group-name></part-group>"
            ))
            .is_empty()
        );
    }

    /// Surrounding whitespace and casing are the exporter's business, not the
    /// reader's, so neither is reported.
    #[test]
    fn padding_and_casing_are_quiet() {
        assert!(errors(&in_part_list(&start_with("\n    Bracket\n  "))).is_empty());
    }

    /// The values `GroupSymbol::from_mxml` panics on.
    fn malformed_cases() -> Vec<(&'static str, &'static str)> {
        vec![
            ("a symbol the format does not define", "curly"),
            ("a near miss on a real value", "brackets"),
            ("an empty element", ""),
            ("a value that is only whitespace", "   "),
        ]
    }

    #[test]
    fn every_malformation_is_reported() {
        for (what, symbol) in malformed_cases() {
            let found = errors(&in_part_list(&start_with(symbol)));

            assert_eq!(found.len(), 1, "{what} gave {found:?}");

            // Quoting the value back is what makes the message actionable, but
            // there is nothing to quote when the element was empty.
            if !symbol.trim().is_empty() {
                assert!(
                    found[0].contains(symbol.trim()),
                    "{what}: {:?} does not name {symbol:?}",
                    found[0]
                );
            }
        }
    }

    /// A `<part-list>` reports each bad symbol separately rather than stopping
    /// at the first, so one walk names everything the document has to fix.
    #[test]
    fn each_bad_symbol_is_reported_on_its_own() {
        let part_groups = format!("{}{}", start_with("curly"), start_with("squiggle"));
        let found = errors(&in_part_list(&part_groups));

        assert_eq!(found.len(), 2, "{found:?}");
        assert!(found[0].contains("curly"));
        assert!(found[1].contains("squiggle"));
    }

    /// The rule this file exists for: every cause reported here is a cause the
    /// render walk actually panics on. A rule describing something the walk
    /// tolerates would be noise, and a panic no rule describes would be a crash
    /// with nothing said about it first.
    #[test]
    fn every_reported_cause_is_one_the_walk_panics_on() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        for (what, symbol) in malformed_cases() {
            let xml = in_part_list(&start_with(symbol));
            assert_eq!(errors(&xml).len(), 1, "{what}");

            let walked = std::panic::catch_unwind(move || {
                let document = Document::parse(&xml).expect("test document does not parse");
                walk_document(&document, &mut |_stage| {});
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
