//! `<staff-details><staff-lines>`: how many lines a staff is drawn with.
//!
//! Five is only the default. A percussion part is often written on one line, a
//! tablature staff has one per string, and `0` asks for a staff with no lines at
//! all. The count reaches three places, each checked here: how much room the
//! staff takes (always a standard staff's, so its neighbours do not collapse
//! onto it), the renderer (how many lines it draws, and where), and the ledger
//! lines, which start below whatever the staff's own bottom drawn line is.
//!
//! A staff of two to five lines draws them from the top of the block it
//! occupies; a six-line staff fills the block; a one-line staff is the
//! percussion exception, its single line drawn on the block's middle, standing
//! in for a five-line staff's middle line, with clef, time signature, rests and
//! name centred on it.

#[cfg(test)]
mod tests {
    use lib::drawable::drawable_element::DrawableElement;
    use lib::drawable::elements::line::Line;
    use lib::score::core::staff_idx::StaffIdx;
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::UserLayout;
    use lib::score::visual::clef::Clef;
    use lib::score::visual::render_fonts::RenderFonts;
    use lib::score::visual::render_pass::{BaseRenderer, RenderPass};
    use lib::score::visual::score::Score;
    use lib::score::visual::staff::Staff;
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

    /// A one-part, one-measure, one-staff score in treble clef. `staff_details`
    /// goes inside the `<attributes>` beside the clef, `notes` after them.
    fn score_xml(staff_details: &str, notes: &str) -> String {
        score_xml_with_clef(
            "<clef><sign>G</sign><line>2</line></clef>",
            staff_details,
            notes,
        )
    }

