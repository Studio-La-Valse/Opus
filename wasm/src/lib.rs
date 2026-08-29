use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::flat_buffer::FlatBufferCanvas;
use lib::drawable::drawable_element::{DrawableElement, compute_bounds, scale_elem};
use lib::geometry::color::Color;
use lib::score::app_defaults::AppDefaults;
use lib::score::engrave::{arrange_score, walk_document};
use lib::score::layout::Layout;
use lib::score::page_orientation::PageOrientation;
use lib::score::user_layout::UserLayout;
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::score::visual::score::Score;
use lib::smufl::smufl_font::SmuflFont;
use roxmltree::{Document, ParsingOptions};
use std::cell::RefCell;
use std::collections::HashMap;
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
}

/// Everything built from a MusicXML document that doesn't depend on
/// [`UserLayout`]. Built by [`load_score`] and stored under the handle it
/// returns; reused by every subsequent [`render`] call for that handle until
/// [`free_score`] drops it.
struct ScoreCache {
    layout: Layout,
    score: Score,
    font: SmuflFont,
}

// Keyed by handle rather than a single slot so that multiple independent
// scores (e.g. several `<music-xml>` elements on one page, each backed by
// this same wasm instance) can be loaded and re-rendered concurrently
// without one's `load_score` call evicting another's cache.
thread_local! {
    static CACHE: RefCell<HashMap<u32, ScoreCache>> = RefCell::new(HashMap::new());
    static NEXT_HANDLE: RefCell<u32> = const { RefCell::new(1) };
}

/// Canvas-ready render output. `geometry` is a tagged f32 stream and `text_blob`
/// holds every Text record's content in encounter order, joined by
/// [`lib::drawable::canvas::flat_buffer::TEXT_DELIMITER`]. See
/// [`lib::drawable::canvas::flat_buffer::FlatBuffer`] for the exact record layout.
#[wasm_bindgen]
pub struct RenderOutput {
    bounds_min_x: f32,
    bounds_min_y: f32,
    bounds_width: f32,
    bounds_height: f32,
    geometry: Vec<f32>,
    text_blob: String,
    font_blob: String,
    font_styles: Vec<f32>,
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

    /// The distinct font families every `TAG_TEXT` record's `fontIndex` points
    /// into, joined by [`lib::drawable::canvas::flat_buffer::TEXT_DELIMITER`].
    #[wasm_bindgen(getter)]
    pub fn font_blob(&mut self) -> String {
        std::mem::take(&mut self.font_blob)
    }

    /// Parallel to `font_blob`: per family, `FONT_STYLE_BOLD` / `FONT_STYLE_ITALIC`
    /// bits or'd together.
    #[wasm_bindgen(getter)]
    pub fn font_styles(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.font_styles)
    }
}

/// Parses `musicxml` and builds the [`Layout`]/[`Score`]/[`SmuflFont`] triple,
/// none of which depend on [`UserLayout`]. Caches the result under a new
/// handle so subsequent [`render`] calls for that handle can re-layout and
/// re-render without re-parsing or re-walking the document. Call
/// [`free_score`] with the returned handle once it's no longer needed.
#[wasm_bindgen]
pub fn load_score(musicxml: &str, meta_json: &str, glyph_names_json: &str) -> Result<u32, JsValue> {
    let font = SmuflFont::load(meta_json, glyph_names_json);

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(musicxml, options)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Only used to satisfy `WalkerCtx::new` during the walk passes; no visitor
    // reads it, since it doesn't affect the document's structure. Every
    // `render` call re-arranges the walked score for its real `UserLayout`.
    let user_layout = UserLayout::default();
    let app_defaults: AppDefaults = Default::default();

    let (score, layout) = walk_document(
        &document,
        &font,
        &user_layout,
        &app_defaults,
        &mut |_stage| {},
    );

    let handle = NEXT_HANDLE.with_borrow_mut(|next| {
        let handle = *next;
        *next += 1;
        handle
    });

    CACHE.with_borrow_mut(|cache| {
        cache.insert(
            handle,
            ScoreCache {
                layout,
                score,
                font,
            },
        );
    });

    Ok(handle)
}

