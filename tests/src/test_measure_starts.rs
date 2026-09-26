//! The columns a system's opening clefs, key signatures and time signatures are
//! drawn against.
//!
//! Each staff used to lay these three out from its own left edge, one after the
//! other, so the position of the time signature depended on how wide *that
//! staff's* key signature happened to be. Transposing instruments make that
//! visible in any real score: a score in C flat major carries seven flats, the
//! trumpets in B flat five, and an unpitched percussion staff none, which put
//! three different time signatures down one system. The three columns are
//! settled once per measure for the whole system instead, from the widest
//! contribution any drawn staff makes.

#[cfg(test)]
mod tests {
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::{APP_DEFAULTS, MeasureStartLayout, UserLayout};
    use lib::score::visual::score::Score;
    use lib::score::visual::staff_measure::StaffMeasure;
    use lib::smufl::smufl_font::SmuflFont;
    use roxmltree::Document;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    const TREBLE: &str = "<clef><sign>G</sign><line>2</line></clef>";
    const BASS: &str = "<clef><sign>F</sign><line>4</line></clef>";
    const TREBLE_HALF_SIZE: &str = "<clef><sign>G</sign><line>2</line></clef>\
        <staff-details><staff-size>50</staff-size></staff-details>";

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

    /// A one-measure score whose parts each declare their own `<key>` and
    /// `<clef>`, the way a score with transposing instruments does. One
    /// `(fifths, clef)` pair per part.
    fn score_xml(parts: &[(i32, &str)]) -> String {
        let part_list: String = (1..=parts.len())
            .map(|id| format!("<score-part id=\"P{id}\"><part-name>P{id}</part-name></score-part>"))
            .collect();

        let bodies: String = parts
            .iter()
            .enumerate()
            .map(|(i, (fifths, clef))| {
                let id = i + 1;
                format!(
                    "<part id=\"P{id}\"><measure number=\"1\" width=\"400\">\
                     <attributes><divisions>4</divisions>\
                     <key><fifths>{fifths}</fifths><mode>major</mode></key>\
                     <time><beats>4</beats><beat-type>4</beat-type></time>\
                     {clef}</attributes>\
                     <note default-x=\"200\"><pitch><step>C</step><octave>4</octave></pitch>\
                     <duration>16</duration><voice>1</voice><type>whole</type></note>\
                     </measure></part>"
                )
            })
            .collect();

        format!(
            "<score-partwise version=\"4.0\">\
             <part-list>{part_list}</part-list>{bodies}</score-partwise>"
        )
    }

    fn engrave(xml: &str, layout: &UserLayout) -> Score {
        let document = Document::parse(xml).expect("test document does not parse");
        let (mut score, defaults, _messages) = walk_document(&document, &mut |_stage| {});

        arrange_score(&mut score, &defaults, font(), layout, &mut |_stage| {});

        score
    }

