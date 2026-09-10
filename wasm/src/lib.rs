//! Browser bindings: one [`Score`] object per MusicXML document.
//!
//! There is no handle table and no module-global cache. A document's engraved
//! state lives in the object the caller constructs, so several `<music-xml>`
//! elements sharing this one wasm instance are independent by construction, and
//! wasm-bindgen's generated `free()` / `[Symbol.dispose]()` is the only
//! lifecycle the JS side has to think about.

use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::flat_buffer::FlatBufferCanvas;
use lib::drawable::drawable_element::{Scale, compute_bounds};
use lib::geometry::xy::XY;
use lib::score::app_defaults::AppDefaults;
use lib::score::engrave::{arrange_score, walk_document};
use lib::score::score_defaults::ScoreDefaults;
use lib::score::user_layout::UserLayout;
use lib::score::visual::render_compositor::RenderCompositor;
use lib::score::visual::render_fonts::RenderFonts;
use lib::score::visual::score;
use lib::smufl::smufl_font::SmuflFont;
use roxmltree::{Document, ParsingOptions};
use serde::Deserialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
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
    /// Parallel page table: 5 f32s per page --
    /// `[geometry_start_index, origin_x, origin_y, width, height]`. Page `i`'s
    /// records span `geometry[page_table[5i] .. page_table[5(i+1)]]` (the last
    /// page runs to `geometry.len()`). Coordinates are in the same space as
    /// `geometry` (global tenths, already `render_scale`-adjusted). JS may use
    /// this to slice the stream per page; the current single-canvas renderer
    /// ignores it.
    page_table: Vec<f32>,
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

    // Takes ownership of the buffer instead of cloning it: `_draw()` in
    // music-xml.js reads each property exactly once per RenderOutput before
    // calling `output.free()`, so there's no reason to pay for a second copy on
    // top of the copy wasm-bindgen already does when handing the Vec/String to JS.
    #[wasm_bindgen(getter)]
    pub fn geometry(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.geometry)
    }

    /// The page table (see the field docs). Empty for a zero-page score.
    #[wasm_bindgen(getter)]
    pub fn page_table(&mut self) -> Vec<f32> {
        std::mem::take(&mut self.page_table)
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

/// Everything [`Score::render`] takes, as one JS object rather than a
/// positional argument list.
///
/// The layout knobs are not enumerated here on purpose: `layout` deserializes
/// straight into [`UserLayout`], which is the single source of truth for that
/// option set, so a new knob is exposed to JS by adding the field there and
/// nothing else. Only the options that aren't part of the layout itself live
/// at this level.
///
/// `deny_unknown_fields` doesn't do much on the browser path -- serde-wasm-bindgen
/// deserializes a struct by looking up the field names it expects, so a key
/// this struct doesn't declare is never seen, let alone rejected. It still
/// holds for any other deserializer (the native tests use serde_json), and it
/// keeps the intent on record.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RenderOptions {
    /// Overlay the debug pass (bounding boxes, anchors, guides).
    pub debug: bool,
    /// The browser's `window.devicePixelRatio`, used only to decide whether the
    /// result needs scaling down to stay within [`MAX_CANVAS_PIXELS`]. Absent,
    /// non-finite or non-positive values are treated as 1.0.
    pub device_pixel_ratio: f32,
    /// Font family for titles / work-level text; falls back to the app default.
    pub title_font: Option<String>,
    /// Font family for lyrics; falls back to the app default.
    pub lyric_font: Option<String>,
    pub layout: UserLayout,
}

