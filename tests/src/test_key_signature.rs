//! How a key signature is drawn, as opposed to what it says -- `test_key`
//! covers the reading of `<fifths>` itself.
//!
//! A signature belongs to its staff and is written in that staff's spaces, so
//! everything about it follows the staff's scale: the accidental glyphs, the
//! gaps between them, and the half-spaces that put each one on its own line. A
//! signature that ignored the scale would be drawn full size on a reduced staff,
//! its accidentals spread across lines that are no longer where they are.

#[cfg(test)]
mod tests {
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::UserLayout;
    use lib::score::visual::key_signature::KeySignature;
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
            SmuflFont::load(&asset(
                "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json",
            ))
        })
    }

    /// A two-part score in four flats, the second part's staff drawn at
    /// `staff_size` percent of the first's. Both parts read from the same clef
    /// and the same key, so their signatures differ in nothing but scale.
    fn score_xml(staff_size: u32) -> String {
        let part = |id: u32, details: String| {
            format!(
                "<part id=\"P{id}\"><measure number=\"1\" width=\"400\">\
                 <attributes><divisions>4</divisions>\
                 <key><fifths>-4</fifths><mode>major</mode></key>\
                 <time><beats>4</beats><beat-type>4</beat-type></time>\
                 <clef><sign>G</sign><line>2</line></clef>{details}</attributes>\
                 <note default-x=\"200\"><pitch><step>C</step><octave>5</octave></pitch>\
                 <duration>16</duration><voice>1</voice><type>whole</type></note>\
                 </measure></part>"
            )
        };

        format!(
            "<score-partwise version=\"4.0\">\
             <part-list>\
             <score-part id=\"P1\"><part-name>P1</part-name></score-part>\
             <score-part id=\"P2\"><part-name>P2</part-name></score-part>\
             </part-list>\
             {}{}</score-partwise>",
            part(1, String::new()),
            part(
                2,
                format!("<staff-details><staff-size>{staff_size}</staff-size></staff-details>")
            ),
        )
    }

    fn engrave(xml: &str) -> Score {
        let document = Document::parse(xml).expect("test document does not parse");
        let (mut score, defaults, _messages) = walk_document(&document, &mut |_stage| {});

        arrange_score(
            &mut score,
            &defaults,
            font(),
            &UserLayout::default(),
            &mut |_stage| {},
        );

        score
    }

    /// Every staff of the score with the opening key signature it draws.
    fn signatures(score: &Score) -> Vec<(&Staff, &KeySignature)> {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.values())
            .map(|staff| {
                let measure = staff.measures.values().next().expect("a first measure");
                (staff, &measure.key_signature_start)
            })
            .collect()
    }

    /// Everything the signature is made of shrinks by the staff's own factor:
    /// the glyphs, the total width, and the gaps between the accidentals -- the
    /// last of which is what the width proves, since it is the only thing the
    /// spacing contributes to.
    #[test]
    fn a_key_signature_is_drawn_at_its_staffs_scale() {
        let score = engrave(&score_xml(50));
        let signatures = signatures(&score);
        assert_eq!(signatures.len(), 2);

        let (full_staff, full) = signatures[0];
        let (half_staff, half) = signatures[1];

        assert_eq!(full_staff.scale, 1.);
        assert_eq!(half_staff.scale, 0.5);
        assert_eq!(full.accidentals.len(), 4, "four flats");
        assert_eq!(half.accidentals.len(), 4);

        assert_eq!(
            half.scale, half_staff.scale,
            "the signature takes the scale"
        );
        assert!(
            half.accidentals.iter().all(|(_, acc)| acc.scale == 0.5),
            "and hands it to every accidental"
        );

        for ((_, full_acc), (_, half_acc)) in full.accidentals.iter().zip(half.accidentals.iter()) {
            assert!(full_acc.width > 0.);
            assert!(
                (half_acc.width - full_acc.width / 2.).abs() < 1e-4,
                "a half-size flat is half as wide"
            );
        }

        assert!(
            (half.width - full.width / 2.).abs() < 1e-4,
            "so the whole signature is half as wide, spacing included: \
             {} against {}",
            half.width,
            full.width
        );
    }

    /// The accidentals also have to land on the reduced staff's own lines. They
    /// are placed in half-spaces from the top of the staff, and a reduced staff's
    /// spaces are smaller, so the offsets scale with everything else.
    #[test]
    fn accidentals_sit_on_the_lines_of_the_staff_they_belong_to() {
        let score = engrave(&score_xml(50));
        let signatures = signatures(&score);

        let (_, full) = signatures[0];
        let (_, half) = signatures[1];

        let offsets = |signature: &KeySignature| -> Vec<f32> {
            signature
                .accidentals
                .iter()
                .map(|(_, acc)| acc.xy.y - signature.xy.y)
                .collect()
        };

        let full_offsets = offsets(full);
        let half_offsets = offsets(half);

        assert!(
            full_offsets.iter().any(|dy| *dy != 0.),
            "the flats of four flats are not all on the same line"
        );

        for (full_dy, half_dy) in full_offsets.iter().zip(half_offsets.iter()) {
            assert!(
                (half_dy - full_dy / 2.).abs() < 1e-4,
                "a half-size staff's lines are half as far apart: \
                 {half_dy} against {full_dy}"
            );
        }
    }
}
