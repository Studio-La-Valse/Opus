use lib::drawable::drawable_element::{DrawableElement, compute_bounds, scale_elem};
use lib::drawable::flat_buffer::to_flat_buffer;
use lib::drawable::layoutable::Layoutable;
use lib::geometry::color::Color;
use lib::geometry::xy::XY;
use lib::score::app_defaults::AppDefaults;
use lib::score::layout::Layout;
use lib::score::layout_ctx::LayoutCtx;
use lib::score::page_orientation::PageOrientation;
use lib::score::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use lib::score::user_layout::UserLayout;
use lib::score::visual::layout_engine::{HorizontalPageLayout, LayoutEngine, VerticalPageLayout};
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_pass::{BaseRenderer, DebugRenderer};
use lib::score::visual::score::Score;
use lib::score::visual::score_element::ScoreElement;
use lib::smufl::smufl_font::SmuflFont;
use lib::xml::visitor::{DefaultVisitor, Visitor};
use lib::xml::visitors::content_visitor::ContentVisitor;
use lib::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::xml::visitors::layout_visitor::LayoutVisitor;
use lib::xml::visitors::setup_visitor::SetupVisitor;
use lib::xml::walker::Walker;
use lib::xml::walker_ctx::WalkerCtx;
use roxmltree::{Document, ParsingOptions};
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
}

/// Everything built from a MusicXML document that doesn't depend on
/// [`UserLayout`]. Rebuilt by [`load_score`], reused by every subsequent
/// [`render`] call until the next [`load_score`] replaces it.
struct ScoreCache {
    layout: Layout,
    score: Score,
    font: SmuflFont,
}

thread_local! {
    static CACHE: RefCell<Option<ScoreCache>> = const { RefCell::new(None) };
}

/// Canvas-ready render output. `geometry` is a tagged f32 stream and `text_blob`
/// holds every Text record's content in encounter order, joined by
/// [`lib::drawable::flat_buffer::TEXT_DELIMITER`]. See
/// [`lib::drawable::flat_buffer::FlatBuffer`] for the exact record layout.
#[wasm_bindgen]
pub struct RenderOutput {
    bounds_min_x: f32,
    bounds_min_y: f32,
    bounds_width: f32,
    bounds_height: f32,
    geometry: Vec<f32>,
    text_blob: String,
}

#[wasm_bindgen]
impl RenderOutput {
    #[wasm_bindgen(getter)]
    pub fn bounds_min_x(&self) -> f32 {
        self.bounds_min_x
    }

    #[wasm_bindgen(getter)]
    pub fn bounds_min_y(&self) -> f32 {
        self.bounds_min_y
    }

    #[wasm_bindgen(getter)]
    pub fn bounds_width(&self) -> f32 {
        self.bounds_width
    }

    #[wasm_bindgen(getter)]
    pub fn bounds_height(&self) -> f32 {
        self.bounds_height
    }

    // Takes ownership of the buffer instead of cloning it: `draw()` in main.js
    // reads each property exactly once per RenderOutput before calling
    // `output.free()`, so there's no reason to pay for a second copy on top of
    // the copy wasm-bindgen already does when handing the Vec/String to JS.
    #[wasm_bindgen(getter)]
    pub fn geometry(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.geometry)
    }

    #[wasm_bindgen(getter)]
    pub fn text_blob(&mut self) -> String {
        std::mem::take(&mut self.text_blob)
    }
}

/// Parses `musicxml` and builds the [`Layout`]/[`Score`]/[`SmuflFont`] triple,
/// none of which depend on [`UserLayout`]. Caches the result so subsequent
/// [`render`] calls can re-layout and re-render without re-parsing or
/// re-walking the document. Replaces (and drops) whatever was cached before.
#[wasm_bindgen]
pub fn load_score(musicxml: &str, meta_json: &str, glyph_names_json: &str) -> Result<(), JsValue> {
    let font = SmuflFont::load(meta_json, glyph_names_json);

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(musicxml, options)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Only used to satisfy `WalkerCtx::new` during the two walk passes below;
    // no visitor reads it, since it doesn't affect the document's structure.
    let user_layout = UserLayout::default();
    let app_defaults: AppDefaults = Default::default();

    let mut layout_ctx = LayoutCtx::default();
    let mut layout = Layout::default();
    let mut score = Score::default();

    let visitor = DefaultVisitor {}
        .uses(LayoutContextVisitor {})
        .uses(SetupVisitor {})
        .uses(LayoutVisitor {
            encountered: HashSet::new(),
        });

    let mut ctx = WalkerCtx::new(
        &user_layout,
        &mut layout,
        &app_defaults,
        &mut layout_ctx,
        &mut score,
        &font,
    );
    Walker::new(visitor).walk(&document, &mut ctx);

    let visitor = DefaultVisitor {}
        .uses(LayoutContextVisitor {})
        .uses(ContentVisitor {
            clef_change: HashMap::new(),
        });

    let mut ctx = WalkerCtx::new(
        &user_layout,
        &mut layout,
        &app_defaults,
        &mut layout_ctx,
        &mut score,
        &font,
    );
    Walker::new(visitor).walk(&document, &mut ctx);

    CACHE.with_borrow_mut(|cache| {
        *cache = Some(ScoreCache {
            layout,
            score,
            font,
        });
    });

    Ok(())
}

