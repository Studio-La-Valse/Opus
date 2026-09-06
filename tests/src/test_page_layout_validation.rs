#[cfg(test)]
mod tests {
    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::Severity;
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::page_layout_visitor::PageLayoutVisitor;
    use lib::musicxml::walker::Walker;
    use lib::score::score_defaults::PageLayout;
    use roxmltree::Document;

    const GOOD_MARGINS: &str = "<page-margins type=\"both\">\
         <left-margin>1</left-margin><right-margin>2</right-margin>\
         <top-margin>3</top-margin><bottom-margin>4</bottom-margin></page-margins>";

    /// Wraps a `<page-layout>` in a `<defaults>`, where the score's own geometry
    /// is declared.
    fn in_defaults(page_layout: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <defaults>{page_layout}</defaults>\
             <part-list><score-part id=\"P1\"/></part-list>\
             <part id=\"P1\"><measure number=\"1\"/></part>\
             </score-partwise>"
        )
    }

    /// Wraps the same thing in a `<print>`, where a mid-document change to it is.
    fn in_print(page_layout: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"/></part-list>\
             <part id=\"P1\"><measure number=\"1\"><print>{page_layout}</print></measure></part>\
             </score-partwise>"
        )
    }

    fn errors(xml: &str) -> Vec<String> {
        let document = Document::parse(xml).expect("test document does not parse");

        let mut ctx = ValidationCtx::default();
        Walker::new(DefaultVisitor {}.uses(PageLayoutVisitor::default())).walk(&document, &mut ctx);

        ctx.issues
            .iter()
            .inspect(|issue| {
                assert_eq!(
                    issue.severity,
                    Severity::Error,
                    "page geometry cannot be guessed at, so nothing here is a warning"
                );
            })
            .map(|issue| issue.message.clone())
            .collect()
    }

    #[test]
    fn a_well_formed_page_layout_is_quiet() {
        let layout = format!(
            "<page-layout><page-height>3560</page-height><page-width>2797</page-width>\
             {GOOD_MARGINS}</page-layout>"
        );

        assert!(errors(&in_defaults(&layout)).is_empty());
        assert!(errors(&in_print(&layout)).is_empty());
    }

    /// Everything in a `<page-layout>` is optional, including the element.
    #[test]
    fn absent_or_empty_page_layout_is_quiet() {
        assert!(errors(&in_defaults("")).is_empty());
        assert!(errors(&in_defaults("<page-layout/>")).is_empty());
        assert!(errors(&in_print("")).is_empty());
    }

    /// `type` is optional and means "both", so only a present-and-wrong value is
    /// a defect.
    #[test]
    fn an_absent_margins_type_is_quiet() {
        let layout = "<page-layout><page-margins>\
             <left-margin>1</left-margin><right-margin>2</right-margin>\
             <top-margin>3</top-margin><bottom-margin>4</bottom-margin></page-margins></page-layout>";

        assert!(errors(&in_defaults(layout)).is_empty());
    }

    /// The four malformations `PageLayout::from_mxml` panics on. Each is checked
    /// in both places a `<page-layout>` can appear, since one parser reads both.
    fn malformed_cases() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            (
                "a type outside the three MusicXML allows",
                "<page-layout><page-margins type=\"sideways\">\
                 <left-margin>1</left-margin><right-margin>2</right-margin>\
                 <top-margin>3</top-margin><bottom-margin>4</bottom-margin>\
                 </page-margins></page-layout>",
                "sideways",
            ),
            (
                "a missing margin edge",
                "<page-layout><page-margins type=\"odd\">\
                 <left-margin>1</left-margin><right-margin>2</right-margin>\
                 <top-margin>3</top-margin></page-margins></page-layout>",
                "bottom-margin",
            ),
            (
                "a margin that is not a number",
                "<page-layout><page-margins type=\"odd\">\
                 <left-margin>lots</left-margin><right-margin>2</right-margin>\
                 <top-margin>3</top-margin><bottom-margin>4</bottom-margin>\
                 </page-margins></page-layout>",
                "left-margin",
            ),
            (
                "a page size that is not a number",
                "<page-layout><page-width>wide</page-width></page-layout>",
                "page-width",
            ),
        ]
    }

    #[test]
    fn every_malformation_is_reported_wherever_it_appears() {
        for (what, layout, expected) in malformed_cases() {
            for (where_, xml) in [
                ("defaults", in_defaults(layout)),
                ("print", in_print(layout)),
            ] {
                let found = errors(&xml);

                assert_eq!(found.len(), 1, "{what} in <{where_}> gave {found:?}");
                assert!(
                    found[0].contains(expected),
                    "{what} in <{where_}>: {:?} does not name {expected}",
                    found[0]
                );
            }
        }
    }

    /// The rule this file exists for: every cause reported here is a cause the
    /// render walk actually panics on. A validation rule that described
    /// something the renderer tolerates would be noise, and one that missed a
    /// panic would leave a crash undescribed.
    #[test]
    fn every_reported_cause_is_one_the_parser_panics_on() {
        let previous = std::panic::take_hook();
        std::panic::set_hook(Box::new(|_| {}));

        for (what, layout, _) in malformed_cases() {
            assert_eq!(
                errors(&in_defaults(layout)).len(),
                1,
                "{what} should be reported"
            );

            let owned = layout.to_string();
            let parsed = std::panic::catch_unwind(move || {
                let document = Document::parse(&owned).expect("layout does not parse");
                PageLayout::from_mxml(&document.root_element())
            });

            assert!(
                parsed.is_err(),
                "{what} is reported by validation but parses without panicking, \
                 so the two have drifted apart"
            );
        }

        std::panic::set_hook(previous);
    }
}
