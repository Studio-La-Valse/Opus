use lib::drawable::drawable_element::DrawableElement;
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
use std::collections::{HashMap, HashSet};
use std::str::FromStr;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
fn init() {
    console_error_panic_hook::set_once();
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

    #[wasm_bindgen(getter)]
    pub fn geometry(&self) -> Vec<f32> {
        self.geometry.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn text_blob(&self) -> String {
        self.text_blob.clone()
    }
}

#[allow(clippy::too_many_arguments)]
#[wasm_bindgen]
pub fn render_elements(
    musicxml: &str,
    meta_json: &str,
    glyph_names_json: &str,
    debug: bool,
    page_color: Option<String>,
    foreground_color: Option<String>,
    page_orientation: Option<String>,
    horizontal_gutter_even: Option<f32>,
    horizontal_gutter_uneven: Option<f32>,
    vertical_gutter: Option<f32>,
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

    let font = SmuflFont::load(meta_json, glyph_names_json);

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(musicxml, options)
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

    let mut layout_ctx = LayoutCtx::default();
    let mut layout = Layout::default();
    let mut visual = Score::default();

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
        &mut visual,
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
        &mut visual,
        &font,
    );
    Walker::new(visitor).walk(&document, &mut ctx);

    visual.apply_layout(&layout, &user_layout, &app_defaults);

    let strat_impl = Box::new(SimpleRebeamStrategy {});
    let strategy = Box::new(OnlyWhenRequiredRebeamStrategy { imp: strat_impl });
    visual.rebeam(strategy.as_ref());

    visual.measure(&XY::INFINITE);

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
    layout_engine.arrange_pages(&mut visual, &XY::ZERO);

    let pass = BaseRenderer {};
    let compositor = RenderCompositor {
        pass: Box::new(pass),
    };
    let mut elements: Vec<DrawableElement<'_>> = compositor.walk(&visual, &font);

    if debug {
        let pass = DebugRenderer {};
        let compositor = RenderCompositor {
            pass: Box::new(pass),
        };
        elements.extend(compositor.walk(&visual, &font));
    }

    let flat = to_flat_buffer(elements);
    Ok(RenderOutput {
        bounds_min_x: flat.bounds.0,
        bounds_min_y: flat.bounds.1,
        bounds_width: flat.bounds.2,
        bounds_height: flat.bounds.3,
        geometry: flat.geometry,
        text_blob: flat.text_blob,
    })
}
