//! SMuFL `engravingDefaults`: the line thicknesses a font recommends, read as
//! a layout tier between the document's `<appearance>` and `APP_DEFAULTS`.
//!
//! These tests pin the tier's place in the precedence, its fallback when a
//! font leaves the block out, and that the app defaults stay equal to
//! Bravura's -- the hand-transcribed copies had drifted before the block was
//! read, and nothing would notice them drifting again.

#[cfg(test)]
mod tests {
    use lib::score::engrave::{arrange_score, walk_document};
    use lib::score::layout_options::{APP_DEFAULTS, StaffLayout, UserLayout};
    use lib::score::visual::score::Score;
    use lib::score::visual::staff::Staff;
    use lib::smufl::smufl_font::SmuflFont;
    use roxmltree::Document;
    use serde_json::Value;
    use std::fs::read_to_string;
    use std::sync::OnceLock;

    const BRAVURA_METADATA: &str =
        "assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json";
    const GLYPH_NAMES: &str = "assets/smufl/metadata/glyphnames.json";

    fn asset(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    fn bravura() -> &'static SmuflFont {
        static FONT: OnceLock<SmuflFont> = OnceLock::new();
        FONT.get_or_init(|| SmuflFont::load(&asset(BRAVURA_METADATA), &asset(GLYPH_NAMES)))
    }

    /// Bravura, with its `engravingDefaults` block handed to `edit` first.
    fn edited_bravura(edit: impl FnOnce(&mut serde_json::Map<String, Value>)) -> SmuflFont {
        let mut meta: Value = serde_json::from_str(&asset(BRAVURA_METADATA)).unwrap();
        edit(meta.as_object_mut().unwrap());
        SmuflFont::load(&meta.to_string(), &asset(GLYPH_NAMES))
    }

    fn with_staff_line_thickness(spaces: f64) -> SmuflFont {
        edited_bravura(|meta| {
            meta["engravingDefaults"]["staffLineThickness"] = Value::from(spaces);
        })
    }

    /// A one-part, one-measure score. `appearance` goes inside `<defaults>`.
    fn score_xml(appearance: &str) -> String {
        format!(
            "<score-partwise version=\"4.0\">\
             <defaults><scaling><millimeters>7</millimeters><tenths>40</tenths></scaling>\
             <appearance>{appearance}</appearance></defaults>\
             <part-list><score-part id=\"P1\"><part-name>P</part-name></score-part></part-list>\
             <part id=\"P1\"><measure number=\"1\" width=\"300\">\
             <attributes><divisions>4</divisions>\
             <key><fifths>0</fifths><mode>major</mode></key>\
             <time><beats>4</beats><beat-type>4</beat-type></time>\
             <clef><sign>G</sign><line>2</line></clef></attributes>\
             <note><pitch><step>C</step><octave>5</octave></pitch><duration>16</duration>\
             <voice>1</voice><type>whole</type></note>\
             </measure></part></score-partwise>"
        )
    }

    fn engrave(xml: &str, font: &SmuflFont, user_layout: &UserLayout) -> Score {
        let document = Document::parse(xml).expect("score does not parse");
        let (mut score, defaults, _) = walk_document(&document, font, &mut |_| {});
        arrange_score(&mut score, &defaults, font, user_layout, &mut |_| {});
        score
    }

    fn first_staff(score: &Score) -> &Staff {
        score
            .pages
            .values()
            .flat_map(|page| page.systems.values())
            .flat_map(|system| system.sections.values())
            .flat_map(|section| section.part_groups.values())
            .flat_map(|group| group.parts.values())
            .flat_map(|part| part.staves.values())
            .next()
            .expect("the score engraved no staff")
    }

    fn staff_line_width(appearance: &str, font: &SmuflFont, user_layout: &UserLayout) -> f32 {
        first_staff(&engrave(&score_xml(appearance), font, user_layout)).line_width
    }

    fn assert_close(actual: f32, expected: f32, what: &str) {
        assert!(
            (actual - expected).abs() < 1e-5,
            "{what}: expected {expected}, got {actual}"
        );
    }