/// Upper bound on the canvas's physical pixel count (`width * height` in
/// device pixels). A score's logical bounds can span many pages laid out
/// side by side, which at a high `device_pixel_ratio` produces a canvas far
/// larger than any viewport - and canvas raster/composite cost scales with
/// physical pixel count, not element count. Chosen as roughly "one big
/// native display's worth of pixels": scores that already fit render at full
/// native sharpness, only oversized ones get scaled down. Purely a
/// browser-canvas concern, so it lives here rather than in `lib`, which also
/// backs non-canvas consumers (e.g. SVG export) that shouldn't be capped.
const MAX_CANVAS_PIXELS: f32 = 12_000_000.0;

/// Applies `UserLayout` to the [`Score`] cached by the last [`load_score`]
/// call and returns the resulting drawable elements. Does no XML parsing or
/// walking, so it's cheap to call on every layout-only change (e.g. a page
/// color tweak).
///
/// `device_pixel_ratio` (the browser's `window.devicePixelRatio`) is used
/// only to decide whether the result needs scaling down to stay within
/// [`MAX_CANVAS_PIXELS`]; the caller still multiplies the returned bounds by
/// it as usual when sizing the canvas backing store.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn render(
    debug: bool,
    page_color: Option<String>,
    foreground_color: Option<String>,
    page_orientation: Option<String>,
    horizontal_gutter_even: Option<f32>,
    horizontal_gutter_uneven: Option<f32>,
    vertical_gutter: Option<f32>,
    device_pixel_ratio: f32,
) -> Result<RenderOutput, JsValue> {
    let page_color = page_color
        .map(|s| Color::from_str(&s))
        .transpose()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let foreground_color = foreground_color
        .map(|s| Color::from_str(&s))
        .transpose()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    let page_orientation = page_orientation
        .map(|s| PageOrientation::from_str(&s))
        .transpose()
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    let user_layout = UserLayout {
        page_color,
        foreground_color,
        page_orientation,
        horizontal_gutter_even,
        horizontal_gutter_uneven,
        vertical_gutter,
        ..Default::default()
    };
    let app_defaults: AppDefaults = Default::default();

    CACHE.with_borrow_mut(|cache| {
        let cache = cache
            .as_mut()
            .ok_or_else(|| JsValue::from_str("no score loaded; call load_score first"))?;

        cache
            .score
            .apply_layout(&cache.layout, &user_layout, &app_defaults);

        let strat_impl = Box::new(SimpleRebeamStrategy {});
        let strategy = Box::new(OnlyWhenRequiredRebeamStrategy { imp: strat_impl });
        cache.score.rebeam(strategy.as_ref());

        cache.score.measure(&XY::INFINITE);

        let orientation = user_layout
            .page_orientation
            .unwrap_or(app_defaults.page_orientation);
        let layout_engine: Box<dyn LayoutEngine> = match orientation {
            PageOrientation::Horizontal => Box::new(HorizontalPageLayout {
                gutter_even: user_layout
                    .horizontal_gutter_even
                    .unwrap_or(app_defaults.horizontal_gutter_even),
                gutter_uneven: user_layout
                    .horizontal_gutter_uneven
                    .unwrap_or(app_defaults.horizontal_gutter_uneven),
            }),
            PageOrientation::Vertical => Box::new(VerticalPageLayout {
                gutter: user_layout
                    .vertical_gutter
                    .unwrap_or(app_defaults.vertical_gutter),
            }),
        };
        layout_engine.arrange_pages(&mut cache.score, &XY::ZERO);

        let pass = BaseRenderer {};
        let compositor = RenderCompositor {
            pass: Box::new(pass),
        };
        let mut elements: Vec<DrawableElement<'_>> = compositor.walk(&cache.score, &cache.font);

        if debug {
            let pass = DebugRenderer {};
            let compositor = RenderCompositor {
                pass: Box::new(pass),
            };
            elements.extend(compositor.walk(&cache.score, &cache.font));
        }

        let (min_x, min_y, max_x, max_y) = compute_bounds(&elements);
        let physical_width = (max_x - min_x) * device_pixel_ratio;
        let physical_height = (max_y - min_y) * device_pixel_ratio;
        let render_scale = if physical_width > 0.0 && physical_height > 0.0 {
            (MAX_CANVAS_PIXELS / (physical_width * physical_height))
                .sqrt()
                .min(1.0)
        } else {
            1.0
        };
        let elements: Vec<DrawableElement<'_>> = if render_scale < 1.0 {
            elements
                .iter()
                .map(|el| scale_elem(el, render_scale))
                .collect()
        } else {
            elements
        };

        let flat = to_flat_buffer(elements);
        Ok(RenderOutput {
            bounds_min_x: flat.bounds.0,
            bounds_min_y: flat.bounds.1,
            bounds_width: flat.bounds.2,
            bounds_height: flat.bounds.3,
            geometry: flat.geometry,
            text_blob: flat.text_blob,
        })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::read_to_string;

    fn fixture(relative_path: &str) -> String {
        read_to_string(format!(
            "{}/../{relative_path}",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap_or_else(|e| panic!("failed to read {relative_path}: {e}"))
    }

    /// `render` reruns apply_layout/rebeam/measure/arrange_pages on the same
    /// cached `Score` every call instead of rebuilding it from scratch. That's
    /// only safe if those steps are idempotent; this guards the assumption by
    /// calling `render` twice with identical arguments and requiring identical
    /// output.
    #[test]
    fn render_is_idempotent_across_repeated_calls_with_the_same_layout() {
        let musicxml = fixture("xmlsamples/ActorPreludeSample.musicxml");
        let meta_json = fixture("smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("smufl/metadata/glyphnames.json");

        load_score(&musicxml, &meta_json, &glyph_names_json).expect("load_score failed");

        let render_once = || {
            render(
                false,
                Some("#ffffff".to_string()),
                Some("#000000".to_string()),
                None,
                None,
                None,
                None,
                1.0,
            )
            .expect("render failed")
        };

        let first = render_once();
        let second = render_once();

        assert_eq!(first.geometry, second.geometry);
        assert_eq!(first.text_blob, second.text_blob);
        assert_eq!(first.bounds_width, second.bounds_width);
        assert_eq!(first.bounds_height, second.bounds_height);
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
        let musicxml = fixture("xmlsamples/ActorPreludeSample.musicxml");
        let meta_json = fixture("smufl/bravura-bravura-1.392/redist/bravura_metadata.json");
        let glyph_names_json = fixture("smufl/metadata/glyphnames.json");

        load_score(&musicxml, &meta_json, &glyph_names_json).expect("load_score failed");

        let render_at = |device_pixel_ratio: f32| {
            render(
                false,
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
            unscaled.bounds_width as f64 * unscaled.bounds_height as f64;
        assert!(
            unscaled_physical_pixels <= MAX_CANVAS_PIXELS as f64,
            "test fixture is expected to already fit the budget at device_pixel_ratio 1.0, \
             got {unscaled_physical_pixels} physical pixels",
        );

        let huge_device_pixel_ratio = 1000.0;
        let scaled = render_at(huge_device_pixel_ratio);
        let scaled_physical_pixels = (scaled.bounds_width as f64 * huge_device_pixel_ratio as f64)
            * (scaled.bounds_height as f64 * huge_device_pixel_ratio as f64);

        assert!(
            scaled_physical_pixels <= MAX_CANVAS_PIXELS as f64 * 1.01, // float slop
            "expected the render at a huge device_pixel_ratio to stay within the canvas pixel \
             budget, got {scaled_physical_pixels} physical pixels",
        );
        assert!(
            scaled.bounds_width < unscaled.bounds_width,
            "expected element coordinates to actually shrink once the budget kicks in",
        );
    }
}
