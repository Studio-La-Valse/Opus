//! The beam geometry rules that used to be constants: how long a hook is drawn,
//! and how far a beam slants.
//!
//! Pure functions over stems and anchors placed by hand, so each rule is checked
//! in tenths without an arranged score.

#[cfg(test)]
mod tests {
    use lib::geometry::ray::Ray;
    use lib::geometry::xy::XY;
    use lib::score::core::duration_base::BaseDuration;
    use lib::score::core::note_kind::NoteKind;
    use lib::score::visual::beam::{BeamAnchor, fit_beam, hook_length};
    use lib::score::visual::chord::Chord;
    use lib::score::visual::note_scale::NoteScale;
    use lib::score::visual::stem::{Stem, UpDown};

    const MAX: f32 = 10.;

    /// A chord whose stem stands at `x`. Nothing else matters to the hook.
    fn stem_at(x: f32) -> Chord {
        let mut stem = Stem::new(
            UpDown::Up,
            BaseDuration::Sixteenth,
            0.into(),
            NoteScale::new(1., NoteKind::Normal),
            None,
        );
        stem.xy = XY { x, y: 0. };

        Chord {
            stem: Some(stem),
            ..Default::default()
        }
    }

    fn stemless() -> Chord {
        Chord::default()
    }

    fn hook(chords: &mut [Chord], at: usize, forward: bool) -> f32 {
        let borrowed: Vec<&mut Chord> = chords.iter_mut().collect();
        hook_length(&borrowed, at, forward, MAX)
    }

    #[test]
    fn a_hook_with_room_to_spare_is_drawn_at_full_length() {
        let mut chords = [stem_at(0.), stem_at(40.)];

        assert_eq!(hook(&mut chords, 0, true), MAX);
    }

    /// Half the gap, so a hook pointing back from the neighbour could never meet
    /// it and read as a beam.
    #[test]
    fn a_hook_between_close_stems_takes_half_the_gap() {
        let mut chords = [stem_at(0.), stem_at(12.)];

        assert_eq!(hook(&mut chords, 0, true), 6.);
    }

    #[test]
    fn a_backward_hook_measures_against_the_stem_before_it() {
        let mut chords = [stem_at(0.), stem_at(100.), stem_at(108.)];

        assert_eq!(hook(&mut chords, 2, false), 4.);
    }

    /// A chord without a stem carries no beam, so the hook reaches towards the
    /// next chord that does.
    #[test]
    fn a_stemless_neighbour_is_skipped() {
        let mut chords = [stem_at(0.), stemless(), stem_at(16.)];

        assert_eq!(hook(&mut chords, 0, true), 8.);
    }

    #[test]
    fn a_hook_with_no_neighbour_on_its_side_is_drawn_at_full_length() {
        let mut chords = [stem_at(0.), stem_at(4.)];

        assert_eq!(hook(&mut chords, 1, true), MAX);
        assert_eq!(hook(&mut chords, 0, false), MAX);
    }

    // ------------------------------------------------------------------ slant

    /// One staff space; a staff step is half of it.
    const SPACE: f32 = 10.;

    /// A natural stem of three and a half spaces.
    const STEM: f32 = 35.;

    /// A stem-up anchor at `x` whose beam-side note sits at `note_y`, reaching a
    /// natural stem length above it.
    fn up(x: f32, note_y: f32) -> BeamAnchor {
        BeamAnchor {
            x,
            note_y,
            reach_y: note_y - STEM,
        }
    }

    fn down(x: f32, note_y: f32) -> BeamAnchor {
        BeamAnchor {
            x,
            note_y,
            reach_y: note_y + STEM,
        }
    }

    fn y_at(ray: &Ray, x: f32) -> f32 {
        ray.origin.y + (x - ray.origin.x) * ray.dir.y / ray.dir.x
    }

    /// The vertical distance the beam covers between the outer anchors: negative
    /// rising, positive falling.
    fn slant(anchors: &[BeamAnchor], stems: UpDown, max_scale: f32) -> f32 {
        let ray = fit_beam(anchors, stems, SPACE, max_scale).expect("anchors given");
        let first = anchors.first().unwrap().x;
        let last = anchors.last().unwrap().x;

        y_at(&ray, last) - y_at(&ray, first)
    }

