use lib::drawable::element::Element;
use lib::layout::{Layout, UserLayout};
use lib::layout_ctx::LayoutCtx;
use lib::score::drawable::bfs_iter::bfs_elements;
use lib::visitor::{DefaultVisitor, Visitor};
use lib::visitors::content_visitor::ContentVisitor;
use lib::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::visitors::layout_visitor::LayoutVisitor;
use lib::visual::layoutable::Layoutable;
use lib::visual::score::Score;
use lib::walker::Walker;
use lib::walker_ctx::WalkerCtx;
use lib::xy::XY;
use roxmltree::{Document, ParsingOptions};
use std::cell::RefCell;
use wasm_bindgen::JsValue;
use wasm_bindgen::prelude::wasm_bindgen;

thread_local! {
    // 1. We must keep the raw XML string alive alongside the Document
    static XML_DATA: RefCell<Option<String>> = const { RefCell::new(None) };

    // 2. We use 'static here because the borrowed string will live for the lifetime of the thread
    static DOC: RefCell<Option<Document<'static>>> = const { RefCell::new(None) };

    // 3. Storing the computed visual score layout
    static VISUAL: RefCell<Option<Score>> = const { RefCell::new(None) };
}

#[wasm_bindgen]
pub fn load_xml(xml: String) {
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    XML_DATA.with(|xml_cell| {
        let mut xml_guard = xml_cell.borrow_mut();

        // Store the string permanently in the thread-local storage
        let xml_ref = xml_guard.insert(xml);

        // Transmute the lifetime of the string reference to 'static.
        // This is safe because XML_DATA and DOC share the exact same thread-local lifetime.
        let static_str: &'static str = unsafe { std::mem::transmute(xml_ref.as_str()) };

        let doc = Document::parse_with_options(static_str, options).unwrap();

        DOC.with(|doc_cell| {
            *doc_cell.borrow_mut() = Some(doc);
        });
    });
}

#[wasm_bindgen]
pub fn compute_layout() {
    DOC.with(|doc_cell| {
        let doc_borrow = doc_cell.borrow();
        let doc = doc_borrow
            .as_ref()
            .expect("Document not loaded! Call load_xml first.");

        let mut layout_ctx = LayoutCtx::default();
        let mut user_layout = UserLayout::default();
        let mut layout = Layout::default();
        let mut visual = Score::default();

        let visitor = DefaultVisitor {}
            .add_callback(LayoutVisitor {})
            .add_callback(LayoutContextVisitor {})
            .add_callback(ContentVisitor {
                staff_measures: Default::default(),
            });

        let mut ctx = WalkerCtx::new(&mut user_layout, &mut layout, &mut layout_ctx, &mut visual);
        Walker::new(visitor).walk(doc, &mut ctx);

        visual.measure(&XY::INFINITE);
        visual.arrange(&XY::ZERO);

        VISUAL.with(|visual_cell| {
            *visual_cell.borrow_mut() = Some(visual);
        });
    });
}

#[wasm_bindgen]
pub fn export_elements() -> JsValue {
    VISUAL.with(|visual_cell| {
        let visual_borrow = visual_cell.borrow();
        let visual = visual_borrow
            .as_ref()
            .expect("Layout not computed! Call compute_layout first.");

        let elements: Vec<Element> = bfs_elements(visual).collect();

        // Directly convert the vector into native JavaScript Arrays/Objects
        serde_wasm_bindgen::to_value(&elements).unwrap()
    })
}
