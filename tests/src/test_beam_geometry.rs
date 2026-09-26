//! The beam geometry rules that used to be constants: how long a hook is drawn.
//!
//! Pure functions over chords placed by hand, so each rule is checked in tenths
//! without an arranged score.

#[cfg(test)]
mod tests {
    use lib::geometry::xy::XY;
    use lib::score::core::duration_base::BaseDuration;
    use lib::score::core::note_kind::NoteKind;
    use lib::score::visual::beam::hook_length;
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
}