    /// Every staff measure numbered `number`, in system order.
    fn staff_measures(score: &Score, number: u32) -> Vec<&StaffMeasure> {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.values())
            .filter_map(|staff| staff.measures.get(&number))
            .collect()
    }

    fn clef_x(measure: &StaffMeasure) -> f32 {
        measure.clef_start.as_ref().expect("an opening clef").xy.x
    }

    fn clef_width(measure: &StaffMeasure) -> f32 {
        measure.clef_start.as_ref().expect("an opening clef").width
    }

    fn key_x(measure: &StaffMeasure) -> f32 {
        measure.key_signature_start.xy.x
    }

    fn time_x(measure: &StaffMeasure) -> f32 {
        measure
            .time_signature_start
            .as_ref()
            .expect("an opening time signature")
            .xy
            .x
    }

    /// The premise the whole scheme rests on: every staff measure of one measure
    /// number starts at the same x, so placing the three elements at equal
    /// offsets really does line them up on the page.
    #[test]
    fn staff_measures_of_one_measure_share_a_left_edge() {
        let score = engrave(
            &score_xml(&[(-7, TREBLE), (-5, TREBLE), (0, TREBLE)]),
            &UserLayout::default(),
        );
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 3);

        for measure in &measures {
            assert_eq!(measure.xy.x, measures[0].xy.x);
        }
    }

    /// The bug this exists for: seven flats, five flats and none, all sharing a
    /// system. Every clef, every key signature and every time signature starts
    /// at the same place.
    #[test]
    fn unequal_key_signatures_do_not_move_the_time_signature() {
        let score = engrave(
            &score_xml(&[(-7, TREBLE), (-5, TREBLE), (0, TREBLE)]),
            &UserLayout::default(),
        );
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 3);

        // The premise: the three key signatures really are different widths, so
        // laying each staff out on its own would have put them out of line.
        let widths: Vec<f32> = measures
            .iter()
            .map(|m| m.key_signature_start.width)
            .collect();
        assert!(
            widths[0] > widths[1] && widths[1] > widths[2],
            "expected three different key signature widths, got {widths:?}"
        );
        assert_eq!(widths[2], 0., "a part in C carries no accidentals");

        for measure in &measures {
            assert_eq!(clef_x(measure), clef_x(measures[0]), "clefs align");
            assert_eq!(key_x(measure), key_x(measures[0]), "key signatures align");
            assert_eq!(
                time_x(measure),
                time_x(measures[0]),
                "time signatures align"
            );
        }

        // And the columns really are in reading order, each clear of the last.
        assert!(clef_x(measures[0]) < key_x(measures[0]));
        assert!(key_x(measures[0]) + widths[0] < time_x(measures[0]));
    }

    /// The cascade: the key column is measured from the *widest* clef, not from
    /// each staff's own. A staff with the narrower clef would have started its
    /// key signature closer in, and that answer must not survive the clef column
    /// moving right for somebody else.
    #[test]
    fn key_signatures_clear_the_widest_clef() {
        let score = engrave(
            &score_xml(&[(-3, TREBLE), (-3, BASS)]),
            &UserLayout::default(),
        );
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 2);

        let widths: Vec<f32> = measures.iter().map(|m| clef_width(m)).collect();
        assert!(
            widths[0] != widths[1],
            "expected a treble and a bass clef to differ in width, got {widths:?}"
        );

        assert_eq!(
            key_x(measures[0]),
            key_x(measures[1]),
            "key signatures align"
        );

        let widest = widths[0].max(widths[1]);
        let padding = APP_DEFAULTS.measure_start.key_signature_padding * measures[0].scale;
        assert_eq!(
            key_x(measures[0]),
            clef_x(measures[0]) + widest + padding,
            "a padding clear of the wider clef"
        );
    }

    /// A staff on its own is laid out exactly as it was before there were shared
    /// columns: padding, clef, padding, key signature, padding, time signature.
    #[test]
    fn a_single_staff_still_follows_its_own_spacing() {
        let score = engrave(&score_xml(&[(-3, TREBLE)]), &UserLayout::default());
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 1);

        let measure = measures[0];
        let defaults = &APP_DEFAULTS.measure_start;
        let clef_padding = defaults.clef_padding * measure.scale;
        let key_padding = defaults.key_signature_padding * measure.scale;
        let time_padding = defaults.time_signature_padding * measure.scale;

        assert_eq!(clef_x(measure) - measure.xy.x, clef_padding);
        assert_eq!(
            key_x(measure),
            clef_x(measure) + clef_width(measure) + key_padding
        );
        assert_eq!(
            time_x(measure),
            key_x(measure) + measure.key_signature_start.width + time_padding
        );
    }

    /// The point of the split: three independent knobs, each moving only the
    /// column it names.
    #[test]
    fn each_padding_moves_only_its_own_column() {
        let layout = UserLayout {
            measure_start: MeasureStartLayout {
                clef_padding: Some(3.),
                key_signature_padding: Some(11.),
                time_signature_padding: Some(23.),
            },
            ..UserLayout::default()
        };
        let score = engrave(&score_xml(&[(-3, TREBLE)]), &layout);
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 1);

        let measure = measures[0];
        assert_eq!(clef_x(measure) - measure.xy.x, 3.);
        assert_eq!(key_x(measure), clef_x(measure) + clef_width(measure) + 11.);
        assert_eq!(
            time_x(measure),
            key_x(measure) + measure.key_signature_start.width + 23.
        );
    }

    /// Before the split, opening the clef away from the barline would also have
    /// pushed the key signature off the clef -- they shared one constant. Now
    /// raising only the clef padding shifts every column right by the same
    /// amount, leaving the gaps between them untouched.
    #[test]
    fn clef_padding_does_not_move_the_key_signature_off_the_clef() {
        let default_score = engrave(&score_xml(&[(-3, TREBLE)]), &UserLayout::default());
        let default_measures = staff_measures(&default_score, 1);
        let default_measure = default_measures[0];
        let default_clef_to_key =
            key_x(default_measure) - clef_x(default_measure) - clef_width(default_measure);
        let default_key_to_time = time_x(default_measure)
            - key_x(default_measure)
            - default_measure.key_signature_start.width;

        let raised_layout = UserLayout {
            measure_start: MeasureStartLayout {
                clef_padding: Some(25.),
                ..Default::default()
            },
            ..UserLayout::default()
        };
        let raised_score = engrave(&score_xml(&[(-3, TREBLE)]), &raised_layout);
        let raised_measures = staff_measures(&raised_score, 1);
        let raised_measure = raised_measures[0];

        assert_eq!(clef_x(raised_measure) - raised_measure.xy.x, 25.);
        assert_eq!(
            key_x(raised_measure) - clef_x(raised_measure) - clef_width(raised_measure),
            default_clef_to_key,
            "the clef-to-key gap is untouched"
        );
        assert_eq!(
            time_x(raised_measure)
                - key_x(raised_measure)
                - raised_measure.key_signature_start.width,
            default_key_to_time,
            "the key-to-time gap is untouched"
        );
    }

    /// Each staff's contribution to a shared column is its own padding scaled
    /// by its own `scale` -- and the widest contribution still wins the column,
    /// so a reduced staff opens at the same offset as its full-size neighbour
    /// rather than at its own smaller one.
    #[test]
    fn paddings_scale_with_the_staff() {
        let layout = UserLayout {
            measure_start: MeasureStartLayout {
                clef_padding: Some(20.),
                ..Default::default()
            },
            ..UserLayout::default()
        };

        let solo = engrave(&score_xml(&[(0, TREBLE_HALF_SIZE)]), &layout);
        let solo_measures = staff_measures(&solo, 1);
        assert_eq!(solo_measures.len(), 1);
        assert_eq!(solo_measures[0].scale, 0.5, "the part drew at half size");
        assert_eq!(
            clef_x(solo_measures[0]) - solo_measures[0].xy.x,
            10.,
            "alone, the reduced staff only asks for half the padding"
        );

        let score = engrave(&score_xml(&[(0, TREBLE), (0, TREBLE_HALF_SIZE)]), &layout);
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 2);
        assert_eq!(measures[1].scale, 0.5);

        assert_eq!(clef_x(measures[0]) - measures[0].xy.x, 20.);
        assert_eq!(
            clef_x(measures[1]) - measures[1].xy.x,
            clef_x(measures[0]) - measures[0].xy.x,
            "the full-size staff's wider contribution wins the shared column"
        );
    }
}