    /// The same, with the `<clef>` spelled out -- a staff that is not five lines
    /// is usually a staff whose clef names no pitch.
    fn score_xml_with_clef(clef: &str, staff_details: &str, notes: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             {clef}{staff_details}</attributes>\
             {notes}</measure></part></score-partwise>"
        )
    }

    /// A two-staff part, so that a `<staff-details number=\"n\">` can be seen to
    /// reach the staff it names and no other.
    fn two_staff_score_xml(staff_details: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef number=\"1\"><sign>G</sign><line>2</line></clef>\
             <clef number=\"2\"><sign>F</sign><line>4</line></clef>\
             {staff_details}</attributes>\
             <note default-x=\"80\"><pitch><step>C</step><octave>5</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type><staff>1</staff></note>\
             <backup><duration>16</duration></backup>\
             <note default-x=\"80\"><pitch><step>C</step><octave>3</octave></pitch>\
             <duration>16</duration><voice>2</voice><type>whole</type><staff>2</staff></note>\
             </measure></part></score-partwise>"
        )
    }

    /// A single whole note at `step``octave`, which is all a ledger line needs.
    fn note(step: &str, octave: i32) -> String {
        format!(
            "<note default-x=\"80\"><pitch><step>{step}</step><octave>{octave}</octave></pitch>\
             <duration>16</duration><voice>1</voice><type>whole</type></note>"
        )
    }

    /// Walk the document and arrange it, which is when ledger lines are placed.
    fn engrave(xml: &str) -> Score {
        let document = Document::parse(xml).expect("test document does not parse");
        let (mut score, defaults, _messages) =
            walk_document(&document, font(), &UserLayout::default(), &mut |_stage| {});

        arrange_score(
            &mut score,
            &defaults,
            font(),
            &UserLayout::default(),
            &mut |_stage| {},
        );

        score
    }

    fn staves(score: &Score) -> Vec<(StaffIdx, &Staff)> {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.iter())
            .map(|(idx, staff)| (*idx, staff))
            .collect()
    }

    /// The one staff of a one-staff score.
    fn staff(score: &Score) -> &Staff {
        let staves = staves(score);
        assert_eq!(staves.len(), 1, "expected a single staff");

        staves[0].1
    }

    /// The clef the one staff of a one-staff score opens with.
    fn opening_clef(score: &Score) -> &Clef {
        staff(score)
            .measures
            .values()
            .next()
            .expect("the staff has no measures")
            .clef_start
            .as_ref()
            .expect("the measure has no opening clef")
    }

    fn ledger_count(score: &Score) -> usize {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.measures.values())
            .map(|measure| measure.ledgers.len())
            .sum()
    }

    /// The lines `BaseRenderer` draws for `staff`, top to bottom.
    fn rendered_lines(staff: &Staff) -> Vec<Line> {
        let fonts = RenderFonts::music_only(font());
        let mut out: Vec<DrawableElement> = Vec::new();
        BaseRenderer {}.render_staff(staff, &fonts, &mut out);

        out.into_iter()
            .map(|element| match element {
                DrawableElement::Line(line) => line,
                _ => panic!("render_staff drew something that is not a line"),
            })
            .collect()
    }

    #[test]
    fn a_staff_is_five_lines_unless_the_document_says_otherwise() {
        let score = engrave(&score_xml("", &note("C", 5)));
        let staff = staff(&score);

        assert_eq!(staff.lines, Staff::DEFAULT_LINES);
        assert_eq!(staff.height(), 4. * Staff::DEFAULT_SPACE_SIZE);
    }

    #[test]
    fn staff_lines_is_read_from_staff_details() {
        for (declared, expected) in [(1, 1), (4, 4), (6, 6), (0, 0)] {
            let score = engrave(&score_xml(
                &format!("<staff-details><staff-lines>{declared}</staff-lines></staff-details>"),
                &note("C", 5),
            ));

            assert_eq!(staff(&score).lines, expected, "<staff-lines>{declared}");
        }
    }

    /// A staff drawn with its full five lines or more is exactly as tall as the
    /// distance between them; a staff drawn with fewer is still a full staff and
    /// occupies a standard staff's height, only some lines undrawn. A staff of
    /// no lines at all is nothing and takes no room.
    #[test]
    fn a_short_staff_still_occupies_a_standard_staffs_height() {
        let standard = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;

        for (lines, expected) in [
            (0, 0.),
            (1, standard),
            (2, standard),
            (4, standard),
            (5, standard),
            (6, 5. * Staff::DEFAULT_SPACE_SIZE),
        ] {
            let score = engrave(&score_xml(
                &format!("<staff-details><staff-lines>{lines}</staff-lines></staff-details>"),
                &note("C", 5),
            ));

            assert_eq!(
                staff(&score).height(),
                expected,
                "<staff-lines>{lines}</staff-lines>",
            );
        }
    }

    /// The height is what stacks: a five-line staff below a one-line percussion
    /// staff sits a full standard staff plus the staff distance below the
    /// percussion line, not right underneath it.
    #[test]
    fn a_staff_below_a_one_line_staff_gets_room_for_its_neighbour() {
        let score = engrave(&two_staff_score_xml(
            "<staff-details number=\"1\"><staff-lines>1</staff-lines></staff-details>",
        ));

        let staves = staves(&score);
        assert_eq!(staves[0].1.lines, 1, "staff 1 was declared one line");

        let gap = staves[1].1.xy.y - staves[0].1.xy.y;
        let standard = Staff::SPACES as f32 * Staff::DEFAULT_SPACE_SIZE;
        assert_eq!(
            gap,
            standard + staves[1].1.distance_final,
            "the lower staff clears the percussion staff's height",
        );
    }

    /// `<staff-details>` names the staff it describes, and a part's other staves
    /// keep the default.
    #[test]
    fn staff_lines_reaches_only_the_staff_it_names() {
        let score = engrave(&two_staff_score_xml(
            "<staff-details number=\"2\"><staff-lines>1</staff-lines></staff-details>",
        ));

        let staves = staves(&score);
        assert_eq!(staves.len(), 2, "expected two staves");
        assert_eq!(staves[0].1.lines, 5, "staff 1 was not described");
        assert_eq!(staves[1].1.lines, 1, "staff 2 was declared one line");
    }

    #[test]
    fn the_renderer_draws_one_line_per_line_the_staff_has() {
        for lines in [0, 1, 4, 5, 6] {
            let staff = Staff {
                lines,
                width: 100.,
                ..Default::default()
            };

            let drawn = rendered_lines(&staff);
            assert_eq!(drawn.len(), lines, "a staff of {lines} lines");
        }
    }

    #[test]
    fn the_rendered_lines_run_downwards_from_the_top_line_a_space_apart() {
        let staff = Staff {
            lines: 6,
            width: 100.,
            ..Default::default()
        };

        let drawn = rendered_lines(&staff);
        let ys: Vec<f32> = drawn.iter().map(|line| line.start.y).collect();

        assert_eq!(ys, vec![0., 10., 20., 30., 40., 50.]);
        assert!(
            drawn.iter().all(|line| line.start.y == line.end.y),
            "staff lines are horizontal"
        );
    }

    /// Middle C sits one space below a five-line treble staff and so needs a
    /// ledger line -- but on a six-line staff that is where the sixth line is,
    /// and a ledger line there would be drawn straight over it.
    #[test]
    fn a_note_on_the_sixth_line_of_a_six_line_staff_needs_no_ledger() {
        let five = engrave(&score_xml("", &note("C", 4)));
        assert_eq!(ledger_count(&five), 1, "middle C below a five-line staff");

        let six = engrave(&score_xml(
            "<staff-details><staff-lines>6</staff-lines></staff-details>",
            &note("C", 4),
        ));
        assert_eq!(ledger_count(&six), 0, "middle C on the sixth line");
    }

    /// The other way round: a staff with fewer lines runs out sooner, so a note
    /// that needed no ledger on five lines needs one on three.
    #[test]
    fn a_shorter_staff_needs_ledgers_sooner() {
        // E4 is the bottom line of a five-line treble staff.
        let five = engrave(&score_xml("", &note("E", 4)));
        assert_eq!(ledger_count(&five), 0, "E4 is on the staff");

        let three = engrave(&score_xml(
            "<staff-details><staff-lines>3</staff-lines></staff-details>",
            &note("E", 4),
        ));
        // Three lines end at what was the middle line, two lines above E4.
        assert_eq!(ledger_count(&three), 2, "E4 hangs two lines below");
    }

    /// The clef a one-line percussion staff opens with is centred on that line
    /// -- which is drawn where a five-line staff's middle line would be, index 4.
    #[test]
    fn a_pitchless_clef_is_centred_on_the_staff_it_opens() {
        for (lines, expected) in [(5, 4), (1, 4), (3, 2)] {
            let score = engrave(&score_xml_with_clef(
                "<clef><sign>percussion</sign></clef>",
                &format!("<staff-details><staff-lines>{lines}</staff-lines></staff-details>"),
                &note("C", 5),
            ));

            assert_eq!(
                opening_clef(&score).clef.line,
                expected,
                "a percussion clef on {lines} lines"
            );
        }
    }

    /// A treble clef names the G it is drawn around, and that G is in the same
    /// place however many lines the staff has -- the note is placed from the top
    /// line too.
    #[test]
    fn a_pitched_clef_keeps_its_line_whatever_the_staff() {
        for lines in [5, 1, 3, 6] {
            let score = engrave(&score_xml(
                &format!("<staff-details><staff-lines>{lines}</staff-lines></staff-details>"),
                &note("C", 5),
            ));

            assert_eq!(
                opening_clef(&score).clef.line,
                6,
                "a treble clef on {lines} lines"
            );
        }
    }

    /// A staff drawn without any lines has nothing for a ledger line to extend.
    #[test]
    fn a_staff_with_no_lines_gets_no_ledgers() {
        let score = engrave(&score_xml(
            "<staff-details><staff-lines>0</staff-lines></staff-details>",
            &note("C", 3),
        ));

        assert_eq!(ledger_count(&score), 0);
        assert!(rendered_lines(staff(&score)).is_empty());
    }

    /// A one-line percussion staff draws its line where a five-line staff's
    /// middle line would be -- two spaces down -- and a measure rest sits on it,
    /// not twenty tenths below in blank space.
    #[test]
    fn a_measure_rest_on_a_one_line_staff_sits_on_the_line() {
        let score = engrave(&score_xml_with_clef(
            "<clef><sign>percussion</sign></clef>",
            "<staff-details><staff-lines>1</staff-lines></staff-details>",
            "<note><rest measure=\"yes\"/><duration>16</duration><voice>1</voice></note>",
        ));

        let staff = staff(&score);
        assert_eq!(staff.lines, 1);

        let rest = staff
            .measures
            .values()
            .next()
            .and_then(|measure| measure.rests.first())
            .expect("the staff measure has a rest");

        assert_eq!(rest.staff_line, 4, "centred where the drawn line is");

        let line_y = staff.xy.y + staff.top_line_offset();
        assert!(
            (rest.xy.y - line_y).abs() < 0.01,
            "the rest glyph origin sits on the drawn line ({} vs {})",
            rest.xy.y,
            line_y,
        );
        assert_eq!(
            staff.top_line_offset(),
            2. * Staff::DEFAULT_SPACE_SIZE,
            "the line is drawn two spaces down",
        );
    }

    /// The single line of a one-line staff, and everything pinned to it, is
    /// drawn two spaces down -- where a five-line staff's middle line sits --
    /// so the staff reads as a full staff with only its middle line inked.
    #[test]
    fn a_one_line_staff_draws_its_line_and_content_where_the_middle_line_would_be() {
        let score = engrave(&score_xml_with_clef(
            "<clef><sign>percussion</sign></clef>",
            "<staff-details><staff-lines>1</staff-lines></staff-details>",
            "<note><rest measure=\"yes\"/><duration>16</duration><voice>1</voice></note>",
        ));

        let staff = staff(&score);
        let middle = 2. * Staff::DEFAULT_SPACE_SIZE;
        assert_eq!(
            staff.height(),
            4. * Staff::DEFAULT_SPACE_SIZE,
            "a full staff tall"
        );

        let lines = rendered_lines(staff);
        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines[0].start.y - staff.xy.y,
            middle,
            "the one line is drawn on the middle",
        );

        let measure = staff.measures.values().next().unwrap();
        let clef = measure.clef_start.as_ref().expect("an opening clef");
        assert_eq!(
            clef.xy.y - staff.xy.y,
            middle,
            "the clef centres on the line",
        );

        let time = measure
            .time_signature_start
            .as_ref()
            .expect("an opening time signature");
        let num = time.num_digits().next().unwrap().1;
        let denom = time.denom_digits().next().unwrap().1;
        assert_eq!(
            num.y - staff.xy.y,
            middle - 10.,
            "numerator a space above the line"
        );
        assert_eq!(
            denom.y - staff.xy.y,
            middle + 10.,
            "denominator a space below it"
        );
    }

    /// The common case is untouched: a rest on a five-line staff is still
    /// centred on its middle line, two spaces down from the top.
    #[test]
    fn a_rest_on_a_five_line_staff_is_still_centred_on_the_middle_line() {
        let score = engrave(&score_xml(
            "",
            "<note><rest measure=\"yes\"/><duration>16</duration><voice>1</voice></note>",
        ));

        let staff = staff(&score);
        let rest = staff
            .measures
            .values()
            .next()
            .and_then(|measure| measure.rests.first())
            .expect("the staff measure has a rest");

        assert_eq!(rest.staff_line, 4);
        assert!(
            (rest.xy.y - staff.xy.y - 2. * Staff::DEFAULT_SPACE_SIZE).abs() < 0.01,
            "two spaces below the top line",
        );
    }
}