impl Default for RenderOptions {
    fn default() -> Self {
        RenderOptions {
            debug: false,
            // Not 0.0: an omitted ratio must mean "unscaled", and a zero would
            // otherwise divide the pixel budget into an infinite render scale.
            device_pixel_ratio: 1.0,
            title_font: None,
            lyric_font: None,
            layout: UserLayout::default(),
        }
    }
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

// Adding a `render_pdf` method
//
// A PDF download would be a sibling method on `Score` returning the bytes
// (wasm-bindgen marshals `Vec<u8>` to a `Uint8Array`). Body: call
// `lib::score::engrave::arrange_score` on the cached score just like `render`
// does, then instead of driving a `FlatBufferCanvas` over the composed pages do
// what `cli/src/commands/render/pdf.rs` does -- one
// `lib::drawable::canvas::pdf::PdfPageCanvas` per page, then
// `lib::drawable::canvas::pdf::write_pdf(&pages, &font_set)`. The missing piece
// is the font programs the `lib::drawable::canvas::pdf::FontSet` embeds:
// there's no system font database in the browser. See the `FontSource` seam
// note in `lib::drawable::canvas::pdf` for the shape -- in short, bundle
// Bravura (plus a text face) via `include_bytes!` or thread the bytes through
// the constructor and stash them next to `SmuflFont`.
//
// (Keep design notes like this one out of `///` doc comments: wasm-bindgen
// copies doc comments into the generated JSDoc, so a `*/` anywhere inside one
// closes the comment early and leaves wasm.js syntactically invalid.)

/// One MusicXML document, walked once on construction and re-arrangeable for
/// any number of different [`RenderOptions`].
///
/// The split matches [`lib::score::engrave`]'s two halves: the constructor runs
/// [`walk_document`] (parse + the two visitor passes, none of which depend on
/// the user layout) and each [`Score::render`] re-runs only [`arrange_score`],
/// so a layout-only change such as a page-colour tweak never re-parses the XML.
#[wasm_bindgen(js_name = Score)]
pub struct WasmScore {
    defaults: ScoreDefaults,
    score: score::Score,
    font: SmuflFont,
}

#[wasm_bindgen(js_class = Score)]
impl WasmScore {
    /// Parses `musicxml` and walks it into a laid-out-on-demand score. The two
    /// JSON arguments are the SMuFL metadata and glyph-name tables.
    #[wasm_bindgen(constructor)]
    pub fn new(
        musicxml: &str,
        meta_json: &str,
        glyph_names_json: &str,
    ) -> Result<WasmScore, JsValue> {
        let font = SmuflFont::load(meta_json, glyph_names_json);

        let options = ParsingOptions {
            allow_dtd: true,
            ..ParsingOptions::default()
        };
        let document = Document::parse_with_options(musicxml, options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // Only used to satisfy `WalkerCtx::new` during the walk passes; no
        // visitor reads it, since it doesn't affect the document's structure.
        // Every `render` call re-arranges the walked score for its real layout.
        let user_layout = UserLayout::default();
        let app_defaults: AppDefaults = Default::default();

        // The walk's log messages are dropped: the browser has nowhere to show
        // them, and nothing in the render path reads them back.
        let (score, defaults, _messages) = walk_document(
            &document,
            &font,
            &user_layout,
            &app_defaults,
            &mut |_stage| {},
        );

        Ok(WasmScore {
            defaults,
            score,
            font,
        })
    }

    /// Lays the score out for `options` and returns the resulting drawable
    /// elements. Does no XML parsing or walking, so it's cheap to call on every
    /// layout-only change (e.g. a page colour tweak).
    ///
    /// `options` is a plain JS object; see [`RenderOptions`], whose `layout`
    /// member accepts every field of
    /// [`UserLayout`](lib::score::user_layout::UserLayout) in camelCase.
    pub fn render(&mut self, options: JsValue) -> Result<RenderOutput, JsValue> {
        let options: RenderOptions = if options.is_undefined() || options.is_null() {
            RenderOptions::default()
        } else {
            serde_wasm_bindgen::from_value(options)?
        };

        Ok(self.render_with(&options))
    }
}

impl WasmScore {
    /// The body of [`WasmScore::render`] once the options have been decoded.
    /// Separate so Rust callers (the test crate) don't have to go through a
    /// `JsValue`.
    pub fn render_with(&mut self, options: &RenderOptions) -> RenderOutput {
        let app_defaults: AppDefaults = Default::default();

        arrange_score(
            &mut self.score,
            &self.defaults,
            &self.font,
            &options.layout,
            &app_defaults,
            &mut |_stage| {},
        );

        let title_font = options
            .title_font
            .as_deref()
            .unwrap_or(&app_defaults.title_font);
        let lyric_font = options
            .lyric_font
            .as_deref()
            .unwrap_or(&app_defaults.lyric_font);
        let fonts = RenderFonts::create(&self.font, title_font, lyric_font);

        // One page-preserving walk; the flat buffer concatenates the pages into
        // its single stream but records each page's boundary in `page_table`.
        let mut pages = RenderCompositor::compose(&self.score, &fonts, options.debug);

        // A ratio JS couldn't supply sensibly (absent, NaN, zero) means
        // "unscaled" rather than an unbounded scale factor.
        let device_pixel_ratio =
            if options.device_pixel_ratio.is_finite() && options.device_pixel_ratio > 0.0 {
                options.device_pixel_ratio
            } else {
                1.0
            };

        let (min_x, min_y, max_x, max_y) = compute_bounds(pages.iter().flat_map(|p| &p.elements));
        let physical_width = (max_x - min_x) * device_pixel_ratio;
        let physical_height = (max_y - min_y) * device_pixel_ratio;
        let render_scale = if physical_width > 0.0 && physical_height > 0.0 {
            (MAX_CANVAS_PIXELS / (physical_width * physical_height))
                .sqrt()
                .min(1.0)
        } else {
            1.0
        };

        // Scale each page's elements *and* its origin/size, so the page table
        // stays consistent with the down-scaled geometry.
        if render_scale < 1.0 {
            for page in &mut pages {
                page.elements = page
                    .elements
                    .iter()
                    .map(|el| el.scale(render_scale, XY::ZERO))
                    .collect();
                page.origin = page.origin.scale(render_scale);
                page.width *= render_scale;
                page.height *= render_scale;
            }
        }

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint_pages(&pages);
        RenderOutput {
            bounds_min_x: flat.bounds.0,
            bounds_min_y: flat.bounds.1,
            bounds_width: flat.bounds.2,
            bounds_height: flat.bounds.3,
            geometry: flat.geometry,
            page_table: flat.page_table,
            text_blob: flat.text_blob,
            font_blob: flat.font_blob,
            font_styles: flat.font_styles,
        }
    }
}
