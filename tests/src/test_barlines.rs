//! How tall a barline is drawn.
//!
//! Two barlines bound a system of music: the systemic one down its left edge,
//! drawn by `render_system`, and the one at each measure end, drawn by
//! `render_section_measure`. Both normally run from the top line of the first
//! visible staff to the bottom line of the last, which is exactly the height
//! those staves occupy.
//!
//! A staff of a single line -- a percussion or rhythm staff -- breaks that,
//! because it is zero tenths tall: a barline held to its height would be a
//! point. Such a staff is barred one space above and one below its line
//! instead, twenty tenths in all, and these tests pin that down at both ends of
//! a system and for a staff that has an ordinary staff above it.

#[cfg(test)]
mod tests {
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::line::Line;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::core::staff_idx::StaffIdx;
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::render_fonts::RenderFonts;
    use lib::score::visual::render_pass::{BaseRenderer, RenderPass};
    use lib::score::visual::score::Score;
    use lib::score::visual::section::Section;
    use lib::score::visual::staff::Staff;
    use lib::score::visual::system::System;
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

    /// A one-part, one-staff, one-measure score holding a single whole note.
    /// `clef` and `staff_details` go inside the `<attributes>`.
    fn one_staff_score_xml(clef: &str, staff_details: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             {clef}{staff_details}</attributes>\
             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type></note>\
             </measure></part></score-partwise>"
        )
    }

    /// A one-part, two-staff, one-measure score: a treble staff over a
    /// percussion staff whose line count `staff_details` declares.
    fn two_staff_score_xml(staff_details: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef number=\"1\"><sign>G</sign><line>2</line></clef>\
             <clef number=\"2\"><sign>percussion</sign></clef>\
             {staff_details}</attributes>\
             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type><staff>1</staff></note>\
             <backup><duration>16</duration></backup>\
             <note default-x=\"80\"><pitch><step>C</step><octave>4</octave></pitch>\
             <duration>16</duration><voice>2</voice><type>whole</type><staff>2</staff></note>\
             </measure></part></score-partwise>"
        )
    }

    fn engrave(xml: &str) -> Score {
        let document = Document::parse(xml).expect("test document does not parse");
        let (mut score, defaults, _messages) = walk_document(
            &document,
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_stage| {},
        );

        arrange_score(
            &mut score,
            &defaults,
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_stage| {},
        );

        score
    }

    /// The one system of a one-measure score.
    fn system(score: &Score) -> &System {
        let systems: Vec<&System> = score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .collect();

        assert_eq!(systems.len(), 1, "expected a single system");
        systems[0]
    }

    /// The one section of a one-part score.
    fn section(score: &Score) -> &Section {
        let sections: Vec<&Section> = system(score).sections.values().collect();

        assert_eq!(sections.len(), 1, "expected a single section");
        sections[0]
    }

    fn staves(score: &Score) -> Vec<(StaffIdx, &Staff)> {
        section(score)
            .part_groups
            .values()
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.iter())
            .map(|(idx, staff)| (*idx, staff))
            .collect()
    }

    /// The only line a render pass drew, which both barline renderers produce
    /// one of per call.
    fn only_line(out: Vec<DrawableElement>) -> Line {
        assert_eq!(out.len(), 1, "expected a single drawn element");

        match out.into_iter().next().unwrap() {
            DrawableElement::Line(line) => line,
            _ => panic!("a barline renderer drew something that is not a line"),
        }
    }

    /// The systemic barline down the left edge of `system`.
    fn systemic_barline(system: &System) -> Line {
        let fonts = RenderFonts::music_only(font());
        let mut out: Vec<DrawableElement> = Vec::new();
        BaseRenderer {}.render_system(system, &fonts, &mut out);

        only_line(out)
    }

    /// The barline at the end of the one measure of `section`.
    fn measure_barline(section: &Section) -> Line {
        let measures: Vec<_> = section.measures.values().collect();
        assert_eq!(measures.len(), 1, "expected a single measure");

        let fonts = RenderFonts::music_only(font());
        let mut out: Vec<DrawableElement> = Vec::new();
        BaseRenderer {}.render_section_measure(measures[0], &fonts, &mut out);

        only_line(out)
    }

    /// Both barlines of a one-measure score, as `(start.y, end.y)` offsets from
    /// the top line of the score's first staff. They bound the same music, so
    /// they must agree.
    fn barline_span(score: &Score) -> (f32, f32) {
        let top_line = staves(score)[0].1.xy.y;

        let systemic = systemic_barline(system(score));
        let measure_end = measure_barline(section(score));

        assert_eq!(
            (systemic.start.y, systemic.end.y),
            (measure_end.start.y, measure_end.end.y),
            "the systemic barline and the one at the measure end disagree"
        );

        (systemic.start.y - top_line, systemic.end.y - top_line)
    }

    /// An ordinary staff needs no help: its own lines enclose the barline.
    #[test]
    fn a_five_line_staff_is_barred_from_its_top_line_to_its_bottom_one() {
        let score = engrave(&one_staff_score_xml(
            "<clef><sign>G</sign><line>2</line></clef>",
            "",
        ));

        assert_eq!(barline_span(&score), (0., 40.));
    }

    /// The case the whole thing is about: a staff of one line is zero tenths
    /// tall, so its barlines are drawn a space above and a space below the
    /// line, twenty tenths in all.
    #[test]
    fn a_single_line_staff_is_barred_a_space_above_and_below_its_line() {
        let score = engrave(&one_staff_score_xml(
            "<clef><sign>percussion</sign></clef>",
            "<staff-details><staff-lines>1</staff-lines></staff-details>",
        ));

        assert_eq!(barline_span(&score), (-10., 10.));
    }

    /// The overhang is a staff space, so it follows a staff that is drawn
    /// smaller than the rest of the score.
    #[test]
    fn the_overhang_is_a_space_of_the_staff_it_overhangs() {
        for (scale, expected) in [(1., 10.), (0.5, 5.), (2., 20.)] {
            let staff = Staff {
                lines: 1,
                scale,
                ..Default::default()
            };

            assert_eq!(staff.barline_overhang(), expected, "at scale {scale}");
        }
    }

    /// Only a staff of exactly one line is short of room. Every other count
    /// encloses spaces of its own, and a staff drawn without any lines has
    /// nothing to bar in the first place.
    #[test]
    fn only_a_single_line_staff_overhangs() {
        for (lines, expected) in [(0, 0.), (1, 10.), (2, 0.), (5, 0.), (6, 0.)] {
            let staff = Staff {
                lines,
                ..Default::default()
            };

            assert_eq!(
                staff.barline_overhang(),
                expected,
                "a staff of {lines} lines"
            );
        }
    }

    /// A single-line staff under an ordinary one only stretches the barline at
    /// the end it is at: the top stays on the treble staff's top line, and the
    /// bottom drops a space past the single line.
    #[test]
    fn a_single_line_staff_below_an_ordinary_one_stretches_only_the_bottom() {
        let score = engrave(&two_staff_score_xml(
            "<staff-details number=\"2\"><staff-lines>1</staff-lines></staff-details>",
        ));

        let staves = staves(&score);
        assert_eq!(staves.len(), 2, "expected two staves");
        assert_eq!(staves[1].1.lines, 1, "staff 2 was declared one line");

        let top_line = staves[0].1.xy.y;
        let single_line = staves[1].1.xy.y;

        let (start, end) = barline_span(&score);
        assert_eq!(
            start, 0.,
            "the barline starts on the treble staff's top line"
        );
        assert_eq!(
            end + top_line,
            single_line + 10.,
            "the barline ends a space below the single line"
        );
    }

    /// The mirror image: a single-line staff above an ordinary one lifts the
    /// top of the barline a space, and leaves the bottom on the lower staff's
    /// bottom line.
    #[test]
    fn a_single_line_staff_above_an_ordinary_one_stretches_only_the_top() {
        let score = engrave(&two_staff_score_xml(
            "<staff-details number=\"1\"><staff-lines>1</staff-lines></staff-details>",
        ));

        let staves = staves(&score);
        assert_eq!(staves[0].1.lines, 1, "staff 1 was declared one line");

        let top_line = staves[0].1.xy.y;
        let bottom_staff = staves[1].1;

        let (start, end) = barline_span(&score);
        assert_eq!(
            start, -10.,
            "the barline starts a space above the single line"
        );
        assert_eq!(
            end + top_line,
            bottom_staff.xy.y + bottom_staff.height(),
            "the barline ends on the lower staff's bottom line"
        );
    }

    /// And the same in reverse: two ordinary staves are barred exactly as far
    /// as they reach, single-line staves being the only ones that overhang.
    #[test]
    fn two_ordinary_staves_are_barred_from_the_first_to_the_last() {
        let score = engrave(&two_staff_score_xml(""));

        let staves = staves(&score);
        let top_line = staves[0].1.xy.y;
        let bottom_staff = staves[1].1;

        let (start, end) = barline_span(&score);
        assert_eq!(start, 0.);
        assert_eq!(end + top_line, bottom_staff.xy.y + bottom_staff.height());
    }
}
