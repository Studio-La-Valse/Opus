#[cfg(test)]
mod tests {
    use lib::score::core::key::{Key, Mode};
    use lib::score::core::step::Step;

    fn step(letter: &str, alter: i32) -> Step {
        Step::parse(letter, alter)
    }

    /// `<fifths>` is the printed accidental count, whatever the mode says, so it
    /// has to survive a round trip through the tonic untouched. This is the
    /// property the whole type exists to protect: a minor key must not have its
    /// mode applied twice.
    #[test]
    fn fifths_survives_a_round_trip_through_the_tonic() {
        for fifths in -7i8..=7 {
            for mode in [Mode::Major, Mode::Minor] {
                let key = Key::from_mxml(fifths, mode);
                assert_eq!(
                    key.accidentals(),
                    fifths,
                    "fifths {fifths} in {mode:?} came back as {}",
                    key.accidentals()
                );
            }
        }
    }

    #[test]
    fn major_fifths_name_the_expected_tonic() {
        let cases = [
            (0i8, "c", 0),
            (1, "g", 0),
            (2, "d", 0),
            (-1, "f", 0),
            (-2, "b", -1), // B flat
            (-3, "e", -1), // E flat
            (6, "f", 1),   // F sharp
            (-6, "g", -1), // G flat
            (7, "c", 1),   // C sharp
            (-7, "c", -1), // C flat
        ];

        for (fifths, letter, alter) in cases {
            let key = Key::from_mxml(fifths, Mode::Major);
            assert_eq!(
                key.step,
                step(letter, alter),
                "{fifths} fifths major should be {letter} (alter {alter})"
            );
        }
    }

    #[test]
    fn minor_fifths_name_the_tonic_not_the_relative_major() {
        // The pairs the old count-based representation got wrong: the signature
        // belongs to the relative major, the tonic does not.
        let cases = [
            (0i8, "a", 0), // A minor, no accidentals
            (-3, "c", 0),  // C minor, three flats
            (2, "b", 0),   // B minor, two sharps
            (3, "f", 1),   // F sharp minor, three sharps
            (-1, "d", 0),  // D minor, one flat
        ];

        for (fifths, letter, alter) in cases {
            let key = Key::from_mxml(fifths, Mode::Minor);
            assert_eq!(
                key.step,
                step(letter, alter),
                "{fifths} fifths minor should be {letter} (alter {alter})"
            );
            assert_eq!(key.accidentals(), fifths);
        }
    }

    /// A minor key's tonic sits three fifths above the major key sharing its
    /// signature, and both must still report that one shared signature.
    #[test]
    fn relative_keys_share_a_signature() {
        let e_flat_major = Key::from_mxml(-3, Mode::Major);
        let c_minor = Key::from_mxml(-3, Mode::Minor);

        assert_eq!(e_flat_major.step, step("e", -1));
        assert_eq!(c_minor.step, step("c", 0));
        assert_eq!(e_flat_major.accidentals(), c_minor.accidentals());
        assert_eq!(c_minor.accidentals(), -3);
    }

    #[test]
    fn the_default_key_is_c_major() {
        assert_eq!(Key::default(), Key::C_MAJOR);
        assert_eq!(Key::C_MAJOR.step, step("c", 0));
        assert_eq!(Key::C_MAJOR.accidentals(), 0);
        assert_eq!(Key::C_MAJOR, Key::from_mxml(0, Mode::Major));
    }

    /// `<mode>` is optional and may be `none` or a church mode; the parser reads
    /// anything that isn't "minor" as major. That names the wrong tonic for a
    /// modal key, but it must not change the engraved signature.
    #[test]
    fn only_major_and_minor_parse_as_modes() {
        assert_eq!(Mode::try_from("major"), Ok(Mode::Major));
        assert_eq!(Mode::try_from("minor"), Ok(Mode::Minor));
        assert!(Mode::try_from("none").is_err());
        assert!(Mode::try_from("dorian").is_err());

        // D dorian is written as one sharp; read as major it becomes G major,
        // whose signature is the same one sharp.
        assert_eq!(Key::from_mxml(1, Mode::Major).accidentals(), 1);
    }
}
