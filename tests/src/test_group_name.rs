//! Where a part's and a part-group's name is placed, and when it draws.
//!
//! A name is drawn as right-aligned text ending a configured padding left of
//! its level's symbol (or of where that symbol would have been), the box
//! reaching left to the page's margin. The first system names in full; every
//! later one abbreviates. These pin that geometry and those conditions.

#[cfg(test)]
mod tests {
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::{APP_DEFAULTS, GroupNameLayout, UserLayout};
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::visual::part::Part;
    use lib::score::visual::part_group::PartGroup;
    use lib::score::visual::score::Score;
    use lib::score::visual::system::System;
    use lib::smufl::smufl_font::SmuflFont;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    const BRAVURA_META: &str = "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";

    /// A braced two-part group, named and abbreviated, nested inside an outer
    /// bracket so it reaches the visual tree as a part-group rather than a
    /// section. Both its symbol and its name draw.
    const NAMED_GROUP: &str = r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
        <part-group type="start">
            <group-name>Strings</group-name>
            <group-abbreviation>Str.</group-abbreviation>
            <group-symbol>brace</group-symbol>
        </part-group>
        <score-part id="P1">
            <part-name>Violin</part-name><part-abbreviation>Vln.</part-abbreviation>
        </score-part>
        <score-part id="P2"><part-name>Viola</part-name></score-part>
        <part-group type="stop"/>
        <part-group type="stop"/>"#;

    fn asset(relative: &str) -> String {
        read_to_string(format!("{}/../{relative}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative}: {e}"))
    }