    /// Every option with a font tier, as (name, the font's value, the app
    /// default).
    fn font_backed_options(font: &SmuflFont) -> Vec<(&'static str, Option<f32>, f32)> {
        let l = &font.layout;
        let app = &APP_DEFAULTS;
        vec![
            ("staff.line_width", l.staff.line_width, app.staff.line_width),
            ("barline.light", l.barline.light, app.barline.light),
            ("barline.heavy", l.barline.heavy, app.barline.heavy),
            ("beam.thickness", l.beam.thickness, app.beam.thickness),
            ("beam.spacing", l.beam.spacing, app.beam.spacing),
            ("stem.thickness", l.stem.thickness, app.stem.thickness),
            (
                "tie.endpoint_thickness",
                l.tie.endpoint_thickness,
                app.tie.endpoint_thickness,
            ),
            (
                "tie.midpoint_thickness",
                l.tie.midpoint_thickness,
                app.tie.midpoint_thickness,
            ),
            (
                "group_bracket.thickness",
                l.group_bracket.thickness,
                app.group_bracket.thickness,
            ),
            (
                "group_line.thickness",
                l.group_line.thickness,
                app.group_line.thickness,
            ),
        ]
    }

    #[test]
    fn bravura_engraving_defaults_deserialize_in_staff_spaces() {
        let defaults = &bravura().meta.engraving_defaults;
        assert_eq!(defaults.staff_line_thickness, Some(0.13));
        assert_eq!(defaults.thin_barline_thickness, Some(0.16));
        assert_eq!(defaults.beam_spacing, Some(0.25));
        assert_eq!(defaults.stem_thickness, Some(0.12));
    }

    #[test]
    fn font_layout_is_in_tenths() {
        assert_close(
            bravura().layout.staff.line_width.unwrap(),
            0.13 * Staff::DEFAULT_SPACE_SIZE,
            "staff.line_width",
        );
    }

    /// The app defaults are the fallback for a font that leaves a key out, and
    /// are documented as Bravura's own. Keeps them from drifting again.
    #[test]
    fn app_defaults_equal_bravuras_engraving_defaults() {
        for (name, font, app) in font_backed_options(bravura()) {
            let font = font.unwrap_or_else(|| panic!("Bravura has no value for {name}"));
            assert_close(app, font, name);
        }
    }

    #[test]
    fn a_font_without_engraving_defaults_still_loads_and_falls_back_to_the_app() {
        let font = edited_bravura(|meta| {
            meta.remove("engravingDefaults");
        });

        for (name, font_value, _) in font_backed_options(&font) {
            assert_eq!(font_value, None, "{name} resolved from a missing block");
        }

        let resolved = staff_line_width("", &font, &UserLayout::default());
        assert_close(resolved, APP_DEFAULTS.staff.line_width, "staff line width");
    }

    #[test]
    fn the_font_outranks_the_app_default() {
        let font = with_staff_line_thickness(0.2);
        let resolved = staff_line_width("", &font, &UserLayout::default());
        assert_close(resolved, 2., "staff line width");
    }

    #[test]
    fn the_document_outranks_the_font() {
        let font = with_staff_line_thickness(0.2);
        let resolved = staff_line_width(
            "<line-width type=\"staff\">3</line-width>",
            &font,
            &UserLayout::default(),
        );
        assert_close(resolved, 3., "staff line width");
    }

    #[test]
    fn the_user_outranks_the_document_and_the_font() {
        let font = with_staff_line_thickness(0.2);
        let user_layout = UserLayout {
            staff: StaffLayout {
                line_width: Some(4.),
            },
            ..Default::default()
        };
        let resolved = staff_line_width(
            "<line-width type=\"staff\">3</line-width>",
            &font,
            &user_layout,
        );
        assert_close(resolved, 4., "staff line width");
    }

    /// The stroke a bracket's tip glyphs are scaled against is the font's own
    /// `bracketThickness`, not a constant.
    #[test]
    fn bracket_glyph_stroke_comes_from_the_font() {
        assert_close(bravura().bracket_top().thickness, 5., "Bravura bracket top");

        let font = edited_bravura(|meta| {
            meta["engravingDefaults"]["bracketThickness"] = Value::from(0.8);
        });
        assert_close(font.bracket_top().thickness, 8., "edited bracket top");
        assert_close(font.bracket_bottom().thickness, 8., "edited bracket bottom");
    }
}
