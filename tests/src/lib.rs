#[cfg(test)]
mod tests {
    use lib::score::core::clef::Clef;
    use lib::score::core::pitch::Pitch;

    #[test]
    fn test_middle_c_returns_base_line() {
        let c4 = Pitch {
            step: "c".into(),
            octave: 4,
        };

        assert_eq!(Clef::Treble.line_index_at_pitch(&c4), 10);
        assert_eq!(Clef::Bass.line_index_at_pitch(&c4), -2);
        assert_eq!(Clef::Alto.line_index_at_pitch(&c4), 4);

        // also move up one step just to be sure
        let d4 = Pitch {
            step: "d".into(),
            octave: 4,
        };

        assert_eq!(Clef::Treble.line_index_at_pitch(&d4), 9);
        assert_eq!(Clef::Bass.line_index_at_pitch(&d4), -3);
        assert_eq!(Clef::Alto.line_index_at_pitch(&d4), 3);
    }

    #[test]
    fn test_higher_pitches_decrease_line_index() {
        let clef = Clef::Treble;

        let c4 = Pitch {
            step: "c".into(),
            octave: 4,
        }; // Base = 10
        let d4 = Pitch {
            step: "d".into(),
            octave: 4,
        }; // 1 step up -> index 9
        let g4 = Pitch {
            step: "g".into(),
            octave: 4,
        }; // 4 steps up -> index 6
        let c5 = Pitch {
            step: "c".into(),
            octave: 5,
        }; // 7 steps up -> index 3

        assert_eq!(clef.line_index_at_pitch(&c4), 10);
        assert_eq!(clef.line_index_at_pitch(&d4), 9);
        assert_eq!(clef.line_index_at_pitch(&g4), 6);
        assert_eq!(clef.line_index_at_pitch(&c5), 3);
    }

    #[test]
    fn test_lower_pitches_increase_line_index() {
        let clef = Clef::Treble;

        let b3 = Pitch {
            step: "b".into(),
            octave: 3,
        }; // 1 step down -> index 11
        let a3 = Pitch {
            step: "a".into(),
            octave: 3,
        }; // 2 steps down -> index 12
        let c3 = Pitch {
            step: "c".into(),
            octave: 3,
        }; // 7 steps down -> index 17

        assert_eq!(clef.line_index_at_pitch(&b3), 11);
        assert_eq!(clef.line_index_at_pitch(&a3), 12);
        assert_eq!(clef.line_index_at_pitch(&c3), 17);
    }
}
