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
    use lib::score::app_defaults::AppDefaults;
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::score::Score;
    use lib::score::visual::staff_measure::StaffMeasure;
    use lib::score::visual::start_columns::{StartColumns, StartMetrics};
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

    fn metrics(padding: f32, clef_width: Option<f32>, key_width: f32) -> StartMetrics {
        StartMetrics {
            padding,
            clef_width,
            key_width,
        }
    }

    /// A score whose parts each declare their own `<key>`, the way a score with
    /// transposing instruments does. `keys` is one `<fifths>` per part.
    fn score_xml(keys: &[i32]) -> String {
        let part_list: String = keys
            .iter()
            .enumerate()
            .map(|(i, _)| {
                let id = i + 1;
                format!("<score-part id=\"P{id}\"><part-name>P{id}</part-name></score-part>")
            })
            .collect();

        let parts: String = keys
            .iter()
            .enumerate()
            .map(|(i, fifths)| {
                let id = i + 1;
                format!(
                    "<part id=\"P{id}\"><measure number=\"1\" width=\"400\">\
                     <attributes><divisions>4</divisions>\
                     <key><fifths>{fifths}</fifths><mode>major</mode></key>\
                     <time><beats>4</beats><beat-type>4</beat-type></time>\
                     <clef><sign>G</sign><line>2</line></clef></attributes>\
                     <note default-x=\"200\"><pitch><step>C</step><octave>5</octave></pitch>\
                     <duration>16</duration><voice>1</voice><type>whole</type></note>\
                     </measure></part>"
                )
            })
            .collect();

        format!(
            "<score-partwise version=\"4.0\">\
             <part-list>{part_list}</part-list>{parts}</score-partwise>"
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
            font(),
            &UserLayout::default(),
            &AppDefaults::default(),
            &mut |_stage| {},
        );

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

    /// The columns are offsets from a measure's own left edge, and every staff
    /// measure of one measure number shares that edge. Asserted separately
    /// because it is the premise the whole scheme rests on: without it, equal
    /// offsets would not put the elements in the same place on the page.
    #[test]
    fn staff_measures_of_one_measure_share_a_left_edge() {
        let score = engrave(&score_xml(&[-7, -5, 0]));
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 3);

        let first = measures[0].xy.x;
        for measure in &measures {
            assert_eq!(measure.xy.x, first);
        }
    }

    /// The bug this exists for: seven flats, five flats and none, all sharing a
    /// system. Every clef, every key signature and every time signature starts
    /// at the same place.
    #[test]
    fn unequal_key_signatures_do_not_move_the_time_signature() {
        let score = engrave(&score_xml(&[-7, -5, 0]));
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

        let clef_x = |m: &StaffMeasure| m.clef_start.as_ref().expect("an opening clef").xy.x;
        let key_x = |m: &StaffMeasure| m.key_signature_start.xy.x;
        let time_x = |m: &StaffMeasure| {
            m.time_signature_start
                .as_ref()
                .expect("an opening time signature")
                .xy
                .x
        };

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

    /// A staff on its own is laid out exactly as it was before there were shared
    /// columns: padding, clef, padding, key signature, padding, time signature.
    #[test]
    fn a_single_staff_still_follows_its_own_spacing() {
        let score = engrave(&score_xml(&[-3]));
        let measures = staff_measures(&score, 1);
        assert_eq!(measures.len(), 1);

        let measure = measures[0];
        let padding = measure.padding();
        let clef = measure.clef_start.as_ref().expect("an opening clef");
        let time = measure
            .time_signature_start
            .as_ref()
            .expect("an opening time signature");

        assert_eq!(clef.xy.x - measure.xy.x, padding);
        assert_eq!(
            measure.key_signature_start.xy.x,
            clef.xy.x + clef.width + padding
        );
        assert_eq!(
            time.xy.x,
            measure.key_signature_start.xy.x + measure.key_signature_start.width + padding
        );
    }

    /// One staff's contribution is what it would have done alone.
    #[test]
    fn one_staff_folds_to_its_own_spacing() {
        let columns = StartColumns::fold(&[metrics(5., Some(20.), 30.)]);

        assert_eq!(columns.clef, 5.);
        assert_eq!(columns.key, 5. + 20. + 5.);
        assert_eq!(columns.time, columns.key + 30. + 5.);
    }

    /// The key column has to be re-derived from the *shared* clef column, not
    /// taken as the widest of what each staff would have decided alone. A staff
    /// with a narrow clef computes a near key column, and that answer does not
    /// survive the clef column moving right for somebody else's wider clef.
    #[test]
    fn the_key_column_clears_the_widest_clef() {
        let columns = StartColumns::fold(&[metrics(5., Some(10.), 0.), metrics(5., Some(40.), 0.)]);

        assert_eq!(columns.key, 5. + 40. + 5., "cleared the wider clef");
    }

    /// The same cascade one column further out: the time column is measured from
    /// the shared key column plus the widest key signature, even when the staff
    /// carrying that key signature is not the one with the widest clef.
    #[test]
    fn the_time_column_clears_the_widest_key_signature() {
        let columns =
            StartColumns::fold(&[metrics(5., Some(40.), 0.), metrics(5., Some(10.), 70.)]);

        assert_eq!(columns.key, 5. + 40. + 5.);
        assert_eq!(columns.time, columns.key + 70. + 5.);
    }

    /// A staff that opens without a clef starts its key signature a padding in
    /// from the measure's edge, and so has nothing to say about where the clef
    /// column sits -- but it is still held to the shared key column.
    #[test]
    fn a_staff_without_a_clef_does_not_widen_the_key_column() {
        let with_clef = StartColumns::fold(&[metrics(5., Some(40.), 0.)]);
        let mixed = StartColumns::fold(&[metrics(5., Some(40.), 0.), metrics(5., None, 0.)]);

        assert_eq!(mixed, with_clef);
    }

    /// Padding scales with the staff, and the column has to satisfy the most
    /// demanding one.
    #[test]
    fn the_widest_padding_wins() {
        let columns = StartColumns::fold(&[metrics(2.5, None, 0.), metrics(5., None, 0.)]);

        assert_eq!(columns.clef, 5.);
        assert_eq!(columns.key, 5.);
    }

    /// Nothing to fold is not a special case: a measure with no staves at all
    /// puts its columns at the measure's own edge.
    #[test]
    fn no_staves_fold_to_no_offsets() {
        assert_eq!(StartColumns::fold(&[]), StartColumns::default());
    }
}
