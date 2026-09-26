//! Browser bindings: one [`Score`] object per MusicXML document.
//!
//! There is no handle table and no module-global cache. A document's engraved
//! state lives in the object the caller constructs, so several `<music-xml>`
//! elements sharing this one wasm instance are independent by construction, and
//! wasm-bindgen's generated `free()` / `[Symbol.dispose]()` is the only
//! lifecycle the JS side has to think about.

use lib::drawable::canvas::CanvasPainter;
use lib::drawable::canvas::flat_buffer::FlatBufferCanvas;
use lib::score::engrave::{arrange_score, walk_document};
use lib::score::layout_options::UserLayout;
use lib::score::score_defaults::ScoreDefaults;
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
    geometry: Vec<f32>,
    /// Parallel page table: 4 f32s per page --
    /// `[geometry_start_index, text_start_index, width, height]`. Page `i`'s
    /// geometry records span `geometry[page_table[4i] .. page_table[4(i+1)]]`
    /// (the last page runs to `geometry.len()`). `text_start_index` is the
    /// number of `text_blob` entries emitted by earlier pages: `TAG_TEXT` and
    /// `TAG_GLYPH` records pull from `text_blob` in stream order, so painting
    /// page `i` on its own means skipping that many entries first. `width` /
    /// `height` are that page's own size, in tenths -- pages are engraved
    /// page-local, so there is no page origin to record.
    page_table: Vec<f32>,
    text_blob: String,
    font_blob: String,
    font_styles: Vec<f32>,
}

#[wasm_bindgen]
impl RenderOutput {
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
/// `layout` is a single, already-merged [`UserLayout`]: a caller with several
/// sources of overrides (say, a layout file and per-element tweaks) layers them
/// with [`UserLayout::overlay`] before handing them over, exactly as the CLI
/// layers its flags on top of `--layout`.
///
/// `deny_unknown_fields` doesn't do much on the browser path -- serde-wasm-bindgen
/// deserializes a struct by looking up the field names it expects, so a key
/// this struct doesn't declare is never seen, let alone rejected. It still
/// holds for any other deserializer (the native tests use serde_json), and it
/// keeps the intent on record.
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase", default, deny_unknown_fields)]
pub struct RenderOptions {
    /// Overlay the debug pass (bounding boxes, anchors, guides).
    pub debug: bool,
    pub layout: UserLayout,
}

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
    /// Parses `musicxml` and walks it into a laid-out-on-demand score.
    /// `meta_json` is the SMuFL font's metadata file.
    #[wasm_bindgen(constructor)]
    pub fn new(musicxml: &str, meta_json: &str) -> Result<WasmScore, JsValue> {
        let font = SmuflFont::load(meta_json);

        let options = ParsingOptions {
            allow_dtd: true,
            ..ParsingOptions::default()
        };
        let document = Document::parse_with_options(musicxml, options)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;

        // The walk's log messages are dropped: the browser has nowhere to show
        // them, and nothing in the render path reads them back.
        let (score, defaults, _messages) = walk_document(&document, &mut |_stage| {});

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
    /// member accepts [`UserLayout`] as nested snake_case objects
    /// (`{ tie: { height_max: 14 } }`).
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
        let layout = &options.layout;

        arrange_score(
            &mut self.score,
            &self.defaults,
            &self.font,
            layout,
            &mut |_stage| {},
        );

        let fonts = RenderFonts::resolve(&self.font, layout);

        // One page-preserving walk; the flat buffer concatenates the pages into
        // its single stream but records each page's boundary in `page_table`.
        // Each page is already sized and positioned page-local, so there's no
        // pixel budget to enforce here -- the browser decides how large a page
        // is displayed, and so how large a backing store it needs.
        let pages = RenderCompositor::compose(&self.score, &fonts, options.debug);

        let flat = CanvasPainter::new(FlatBufferCanvas::new()).paint_pages(&pages);
        RenderOutput {
            geometry: flat.geometry,
            page_table: flat.page_table,
            text_blob: flat.text_blob,
            font_blob: flat.font_blob,
            font_styles: flat.font_styles,
        }
    }
}
