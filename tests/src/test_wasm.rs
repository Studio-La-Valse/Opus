#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use wasm::{MAX_CANVAS_PIXELS, RenderOptions, RenderOutput, WasmScore};

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
    fn render_at(score: &mut WasmScore, device_pixel_ratio: f32) -> RenderOutput {
        score.render_with(&RenderOptions {
            device_pixel_ratio,
            ..Default::default()
        })
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

        // `geometry()`/`text_blob()` take the buffer out of the RenderOutput
        // (mem::take) rather than clone it, so each must be read exactly once
        // per instance into a local - re-reading the same instance would come
        // back empty.
        let mut first = render_once(&mut score);
        let mut second = render_once(&mut score);
        let first_geometry = first.geometry();
        let second_geometry = second.geometry();
        let first_text_blob = first.text_blob();
        let second_text_blob = second.text_blob();

        assert_eq!(first_geometry, second_geometry);
        assert_eq!(first_text_blob, second_text_blob);
        assert_eq!(first.bounds_width(), second.bounds_width());
        assert_eq!(first.bounds_height(), second.bounds_height());
    }

    /// A single small score stays within [`MAX_CANVAS_PIXELS`] even at a
    /// typical devicePixelRatio, so it should render unscaled (this also
    /// guards against `render_scale` kicking in when it shouldn't). At an
    /// extreme device_pixel_ratio, though, the same score would blow well
    /// past the budget if left unscaled, so `render` must shrink it down to
    /// fit - this is what actually keeps the browser's canvas raster/composite
    /// cost bounded regardless of how a caller reports its pixel ratio.
    #[test]
    fn render_keeps_the_canvas_backing_store_within_the_pixel_budget() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let unscaled = render_at(&mut score, 1.0);
        let unscaled_physical_pixels =
            unscaled.bounds_width() as f64 * unscaled.bounds_height() as f64;
        assert!(
            unscaled_physical_pixels <= MAX_CANVAS_PIXELS as f64,
            "test fixture is expected to already fit the budget at device_pixel_ratio 1.0, \
             got {unscaled_physical_pixels} physical pixels",
        );

        let huge_device_pixel_ratio = 1000.0;
        let scaled = render_at(&mut score, huge_device_pixel_ratio);
        let scaled_physical_pixels = (scaled.bounds_width() as f64
            * huge_device_pixel_ratio as f64)
            * (scaled.bounds_height() as f64 * huge_device_pixel_ratio as f64);

        assert!(
            scaled_physical_pixels <= MAX_CANVAS_PIXELS as f64 * 1.01, // float slop
            "expected the render at a huge device_pixel_ratio to stay within the canvas pixel \
             budget, got {scaled_physical_pixels} physical pixels",
        );
        assert!(
            scaled.bounds_width() < unscaled.bounds_width(),
            "expected element coordinates to actually shrink once the budget kicks in",
        );
    }

    /// A device pixel ratio JS couldn't supply sensibly - the field left off
    /// the options object entirely, or a NaN/zero coming out of a bad
    /// `devicePixelRatio` read - must mean "render unscaled" rather than
    /// dividing the pixel budget by zero and scaling by infinity.
    #[test]
    fn render_treats_an_unusable_device_pixel_ratio_as_one() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let mut baseline = render_at(&mut score, 1.0);
        let baseline_geometry = baseline.geometry();

        for ratio in [0.0, -2.0, f32::NAN, f32::INFINITY] {
            let mut output = render_at(&mut score, ratio);
            assert_eq!(
                output.geometry(),
                baseline_geometry,
                "device_pixel_ratio {ratio} should render exactly like 1.0",
            );
        }

        // The default options object leaves it off altogether.
        let mut defaulted = score.render_with(&RenderOptions::default());
        assert_eq!(defaulted.geometry(), baseline_geometry);
    }

    /// `render` exposes a page table parallel to `geometry`: 5 f32s per page,
    /// `[start_index, origin_x, origin_y, width, height]`. The start indices
    /// must be non-decreasing and land within `geometry`, the first must be 0,
    /// and the budget down-scaling must shrink the page rectangles alongside
    /// the geometry so the table stays consistent with the scaled stream.
    #[test]
    fn render_page_table_tracks_the_geometry_and_scales_with_it() {
        let mut score = score("assets/xmlsamples/ActorPreludeSample.musicxml");

        let mut unscaled = render_at(&mut score, 1.0);
        let geometry_len = unscaled.geometry().len();
        let page_table = unscaled.page_table();

        assert_eq!(page_table.len() % 5, 0, "5 f32s per page");
        assert!(
            page_table.len() >= 10,
            "the ActorPrelude sample lays out onto multiple pages, got {} page(s)",
            page_table.len() / 5,
        );
        assert_eq!(page_table[0], 0.0, "first page starts at geometry index 0");

        let mut prev_start = 0.0_f32;
        for record in page_table.chunks(5) {
            let start = record[0];
            assert!(
                start >= prev_start,
                "page start indices must be non-decreasing, got {start} after {prev_start}",
            );
            assert!(
                start as usize <= geometry_len,
                "page start {start} must index into a {geometry_len}-long geometry",
            );
            assert!(
                record[3] > 0.0 && record[4] > 0.0,
                "page rectangle must be non-empty, got {}x{}",
                record[3],
                record[4],
            );
            prev_start = start;
        }

        let mut scaled = render_at(&mut score, 1000.0);
        let scaled_table = scaled.page_table();
        assert_eq!(
            scaled_table.len(),
            page_table.len(),
            "scaling must not change the page count",
        );
        for (unscaled_rec, scaled_rec) in page_table.chunks(5).zip(scaled_table.chunks(5)) {
            assert!(
                scaled_rec[3] < unscaled_rec[3] && scaled_rec[4] < unscaled_rec[4],
                "budget scaling must shrink each page rectangle: {}x{} -> {}x{}",
                unscaled_rec[3],
                unscaled_rec[4],
                scaled_rec[3],
                scaled_rec[4],
            );
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

        let mut a_before = render_at(&mut a, 1.0);
        let mut b_output = render_at(&mut b, 1.0);
        let mut a_after = render_at(&mut a, 1.0);

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
        let mut b_again = render_at(&mut b, 1.0);
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
        let layout: lib::score::user_layout::UserLayout = serde_json::from_str(
            r##"{ "pageColor": "#112233", "pageOrientation": "vertical", "tieHeightRatio": 0.25 }"##,
        )
        .expect("valid layout options failed to deserialize");

        assert_eq!(layout.page_color.expect("page_color").to_hex(), "#112233FF");
        assert_eq!(
            layout.page_orientation,
            Some(lib::score::page_orientation::PageOrientation::Vertical),
        );
        assert_eq!(layout.tie_height_ratio, Some(0.25));
        assert_eq!(layout.vertical_gutter, None, "unset options stay None");

        let unknown = serde_json::from_str::<lib::score::user_layout::UserLayout>(
            r##"{ "pageColour": "#112233" }"##,
        );
        assert!(unknown.is_err(), "an unknown option name must be rejected");

        let bad_color = serde_json::from_str::<lib::score::user_layout::UserLayout>(
            r#"{ "pageColor": "nope" }"#,
        );
        assert!(bad_color.is_err(), "an unparseable colour must be rejected");
    }
}
