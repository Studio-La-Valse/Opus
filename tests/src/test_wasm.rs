#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use wasm::{RenderOptions, RenderOutput, WasmScore};

    fn fixture(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    /// Builds a score from one of the sample documents and the real SMuFL
    /// metadata, the way the constructor is called from JS.
    fn score(musicxml_path: &str) -> WasmScore {
        let musicxml = fixture(musicxml_path);
        let meta_json = fixture("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("assets/smufl/metadata/glyphnames.json");

        WasmScore::new(&musicxml, &meta_json, &glyph_names_json).expect("failed to build score")
    }

    /// These tests go through `render_with` rather than the `render(JsValue)`
    /// wrapper: decoding a `JsValue` needs wasm-bindgen's real (non-stub)
    /// implementation, which panics on a native test target. Everything past
    /// the decode is the same code either way.
    fn render_at(score: &mut WasmScore) -> RenderOutput {
        score.render_with(&RenderOptions::default())
    }

    /// `render` reruns rebeam/measure/arrange_pages on the same cached `Score`
    /// every call instead of rebuilding it from scratch. That's only safe if
    /// those steps are idempotent; this guards the assumption by rendering
    /// twice with identical options and requiring identical output.
    #[test]
    fn render_is_idempotent_across_repeated_calls_with_the_same_layout() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let render_once = |score: &mut WasmScore| {
            score.render_with(&RenderOptions {
                // Doubled hashes because the JSON itself contains `"#`, which
                // would close a single-hash raw string.
                layout: serde_json::from_str(
                    r##"{ "pageColor": "#ffffff", "foregroundColor": "#000000" }"##,
                )
                .expect("layout options failed to deserialize"),
                ..Default::default()
            })
        };

        // `geometry()`/`text_blob()`/`page_table()` take the buffer out of the
        // RenderOutput (mem::take) rather than clone it, so each must be read
        // exactly once per instance into a local - re-reading the same
        // instance would come back empty.
        let mut first = render_once(&mut score);
        let mut second = render_once(&mut score);
        let first_geometry = first.geometry();
        let second_geometry = second.geometry();
        let first_text_blob = first.text_blob();
        let second_text_blob = second.text_blob();
        let first_page_table = first.page_table();
        let second_page_table = second.page_table();

        assert_eq!(first_geometry, second_geometry);
        assert_eq!(first_text_blob, second_text_blob);
        assert_eq!(first_page_table, second_page_table);
    }

    /// `render` exposes a page table parallel to `geometry`: 4 f32s per page,
    /// `[start_index, text_start_index, width, height]`. The start indices
    /// must be non-decreasing and land within `geometry`, the first must be 0,
    /// and every page rectangle must be non-empty. Pages are engraved
    /// page-local, so there is no page origin to track any more.
    #[test]
    fn render_page_table_tracks_the_geometry() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let mut output = render_at(&mut score);
        let geometry_len = output.geometry().len();
        let page_table = output.page_table();

        assert_eq!(page_table.len() % 4, 0, "4 f32s per page");
        assert!(
            page_table.len() >= 8,
            "the ActorPrelude sample lays out onto multiple pages, got {} page(s)",
            page_table.len() / 4,
        );
        assert_eq!(page_table[0], 0.0, "first page starts at geometry index 0");
        assert_eq!(
            page_table[1], 0.0,
            "first page starts at text index 0: nothing precedes it",
        );

        let mut prev_geometry_start = 0.0_f32;
        let mut prev_text_start = 0.0_f32;
        for record in page_table.chunks(4) {
            let geometry_start = record[0];
            let text_start = record[1];
            assert!(
                geometry_start >= prev_geometry_start,
                "page geometry start indices must be non-decreasing, got {geometry_start} after {prev_geometry_start}",
            );
            assert!(
                geometry_start as usize <= geometry_len,
                "page geometry start {geometry_start} must index into a {geometry_len}-long geometry",
            );
            assert!(
                text_start >= prev_text_start,
                "page text start indices must be non-decreasing, got {text_start} after {prev_text_start}",
            );
            assert!(
                record[2] > 0.0 && record[3] > 0.0,
                "page rectangle must be non-empty, got {}x{}",
                record[2],
                record[3],
            );
            prev_geometry_start = geometry_start;
            prev_text_start = text_start;
        }
    }

    /// Two scores alive at once (e.g. two `<music-xml>` elements on one page
    /// sharing this wasm instance) must stay independent: each owns its
    /// engraved state, so building or dropping one can't disturb the other's
    /// renders. Nothing is shared to get wrong now that there's no registry,
    /// which is exactly the property worth pinning down.
    #[test]
    fn two_scores_render_independently() {
        let mut a = score("assets/xmlsamples/ActorPreludeSample.musicxml");
        let mut b = score("assets/xmlsamples/BrahWiMeSample.musicxml");

        let mut a_before = render_at(&mut a);
        let mut b_output = render_at(&mut b);
        let mut a_after = render_at(&mut a);

        // Each instance's geometry() is read exactly once into a local -
        // re-reading the same instance later would come back empty.
        let a_before_geometry = a_before.geometry();
        let b_geometry = b_output.geometry();
        let a_after_geometry = a_after.geometry();

        assert_eq!(
            a_before_geometry, a_after_geometry,
            "building and rendering b must not change a's rendered geometry"
        );
        assert_ne!(
            a_after_geometry, b_geometry,
            "two different scores are expected to render different geometry"
        );

        drop(a);
        let mut b_again = render_at(&mut b);
        assert_eq!(
            b_again.geometry(),
            b_geometry,
            "dropping a must not affect b"
        );
    }

    /// The layout options are deserialized straight into `UserLayout`, so every
    /// field it gains is exposed to callers for free, spelled in camelCase and
    /// parsed by the same `FromStr` impls the CLI flags use.
    ///
    /// The unknown-name case only bites for a real map deserializer such as
    /// serde_json: serde-wasm-bindgen's struct deserializer looks up just the
    /// field names it expects, so through the browser path an unknown key is
    /// invisible rather than an error. `deny_unknown_fields` is still worth
    /// having for every other caller.
    #[test]
    fn layout_options_deserialize_by_userlayout_field_name() {
        let layout: lib::score::user_layout::UserLayout =
            serde_json::from_str(r##"{ "pageColor": "#112233", "tieHeightRatio": 0.25 }"##)
                .expect("valid layout options failed to deserialize");

        assert_eq!(layout.page_color.expect("page_color").to_hex(), "#112233FF");
        assert_eq!(layout.tie_height_ratio, Some(0.25));
        assert_eq!(layout.staff_line_width, None, "unset options stay None");

        let unknown = serde_json::from_str::<lib::score::user_layout::UserLayout>(
            r##"{ "pageColour": "#112233" }"##,
        );
        assert!(unknown.is_err(), "an unknown option name must be rejected");

        let bad_color = serde_json::from_str::<lib::score::user_layout::UserLayout>(
            r#"{ "pageColor": "nope" }"#,
        );
        assert!(bad_color.is_err(), "an unparseable colour must be rejected");
    }

    /// Inverted tie height bounds used to panic inside `f32::clamp` during
    /// `arrange_score`, and on `wasm32` that panic traps without unwinding --
    /// see `TieMetrics::from_sources` / `ordered_bounds` in `tie.rs`. This
    /// pins the fix at the same boundary JS calls through: `render_with` must
    /// return rather than abort when `tieHeightMin` > `tieHeightMax`.
    #[test]
    fn render_survives_inverted_tie_height_bounds() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let mut output = score.render_with(&RenderOptions {
            layout: serde_json::from_str(r#"{ "tieHeightMin": 30, "tieHeightMax": 16 }"#)
                .expect("layout options failed to deserialize"),
            ..Default::default()
        });

        let page_table = output.page_table();
        assert!(!page_table.is_empty(), "expected at least one page");
        assert!(page_table[2] > 0.0, "first page width must be positive");
        assert!(page_table[3] > 0.0, "first page height must be positive");
    }

    /// The group-symbol overrides come through the same route, which is what
    /// lets a page re-draw a score's brackets as braces without re-parsing it.
    /// Their values are the MusicXML spellings, so a CSS custom property, a CLI
    /// flag and a `<group-symbol>` all say the same word.
    #[test]
    fn group_symbol_options_deserialize_by_their_musicxml_spelling() {
        use lib::score::core::group_symbol::GroupSymbol;

        let layout: lib::score::user_layout::UserLayout = serde_json::from_str(
            r##"{ "sectionSymbol": "line", "partGroupSymbol": "square",
                  "partSymbol": "none", "groupLineThickness": 3.5,
                  "sectionSymbolGap": 20, "groupNameSize": 18, "groupNamePadding": 12 }"##,
        )
        .expect("valid group symbol options failed to deserialize");

        assert_eq!(layout.section_symbol, Some(GroupSymbol::Line));
        assert_eq!(layout.part_group_symbol, Some(GroupSymbol::Square));
        assert_eq!(
            layout.part_symbol,
            Some(GroupSymbol::None),
            "'none' is a symbol that draws nothing, not an absent option"
        );
        assert_eq!(layout.group_line_thickness, Some(3.5));
        assert_eq!(layout.section_symbol_gap, Some(20.));
        assert_eq!(layout.group_name_size, Some(18.));
        assert_eq!(layout.group_name_padding, Some(12.));
        assert_eq!(layout.part_symbol_gap, None, "unset options stay None");

        let bare: lib::score::user_layout::UserLayout =
            serde_json::from_str("{}").expect("an empty layout object deserializes");
        assert_eq!(bare.group_name_size, None, "unset name knobs stay None");
        assert_eq!(bare.group_name_padding, None);

        let unknown = serde_json::from_str::<lib::score::user_layout::UserLayout>(
            r#"{ "sectionSymbol": "curly" }"#,
        );
        assert!(unknown.is_err(), "an unknown symbol name must be rejected");
    }
}