    fn assert_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "expected {expected}, got {actual}"
        );
    }

    #[test]
    fn outer_notes_on_the_same_position_give_a_flat_beam() {
        let anchors = [up(0., 50.), up(20., 30.), up(40., 50.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), 0.);
    }

    /// An inner note nearer the beam than both ends: a beam following the ends
    /// would cut into its stem, so it goes flat instead.
    #[test]
    fn an_inner_note_nearer_the_beam_than_both_ends_gives_a_flat_beam() {
        let anchors = [up(0., 50.), up(20., 20.), up(40., 40.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), 0.);
    }

    #[test]
    fn a_second_slants_a_quarter_space() {
        let anchors = [up(0., 50.), up(100., 45.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), -2.5);
    }

    #[test]
    fn a_third_slants_half_a_space() {
        let anchors = [up(0., 50.), up(100., 40.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), -5.);
    }

    #[test]
    fn a_falling_interval_slants_the_beam_down() {
        let anchors = [up(0., 40.), up(100., 50.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), 5.);
    }

    /// Two octaves would be three and a half spaces; a two-note group stops at
    /// two.
    #[test]
    fn a_two_note_group_slants_at_most_two_spaces() {
        let anchors = [up(0., 80.), up(200., 10.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), -20.);
    }

    #[test]
    fn a_group_of_three_or_more_slants_at_most_one_space() {
        let anchors = [up(0., 50.), up(100., 40.), up(200., 15.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), -10.);
    }

    /// A fifth wants a whole space, but two stems two spaces apart allow only a
    /// quarter of that distance.
    #[test]
    fn close_spacing_caps_the_slant_at_a_quarter_of_the_span() {
        let anchors = [up(0., 50.), up(20., 30.)];

        assert_close(slant(&anchors, UpDown::Up, 1.), -5.);
    }

    #[test]
    fn a_grace_group_scales_its_caps() {
        let anchors = [up(0., 80.), up(200., 10.)];

        assert_close(slant(&anchors, UpDown::Up, 0.5), -10.);
    }

    #[test]
    fn stems_down_follow_the_same_rule_below_the_notes() {
        let anchors = [down(0., 50.), down(100., 60.)];

        assert_close(slant(&anchors, UpDown::Down, 1.), 5.);
    }

    /// The middle stem carries more levels and so reaches further than the
    /// outer ones: the beam keeps the slant the notes asked for and is lifted as
    /// a whole to clear it.
    #[test]
    fn the_beam_is_pushed_out_to_clear_every_stem() {
        let mut anchors = [up(0., 50.), up(50., 45.), up(100., 40.)];
        anchors[1].reach_y -= 15.;

        let ray = fit_beam(&anchors, UpDown::Up, SPACE, 1.).unwrap();

        for anchor in &anchors {
            assert!(
                y_at(&ray, anchor.x) <= anchor.reach_y + 1e-3,
                "the stem at {} is shorter than its natural length",
                anchor.x
            );
        }
        assert_close(y_at(&ray, 50.), anchors[1].reach_y);
        assert_close(y_at(&ray, 100.) - y_at(&ray, 0.), -5.);
    }

    #[test]
    fn stems_down_are_pushed_out_below_every_reach() {
        let mut anchors = [down(0., 50.), down(50., 55.), down(100., 60.)];
        anchors[1].reach_y += 15.;

        let ray = fit_beam(&anchors, UpDown::Down, SPACE, 1.).unwrap();

        for anchor in &anchors {
            assert!(y_at(&ray, anchor.x) >= anchor.reach_y - 1e-3);
        }
        assert_close(y_at(&ray, 50.), anchors[1].reach_y);
    }

    #[test]
    fn a_single_stem_gets_a_level_beam_at_its_reach() {
        let anchors = [up(0., 50.)];

        let ray = fit_beam(&anchors, UpDown::Up, SPACE, 1.).unwrap();

        assert_close(ray.dir.y, 0.);
        assert_close(ray.origin.y, 15.);
    }

    #[test]
    fn no_anchors_give_no_beam() {
        assert!(fit_beam(&[], UpDown::Up, SPACE, 1.).is_none());
    }
}
