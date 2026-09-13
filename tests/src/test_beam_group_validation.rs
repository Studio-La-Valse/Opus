#[cfg(test)]
mod tests {
    use lib::musicxml::validate::ValidationCtx;
    use lib::musicxml::validation_issue::Severity;
    use lib::musicxml::visitor::{DefaultVisitor, Visitor};
    use lib::musicxml::visitors::validators::beam_group_visitor::BeamGroupVisitor;
    use lib::musicxml::walker::Walker;
    use roxmltree::Document;

    /// Wraps bare `<note>` markup in the smallest document the walker will
    /// descend into. `measures` is a list of measure bodies.
    fn score(measures: &[&str]) -> String {
        let body: String = measures
            .iter()
            .enumerate()
            .map(|(i, m)| format!("<measure number=\"{}\">{m}</measure>", i + 1))
            .collect();

        format!(
            "<score-partwise version=\"4.0\">\
             <part-list><score-part id=\"P1\"/></part-list>\
             <part id=\"P1\">{body}</part>\
             </score-partwise>"
        )
    }

    fn note(voice: u32, beams: &str) -> String {
        format!(
            "<note><pitch><step>C</step><octave>5</octave></pitch><duration>1</duration>\
             <voice>{voice}</voice><type>16th</type><stem>up</stem>{beams}</note>"
        )
    }

    fn beam(number: u32, kind: &str) -> String {
        format!("<beam number=\"{number}\">{kind}</beam>")
    }

    /// Runs the visitor over `measures` and returns the warning messages, in
    /// order.
    fn warnings(measures: &[&str]) -> Vec<String> {
        let xml = score(measures);
        let document = Document::parse(&xml).expect("test document does not parse");

        let mut ctx = ValidationCtx::default();
        Walker::new(DefaultVisitor {}.uses(BeamGroupVisitor::default())).walk(&document, &mut ctx);

        ctx.issues
            .iter()
            .inspect(|issue| {
                assert_eq!(
                    issue.severity,
                    Severity::Warning,
                    "an unclosed beam is repairable, so it is never an error"
                );
            })
            .map(|issue| issue.message.clone())
            .collect()
    }

    #[test]
    fn a_well_formed_beam_group_is_quiet() {
        let m = format!(
            "{}{}{}",
            note(1, &format!("{}{}", beam(1, "begin"), beam(2, "begin"))),
            note(
                1,
                &format!("{}{}", beam(1, "continue"), beam(2, "continue"))
            ),
            note(1, &format!("{}{}", beam(1, "end"), beam(2, "end"))),
        );

        assert!(warnings(&[&m]).is_empty());
    }

    /// The case that used to panic in `create_beams`: every note declares two
    /// beams, so the rebeam pass sees a consistent count and leaves it alone,
    /// but level 2 never ends.
    #[test]
    fn a_secondary_level_that_never_ends_warns() {
        let m = format!(
            "{}{}{}",
            note(1, &format!("{}{}", beam(1, "begin"), beam(2, "begin"))),
            note(
                1,
                &format!("{}{}", beam(1, "continue"), beam(2, "continue"))
            ),
            note(1, &format!("{}{}", beam(1, "end"), beam(2, "continue"))),
        );

        let found = warnings(&[&m]);
        assert_eq!(found.len(), 1, "expected one warning, got {found:?}");
        assert!(found[0].contains("number=\"2\""), "{}", found[0]);
    }

    /// The group itself running off the end of the part is the same defect a
    /// level up, and panicked in the same place.
    #[test]
    fn a_group_that_never_ends_warns() {
        let m = format!(
            "{}{}",
            note(1, &beam(1, "begin")),
            note(1, &beam(1, "continue")),
        );

        let found = warnings(&[&m]);
        assert_eq!(found.len(), 1, "expected one warning, got {found:?}");
        assert!(found[0].contains("number=\"1\""), "{}", found[0]);
    }

    /// Only the first note of a chord spells the chord's beams; the rest carry
    /// `<chord/>` and nothing else. Reading those as beamless notes would close
    /// every group that contains a chord -- which is most real music.
    #[test]
    fn chord_members_do_not_close_a_group() {
        let chord_member = "<note><chord/><pitch><step>E</step><octave>5</octave></pitch>\
                            <duration>1</duration><voice>1</voice><type>16th</type></note>";
        let m = format!(
            "{}{}{}",
            note(1, &beam(1, "begin")),
            chord_member,
            note(1, &beam(1, "end")),
        );

        assert!(warnings(&[&m]).is_empty());
    }

    /// A rest never becomes a chord, so it is not an element of the run the
    /// renderer beams either.
    #[test]
    fn a_rest_does_not_close_a_group() {
        let rest = "<note><rest/><duration>1</duration><voice>1</voice><type>16th</type></note>";
        let m = format!(
            "{}{}{}",
            note(1, &beam(1, "begin")),
            rest,
            note(1, &beam(1, "end")),
        );

        assert!(warnings(&[&m]).is_empty());
    }

    /// A note carrying no beams at all ends the group it lands in, so anything
    /// still open above level 1 is stranded.
    #[test]
    fn a_beamless_note_closes_the_group() {
        let m = format!(
            "{}{}",
            note(1, &format!("{}{}", beam(1, "begin"), beam(2, "begin"))),
            note(1, ""),
        );

        let found = warnings(&[&m]);
        assert_eq!(found.len(), 2, "both levels are stranded, got {found:?}");
    }

    /// Voices are beamed separately, so one voice's open level is not closed by
    /// another voice's notes.
    #[test]
    fn voices_are_tracked_separately() {
        let m = format!(
            "{}{}{}{}",
            note(1, &beam(1, "begin")),
            note(2, &beam(1, "begin")),
            note(2, &beam(1, "end")),
            note(1, &beam(1, "end")),
        );

        assert!(warnings(&[&m]).is_empty());
    }

    /// A beam group may span a barline: `BeamArranger` builds its runs from a
    /// whole part, so a level opened at the end of one measure and closed at the
    /// start of the next is well formed, not stranded.
    #[test]
    fn a_group_continues_across_a_barline() {
        let first = note(1, &beam(1, "begin"));
        let second = note(1, &beam(1, "end"));

        assert!(warnings(&[&first, &second]).is_empty());
    }

    /// The barline is not a boundary, so a level left open at the *end of the
    /// part* is what is finally stranded.
    #[test]
    fn a_group_left_open_at_the_end_of_the_part_warns() {
        let first = note(1, &beam(1, "begin"));
        let second = note(1, &beam(1, "continue"));

        let found = warnings(&[&first, &second]);
        assert_eq!(found.len(), 1, "expected one warning, got {found:?}");
    }

    /// Re-opening a level that is already open leaves the first one
    /// unaccounted for.
    #[test]
    fn reopening_a_level_warns_for_the_first_one() {
        let m = format!(
            "{}{}{}",
            note(1, &beam(1, "begin")),
            note(1, &beam(1, "begin")),
            note(1, &beam(1, "end")),
        );

        let found = warnings(&[&m]);
        assert_eq!(found.len(), 1, "expected one warning, got {found:?}");
    }
}
