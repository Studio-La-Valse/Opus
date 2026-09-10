#[cfg(test)]
mod tests {
    use lib::geometry::xy::XY;
    use lib::score::app_defaults::AppDefaults;
    use lib::score::core::duration_base::BaseDuration;
    use lib::score::core::time_signature::TimeSignature as TimeSignatureCore;
    use lib::score::score_defaults::ScoreDefaults;
    use lib::score::user_layout::UserLayout;
    use lib::score::visual::layoutable::{LayoutParams, Layoutable};
    use lib::score::visual::time_signature::TimeSignature;
    use lib::smufl::smufl_font::SmuflFont;
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

    /// SMuFL codepoints for `timeSig0`..`timeSig9` are contiguous from U+E080,
    /// which lets a test read a rendered number back as an ordinary string.
    fn spell(number: &lib::smufl::glyphs::number::Number) -> String {
        number
            .digits
            .iter()
            .map(|d| {
                let offset = u32::from(d.codepoint) - 0xE080;
                char::from_digit(offset, 10).expect("not a timeSig digit glyph")
            })
            .collect()
    }

    /// The font has a glyph per decimal digit and none for a whole number, so a
    /// value of 10 or more has to come back as a sequence.
    #[test]
    fn a_number_is_spelled_out_digit_by_digit() {
        for value in [0u32, 1, 4, 9, 10, 12, 16, 32, 64, 128, 1024] {
            assert_eq!(spell(&font().number(value)), value.to_string());
        }
    }

    #[test]
    fn a_single_digit_number_is_one_glyph() {
        assert_eq!(font().number(7).digits.len(), 1);
        assert_eq!(font().number(0).digits.len(), 1);
        assert_eq!(font().number(64).digits.len(), 2);
    }

    /// Every `x/16`, `x/32` and `x/64` signature has a two-digit denominator,
    /// and numerators of 10 and up are ordinary. All of these used to abort the
    /// render on a missing `timeSig16`-style glyph.
    #[test]
    fn multi_digit_time_signatures_resolve() {
        let cases = [
            (4u8, 4u32, "4", "4"),
            (12, 8, "12", "8"),
            (3, 16, "3", "16"),
            (15, 32, "15", "32"),
            (1, 64, "1", "64"),
        ];

        for (beats, beat_type, expect_num, expect_denom) in cases {
            let core = TimeSignatureCore {
                time: beats,
                base: BaseDuration::from(beat_type),
            };
            let (num, denom) = font().time_signature(core);

            assert_eq!(spell(&num), expect_num);
            assert_eq!(spell(&denom), expect_denom);
        }
    }

    /// Laid out on a normal five-line staff, 40 tenths tall.
    fn measured(beats: u8, beat_type: u32) -> TimeSignature {
        measured_on_staff(beats, beat_type, 40.)
    }

    /// Laid out on a staff of `staff_height` tenths -- 40 for the usual five
    /// lines, 0 for a staff of a single line.
    fn measured_on_staff(beats: u8, beat_type: u32, staff_height: f32) -> TimeSignature {
        let (num, denom) = font().time_signature(TimeSignatureCore {
            time: beats,
            base: BaseDuration::from(beat_type),
        });
        let mut time_signature = TimeSignature::new(num, denom);

        let score_defaults = ScoreDefaults::default();
        let user_layout = UserLayout::default();
        let app_defaults = AppDefaults::default();
        let params = LayoutParams {
            score_defaults: &score_defaults,
            user_layout: &user_layout,
            app_defaults: &app_defaults,
            font: font(),
            abbreviate_names: false,
        };
        time_signature.resolve_layout(params);
        time_signature.measure(
            &XY {
                x: 0.,
                y: staff_height,
            },
            params,
        );
        time_signature.arrange(&XY::ZERO);

        time_signature
    }

    /// The two halves sit a space either side of the middle of the staff. On
    /// five lines that is the second and the fourth, which is where they have
    /// always been drawn.
    #[test]
    fn the_halves_sit_a_space_either_side_of_the_middle_of_the_staff() {
        let time_signature = measured_on_staff(4, 4, 40.);

        let num = time_signature.num_digits().next().unwrap().1;
        let denom = time_signature.denom_digits().next().unwrap().1;

        assert_eq!(num.y, 10., "a space above the middle line");
        assert_eq!(denom.y, 30., "a space below it");
    }

    /// A staff of one line is zero tenths tall. Splitting that height into
    /// quarters put both halves of the signature in the same place, one drawn
    /// over the other; measured from the middle outwards they straddle the line.
    #[test]
    fn the_halves_straddle_the_single_line_of_a_one_line_staff() {
        let time_signature = measured_on_staff(4, 4, 0.);

        let num = time_signature.num_digits().next().unwrap().1;
        let denom = time_signature.denom_digits().next().unwrap().1;

        assert_eq!(num.y, -10., "a space above the line");
        assert_eq!(denom.y, 10., "a space below it");
    }

    /// Digits step by the font's advance width, which is wider than the glyph's
    /// ink by its side bearings -- stepping by the bounding box instead would
    /// set them flush against each other.
    #[test]
    fn digits_step_by_their_advance_width() {
        let time_signature = measured(12, 8);
        let placed: Vec<_> = time_signature.num_digits().collect();

        assert_eq!(placed.len(), 2, "12 should be two glyphs");

        let (first, first_xy) = placed[0];
        let (_, second_xy) = placed[1];

        // `advance` is in staff spaces; one space is 10 tenths at scale 1.
        assert!(
            (second_xy.x - first_xy.x - first.advance * 10.).abs() < 1e-3,
            "expected a step of {}, got {}",
            first.advance * 10.,
            second_xy.x - first_xy.x
        );
        assert_eq!(first_xy.y, second_xy.y, "digits share a baseline");
    }

    /// The element reserves whichever half is wider, and the narrower half is
    /// centred under (or over) it. A `1` is much narrower than a `4`, so even a
    /// single-digit-over-single-digit signature needs the centring.
    #[test]
    fn the_narrower_half_is_centred_against_the_wider_one() {
        let time_signature = measured(1, 4);

        let num: Vec<_> = time_signature.num_digits().collect();
        let denom: Vec<_> = time_signature.denom_digits().collect();

        let num_advance = num[0].0.advance * 10.;
        let denom_advance = denom[0].0.advance * 10.;
        assert!(num_advance < denom_advance, "a 1 is narrower than a 4");

        assert_eq!(time_signature.width, denom_advance);

        let num_centre = num[0].1.x + num_advance / 2.;
        let denom_centre = denom[0].1.x + denom_advance / 2.;
        assert!(
            (num_centre - denom_centre).abs() < 1e-3,
            "numerator centred at {num_centre}, denominator at {denom_centre}"
        );
    }

    /// A two-digit numerator makes the element wider than its one-digit
    /// denominator, rather than the denominator dictating the width.
    #[test]
    fn the_element_reserves_the_wider_of_the_two_halves() {
        let time_signature = measured(12, 8);

        let num_width: f32 = time_signature
            .num_digits()
            .map(|(digit, _)| digit.advance * 10.)
            .sum();
        let denom_width: f32 = time_signature
            .denom_digits()
            .map(|(digit, _)| digit.advance * 10.)
            .sum();

        assert!(num_width > denom_width);
        assert!((time_signature.width - num_width).abs() < 1e-3);
    }
}
