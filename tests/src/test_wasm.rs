#[cfg(test)]
mod tests {
    use std::fs::read_to_string;
    use wasm::{MAX_CANVAS_PIXELS, free_score, load_score, render};

    fn fixture(relative_path: &str) -> String {
        read_to_string(format!("{}/../{relative_path}", env!("CARGO_MANIFEST_DIR")))
            .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    /// `render` reruns apply_layout/rebeam/measure/arrange_pages on the same
    /// cached `Score` every call instead of rebuilding it from scratch. That's
    /// only safe if those steps are idempotent; this guards the assumption by
    /// calling `render` twice with identical arguments and requiring identical
    /// output.
    #[test]
    fn render_is_idempotent_across_repeated_calls_with_the_same_layout() {
        let musicxml = fixture("assets/xmlsamples/ActorPreludeSample.musicxml");
        let meta_json = fixture("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("assets/smufl/metadata/glyphnames.json");

        let handle =
            load_score(&musicxml, &meta_json, &glyph_names_json).expect("load_score failed");

        let render_once = || {
            render(
                handle,
                false,
                Some("#ffffff".to_string()),
                Some("#000000".to_string()),
                None,
                None,
                None,
                None,
                None,
                None,
                1.0,
            )
            .expect("render failed")
        };

        // `geometry()`/`text_blob()` take the buffer out of the RenderOutput
        // (mem::take) rather than clone it, so each must be read exactly once
        // per instance into a local - re-reading the same instance would come
        // back empty.
        let mut first = render_once();
        let mut second = render_once();
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
        let musicxml = fixture("assets/xmlsamples/ActorPreludeSample.musicxml");
        let meta_json = fixture("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("assets/smufl/metadata/glyphnames.json");

        let handle =
            load_score(&musicxml, &meta_json, &glyph_names_json).expect("load_score failed");

        let render_at = |device_pixel_ratio: f32| {
            render(
                handle,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                device_pixel_ratio,
            )
            .expect("render failed")
        };

        let unscaled = render_at(1.0);
        let unscaled_physical_pixels =
            unscaled.bounds_width() as f64 * unscaled.bounds_height() as f64;
        assert!(
            unscaled_physical_pixels <= MAX_CANVAS_PIXELS as f64,
            "test fixture is expected to already fit the budget at device_pixel_ratio 1.0, \
             got {unscaled_physical_pixels} physical pixels",
        );

        let huge_device_pixel_ratio = 1000.0;
        let scaled = render_at(huge_device_pixel_ratio);
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

    /// Two scores loaded concurrently (e.g. two `<music-xml>` elements on one
    /// page sharing this wasm instance) must stay independent: loading the
    /// second must not evict or corrupt the first's cache, and each handle's
    /// `render` must keep reflecting only the document it was given.
    #[test]
    fn two_handles_render_independently_and_do_not_evict_each_other() {
        let meta_json = fixture("assets/smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("assets/smufl/metadata/glyphnames.json");

        let musicxml_a = fixture("assets/xmlsamples/ActorPreludeSample.musicxml");
        let musicxml_b = fixture("assets/xmlsamples/BrahWiMeSample.musicxml");

        let handle_a =
            load_score(&musicxml_a, &meta_json, &glyph_names_json).expect("load_score a failed");
        let handle_b =
            load_score(&musicxml_b, &meta_json, &glyph_names_json).expect("load_score b failed");
        assert_ne!(
            handle_a, handle_b,
            "expected distinct handles per load_score call"
        );

        let render_handle = |handle: u32| {
            render(
                handle, false, None, None, None, None, None, None, None, None, 1.0,
            )
            .expect("render failed")
        };

        let mut a_before = render_handle(handle_a);
        let mut b = render_handle(handle_b);
        let mut a_after = render_handle(handle_a);

        // Each instance's geometry() is read exactly once into a local -
        // re-reading the same instance later (e.g. `b` again below) would
        // come back empty.
        let a_before_geometry = a_before.geometry();
        let b_geometry = b.geometry();
        let a_after_geometry = a_after.geometry();

        assert_eq!(
            a_before_geometry, a_after_geometry,
            "loading handle b must not change handle a's rendered geometry"
        );
        assert_ne!(
            a_after_geometry, b_geometry,
            "two different scores are expected to render different geometry"
        );

        // `render`'s missing-handle error path builds a `JsValue`, which only
        // wasm-bindgen's real (non-stub) implementation supports - it panics
        // when exercised on a native (non-wasm32) test target - so this only
        // checks that freeing one handle leaves the other's cache intact,
        // not the error path itself.
        free_score(handle_a);
        let mut b_again = render_handle(handle_b);
        assert_eq!(
            b_again.geometry(),
            b_geometry,
            "freeing handle a must not affect handle b"
        );
    }
}
