#[cfg(test)]
mod tests {
    use lib::score::engrave::walk_document;
    use lib::score::layout_options::UserLayout;
    use lib::score::score_defaults::{PageLayout, PageMargins, ScoreDefaults};
    use lib::smufl::smufl_font::SmuflFont;
    use roxmltree::{Document, ParsingOptions};
    use std::fs::read_to_string;

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    fn margins(left: f32, right: f32, top: f32, bottom: f32) -> PageMargins {
        PageMargins {
            left,
            right,
            top,
            bottom,
        }
    }

    /// Parses a bare `<page-layout>` element.
    fn layout(xml: &str) -> PageLayout {
        let document = Document::parse(xml).expect("test element does not parse");

        PageLayout::from_mxml(&document.root_element())
    }

    #[test]
    fn an_untyped_page_margins_is_read_as_both() {
        let parsed = layout(
            "<page-layout><page-margins>\
             <left-margin>1</left-margin><right-margin>2</right-margin>\
             <top-margin>3</top-margin><bottom-margin>4</bottom-margin>\
             </page-margins></page-layout>",
        );

        assert!(parsed.margins_both.is_some());
        assert!(parsed.margins_odd.is_none());
        assert!(parsed.margins_even.is_none());

        // "both" answers for either parity.
        assert_eq!(parsed.margins_for(1).unwrap().left, 1.);
        assert_eq!(parsed.margins_for(2).unwrap().left, 1.);
    }

    #[test]
    fn odd_and_even_margins_answer_for_their_own_parity() {
        let parsed = layout(
            "<page-layout>\
             <page-margins type=\"odd\"><left-margin>10</left-margin><right-margin>0</right-margin>\
             <top-margin>0</top-margin><bottom-margin>0</bottom-margin></page-margins>\
             <page-margins type=\"even\"><left-margin>20</left-margin><right-margin>0</right-margin>\
             <top-margin>0</top-margin><bottom-margin>0</bottom-margin></page-margins>\
             </page-layout>",
        );

        assert_eq!(parsed.margins_for(1).unwrap().left, 10.);
        assert_eq!(parsed.margins_for(2).unwrap().left, 20.);
        assert_eq!(parsed.margins_for(3).unwrap().left, 10.);
    }

    /// `<page-height>` / `<page-width>` are optional, and a `<page-layout>` that
    /// carries only margins must not be read as one that sets the size to
    /// anything.
    #[test]
    fn a_page_layout_without_a_size_names_no_size() {
        let parsed = layout("<page-layout/>");

        assert!(parsed.width.is_none());
        assert!(parsed.height.is_none());
        assert!(parsed.margins_for(1).is_none());
    }

    fn defaults_with_odd_and_even() -> ScoreDefaults {
        ScoreDefaults {
            page_layout: PageLayout {
                width: Some(1000.),
                height: Some(2000.),
                margins_odd: Some(margins(177., 79., 127., 45.)),
                margins_even: Some(margins(141., 115., 127., 45.)),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn with_no_overrides_a_page_takes_the_defaults_for_its_parity() {
        let defaults = defaults_with_odd_and_even();

        assert_eq!(defaults.resolve_page(1).margins.left, 177.);
        assert_eq!(defaults.resolve_page(2).margins.left, 141.);
        assert_eq!(defaults.resolve_page(1).width, 1000.);
        assert_eq!(defaults.resolve_page(1).height, 2000.);
    }

    /// The case that made this worth having: a `<print>` carrying an untyped
    /// (`both`) `<page-margins>` has to replace the `odd` margins the defaults
    /// set for that page, not lose to them for sitting in a less specific slot.
    #[test]
    fn a_print_override_beats_a_more_specific_default() {
        let mut defaults = defaults_with_odd_and_even();
        defaults.page_overrides.insert(
            1,
            PageLayout {
                margins_both: Some(margins(177., 79., 64., 45.)),
                ..Default::default()
            },
        );

        // The override wins on the page it names...
        assert_eq!(defaults.resolve_page(1).margins.top, 64.);
        // ...and, being "both", on the following pages too, until changed.
        assert_eq!(defaults.resolve_page(2).margins.top, 64.);
    }

    /// An override applies from its own page onward, so a page before it is
    /// untouched and a page after it keeps it until something else changes it.
    #[test]
    fn an_override_applies_from_its_page_onward() {
        let mut defaults = defaults_with_odd_and_even();
        defaults.page_overrides.insert(
            3,
            PageLayout {
                margins_both: Some(margins(1., 2., 3., 4.)),
                ..Default::default()
            },
        );

        assert_eq!(defaults.resolve_page(2).margins.top, 127., "before it");
        assert_eq!(defaults.resolve_page(3).margins.top, 3., "on it");
        assert_eq!(defaults.resolve_page(4).margins.top, 3., "after it");
    }

    /// An override changes what it names and nothing else: a size-only override
    /// leaves the margins alone, and a margins-only one leaves the size alone.
    #[test]
    fn an_override_changes_only_what_it_names() {
        let mut defaults = defaults_with_odd_and_even();
        defaults.page_overrides.insert(
            2,
            PageLayout {
                width: Some(500.),
                ..Default::default()
            },
        );
        defaults.page_overrides.insert(
            3,
            PageLayout {
                margins_both: Some(margins(1., 2., 3., 4.)),
                ..Default::default()
            },
        );

        let page2 = defaults.resolve_page(2);
        assert_eq!(page2.width, 500., "the size it named");
        assert_eq!(page2.height, 2000., "the size it did not");
        assert_eq!(page2.margins.left, 141., "margins it did not name");

        let page3 = defaults.resolve_page(3);
        assert_eq!(page3.width, 500., "still in force from page 2");
        assert_eq!(page3.margins.left, 1.);
    }

    /// End to end: ActorPreludeSample gives every page its own top margin
    /// through `<print><page-layout>`, and nothing else in the bundled samples
    /// does.
    #[test]
    fn the_sample_that_sets_per_page_margins_gets_them() {
        let data = asset("assets/xmlsamples/ActorPreludeSample.musicxml");
        let document = Document::parse_with_options(
            &data,
            ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("sample does not parse");

        let font = SmuflFont::load(
            &asset("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json"),
            &asset("assets/smufl/metadata/glyphnames.json"),
        );

        let (_score, defaults, _messages) =
            walk_document(&document, &font, &UserLayout::default(), &mut |_stage| {});

        // One `<print><page-layout>` per page, each with its own top margin.
        let tops: Vec<f32> = (1..=4)
            .map(|n| defaults.resolve_page(n).margins.top)
            .collect();
        assert_eq!(tops, vec![64., 104., 109., 104.]);

        // The left/right margins alternate with the page parity, as the
        // document's own odd/even defaults also do.
        let lefts: Vec<f32> = (1..=4)
            .map(|n| defaults.resolve_page(n).margins.left)
            .collect();
        assert_eq!(lefts, vec![177., 141., 177., 141.]);
    }
}