    fn font() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&asset(BRAVURA_META)))
    }

    /// A `<score-partwise>` around `part_list`, giving every part in `parts`
    /// `systems` one-note measures, each on a new system.
    fn score_xml(part_list: &str, parts: &[&str], systems: usize) -> String {
        let bodies: String = parts
            .iter()
            .map(|id| {
                let measures: String = (1..=systems)
                    .map(|n| {
                        let attributes = if n == 1 {
                            "<attributes><divisions>4</divisions>\
                             <key><fifths>0</fifths><mode>major</mode></key>\
                             <time><beats>4</beats><beat-type>4</beat-type></time>\
                             <clef><sign>G</sign><line>2</line></clef></attributes>"
                        } else {
                            ""
                        };
                        // Indent every system so there is room to the left of
                        // the symbols for a name box.
                        format!(
                            "<measure number=\"{n}\" width=\"300\">\
                             <print new-system=\"yes\"><system-layout><system-margins>\
                             <left-margin>150</left-margin><right-margin>0</right-margin>\
                             </system-margins></system-layout></print>{attributes}\
                             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
                             <duration>16</duration><voice>1</voice><type>whole</type></note>\
                             </measure>"
                        )
                    })
                    .collect();
                format!("<part id=\"{id}\">{measures}</part>")
            })
            .collect();

        format!(
            "<score-partwise version=\"4.0\"><part-list>{part_list}</part-list>{bodies}\
             </score-partwise>"
        )
    }

    fn walk(xml: &str) -> (Score, ScoreDefaults) {
        let document = roxmltree::Document::parse_with_options(
            xml,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .expect("test document does not parse");

        let (score, defaults, _) = walk_document(&document, &mut |_stage| {});
        (score, defaults)
    }

    fn arrange(score: &mut Score, defaults: &ScoreDefaults, user_layout: &UserLayout) {
        arrange_score(score, defaults, font(), user_layout, &mut |_stage| {});
    }

    fn engrave_with(xml: &str, user_layout: &UserLayout) -> Score {
        let (mut score, defaults) = walk(xml);
        arrange(&mut score, &defaults, user_layout);
        score
    }

    fn engrave(xml: &str) -> Score {
        engrave_with(xml, &UserLayout::default())
    }

    /// The system at `ordinal`, 1-based: `system(&score, 1)` is the first system
    /// of the score -- the one that names in full.
    fn system(score: &Score, ordinal: u32) -> &System {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .find(|system| system.index == ordinal)
            .unwrap_or_else(|| panic!("no system with index {ordinal}"))
    }

    /// The first part-group of a system, across every section.
    fn group(system: &System) -> &PartGroup {
        system
            .sections
            .values()
            .flat_map(|section| section.part_groups.values())
            .next()
            .expect("a part-group")
    }

    fn part<'a>(system: &'a System, id: &str) -> &'a Part {
        system
            .sections
            .values()
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.iter())
            .find(|(part_id, _)| part_id.as_str() == id)
            .map(|(_, part)| part)
            .expect("the requested part")
    }

    fn close(a: f32, b: f32, what: &str) {
        assert!((a - b).abs() < 0.05, "{what}: expected {b}, got {a}");
    }

    #[test]
    fn the_box_ends_a_padding_left_of_a_drawn_symbol() {
        let score = engrave(&score_xml(NAMED_GROUP, &["P1", "P2"], 1));
        let group = group(system(&score, 1));

        let padding = APP_DEFAULTS.group_name.padding;

        assert!(group.shows_symbol(), "the brace draws");
        assert!(group.shows_name(), "the group name draws");
        close(
            group.name.bounds().x_max(),
            group.symbol.bounds().x_min() - padding,
            "name right edge sits a padding left of the brace",
        );
    }

    #[test]
    fn without_a_symbol_the_box_ends_where_the_symbol_would_have_been() {
        // The inner group asks for `none`: it still names itself, but nothing is
        // drawn for the name to align against, so it aligns against the edge the
        // (single, undrawn) section handed it -- the system's own left edge.
        let part_list = r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
            <part-group type="start">
                <group-name>Strings</group-name>
                <group-symbol>none</group-symbol>
            </part-group>
            <score-part id="P1"><part-name>Violin</part-name></score-part>
            <score-part id="P2"><part-name>Viola</part-name></score-part>
            <part-group type="stop"/>
            <part-group type="stop"/>"#;

        let score = engrave(&score_xml(part_list, &["P1", "P2"], 1));
        let group = group(system(&score, 1));

        let padding = APP_DEFAULTS.group_name.padding;

        assert!(!group.shows_symbol(), "an explicit 'none' draws nothing");
        assert!(group.shows_name());
        close(
            group.name.bounds().x_max(),
            group.xy.x - padding,
            "name right edge falls a padding left of the system edge",
        );
    }

    #[test]
    fn the_box_reaches_left_to_the_page_margin() {
        let score = engrave(&score_xml(NAMED_GROUP, &["P1", "P2"], 1));
        let page = score.pages.values().next().expect("a page");
        let margin_left = page.margins.left;

        let system = system(&score, 1);
        close(
            group(system).name.bounds().x_min(),
            margin_left,
            "part-group name left edge is the page margin",
        );

        let violin = part(system, "P1");
        assert!(violin.shows_name());
        close(
            violin.name.bounds().x_min(),
            margin_left,
            "part name left edge is the page margin",
        );
    }

    #[test]
    fn the_name_is_centred_on_the_staves_it_covers() {
        let score = engrave(&score_xml(NAMED_GROUP, &["P1", "P2"], 1));
        let group = group(system(&score, 1));

        let bounds = group.name.bounds();
        close(
            group.name.anchor().y,
            bounds.y_min() + bounds.height() / 2.,
            "anchor is the box's vertical centre",
        );
        assert!(bounds.height() > 0., "the box spans the staves");
    }

    #[test]
    fn the_first_system_names_in_full_and_later_systems_abbreviate() {
        let score = engrave(&score_xml(NAMED_GROUP, &["P1", "P2"], 2));

        assert_eq!(group(system(&score, 1)).name.text(), "Strings");
        assert_eq!(part(system(&score, 1), "P1").name.text(), "Violin");

        assert_eq!(
            group(system(&score, 2)).name.text(),
            "Str.",
            "the second system uses the group abbreviation",
        );
        assert_eq!(
            part(system(&score, 2), "P1").name.text(),
            "Vln.",
            "the second system uses the part abbreviation",
        );
        assert_eq!(
            part(system(&score, 2), "P2").name.text(),
            "Viola",
            "a missing abbreviation falls back to the full name",
        );
    }

    #[test]
    fn a_padding_override_moves_only_the_right_edge() {
        let extra = 20.;
        let widened = APP_DEFAULTS.group_name.padding + extra;

        let xml = score_xml(NAMED_GROUP, &["P1", "P2"], 1);
        let base = engrave_with(&xml, &UserLayout::default());
        let wide = engrave_with(
            &xml,
            &UserLayout {
                group_name: GroupNameLayout {
                    padding: Some(widened),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        let base_box = group(system(&base, 1)).name.bounds();
        let wide_box = group(system(&wide, 1)).name.bounds();

        close(wide_box.x_min(), base_box.x_min(), "left edge stays put");
        close(
            wide_box.x_max(),
            base_box.x_max() - extra,
            "right edge moves in by the extra padding",
        );
    }

    #[test]
    fn a_part_with_no_name_draws_nothing() {
        let part_list = r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
            <part-group type="start"><group-symbol>brace</group-symbol></part-group>
            <score-part id="P1"><part-name></part-name></score-part>
            <score-part id="P2"><part-name>Viola</part-name></score-part>
            <part-group type="stop"/>
            <part-group type="stop"/>"#;

        let score = engrave(&score_xml(part_list, &["P1", "P2"], 1));
        let system = system(&score, 1);

        assert!(
            !part(system, "P1").shows_name(),
            "an empty name draws nothing"
        );
        assert!(part(system, "P2").shows_name(), "a named part still draws");
    }

    #[test]
    fn a_one_part_group_draws_no_group_name() {
        let part_list = r#"<part-group type="start"><group-symbol>bracket</group-symbol></part-group>
            <part-group type="start">
                <group-name>Solo</group-name>
                <group-symbol>brace</group-symbol>
            </part-group>
            <score-part id="P1"><part-name>Violin</part-name></score-part>
            <part-group type="stop"/>
            <part-group type="stop"/>"#;

        let score = engrave(&score_xml(part_list, &["P1"], 1));
        let group = group(system(&score, 1));

        assert!(!group.shows_name(), "a group of one is not a group");
        assert!(!group.shows_symbol());
    }

    /// A one-line percussion staff occupies a standard staff's height, its
    /// single line drawn where the middle line would be, so its name is boxed
    /// and centred exactly as a five-line staff's is -- and its anchor lands on
    /// that line. Regression: the name of every percussion instrument used to
    /// be dropped, first because `shows_name` gated on a positive span and then
    /// because the staff was zero tenths tall.
    #[test]
    fn a_single_line_percussion_staff_still_draws_its_name() {
        let xml = r#"<score-partwise version="4.0">
            <part-list>
                <score-part id="P1"><part-name>Snare Drum</part-name></score-part>
            </part-list>
            <part id="P1"><measure number="1" width="300">
                <print new-system="yes"><system-layout><system-margins>
                    <left-margin>150</left-margin><right-margin>0</right-margin>
                </system-margins></system-layout></print>
                <attributes><divisions>1</divisions>
                    <time><beats>4</beats><beat-type>4</beat-type></time>
                    <clef><sign>percussion</sign></clef>
                    <staff-details><staff-lines>1</staff-lines></staff-details>
                </attributes>
                <note><rest measure="yes"/><duration>4</duration><voice>1</voice></note>
            </measure></part>
        </score-partwise>"#;

        let score = engrave(xml);
        let snare = part(system(&score, 1), "P1");

        assert_eq!(snare.name.text(), "Snare Drum");
        assert!(snare.shows_name(), "a one-line staff still gets its name");
        assert_eq!(
            snare.name.bounds().height(),
            40.,
            "the name box is a standard staff tall, like any other",
        );

        let staff = snare.staves.values().next().expect("the one staff");
        let line_y = staff.xy.y + staff.top_line_offset();
        close(
            snare.name.anchor().y,
            line_y,
            "the name anchor lands on the drawn line",
        );
    }
}