/// Drops the score cached under `handle` by [`load_score`]. A no-op if the
/// handle doesn't exist (already freed, or never valid).
#[wasm_bindgen]
pub fn free_score(handle: u32) {
    CACHE.with_borrow_mut(|cache| {
        cache.remove(&handle);
    });
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
pub const MAX_CANVAS_PIXELS: f32 = 12_000_000.0;

/// Applies `UserLayout` to the [`Score`] cached under `handle` by
/// [`load_score`] and returns the resulting drawable elements. Does no XML
/// parsing or walking, so it's cheap to call on every layout-only change
/// (e.g. a page color tweak).
///
/// `device_pixel_ratio` (the browser's `window.devicePixelRatio`) is used
/// only to decide whether the result needs scaling down to stay within
/// [`MAX_CANVAS_PIXELS`]; the caller still multiplies the returned bounds by
/// it as usual when sizing the canvas backing store.
///
/// # Adding a `render_pdf` export
///
/// A PDF download would be a sibling `#[wasm_bindgen] pub fn render_pdf(handle:
/// u32, debug: bool, /* same layout args */) -> Result<Vec<u8>, JsValue>`
/// returning the bytes (wasm-bindgen marshals `Vec<u8>` to a `Uint8Array`).
/// Body: call [`lib::score::engrave::arrange_score`] on the cached score just
/// like [`render`] does, then instead of `compositor.walk(..)` +
/// `FlatBufferCanvas` do what `cli/src/commands/render.rs` does for
/// `OutputFormat::Pdf` -- `compositor.walk_pages(&cache.score, &fonts)`, one
/// `lib::drawable::canvas::pdf::PdfPageCanvas` per page, then
/// `lib::drawable::canvas::pdf::write_pdf(&pages, &font_set)`. The missing
/// piece is the font programs the [`lib::drawable::canvas::pdf::FontSet`]
/// embeds: there's no system font database in the browser. See the
/// `FontSource` seam note in [`lib::drawable::canvas::pdf`] for the shape --
/// in short, bundle Bravura (plus a text face) via `include_bytes!` or thread
/// the bytes through `load_score` and stash them in the cache next to
/// `SmuflFont`.
#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn render(
    handle: u32,
    debug: bool,
    page_color: Option<String>,
    foreground_color: Option<String>,
    page_orientation: Option<String>,
    horizontal_gutter_even: Option<f32>,
    horizontal_gutter_uneven: Option<f32>,
    vertical_gutter: Option<f32>,
    title_font: Option<String>,
    lyric_font: Option<String>,
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

    let title_font = title_font.as_deref().unwrap_or(&app_defaults.title_font);
    let lyric_font = lyric_font.as_deref().unwrap_or(&app_defaults.lyric_font);

    CACHE.with_borrow_mut(|cache| {
        let cache = cache.get_mut(&handle).ok_or_else(|| {
            JsValue::from_str("no score loaded for this handle; call load_score first")
        })?;

        arrange_score(
            &mut cache.score,
            &cache.layout,
            &user_layout,
            &app_defaults,
            &mut |_stage| {},
        );

        let fonts = RenderFonts::create(&cache.font, title_font, lyric_font);

        let mut elements: Vec<DrawableElement<'_>> =
            RenderCompositor::base().walk(&cache.score, &fonts);

        if debug {
            elements.extend(RenderCompositor::debug().walk(&cache.score, &fonts));
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

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint(&elements);
        Ok(RenderOutput {
            bounds_min_x: flat.bounds.0,
            bounds_min_y: flat.bounds.1,
            bounds_width: flat.bounds.2,
            bounds_height: flat.bounds.3,
            geometry: flat.geometry,
            text_blob: flat.text_blob,
            font_blob: flat.font_blob,
            font_styles: flat.font_styles,
        })
    })
}
