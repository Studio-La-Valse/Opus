use crate::commands::print_issues;
use clap::Args;
use lib::drawable::drawable_element::{DrawableElement, to_svg};
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
use lib::xml::validate::ValidationCtx;
use lib::xml::validation_issue::{Severity, ValidationIssue};
use lib::xml::visitor::{DefaultVisitor, Visitor};
use lib::xml::visitors::content_visitor::ContentVisitor;
use lib::xml::visitors::layout_ctx_visitor::LayoutContextVisitor;
use lib::xml::visitors::layout_visitor::LayoutVisitor;
use lib::xml::visitors::part_consistency_visitor::PartConsistencyVisitor;
use lib::xml::visitors::position_visitor::PositionVisitor;
use lib::xml::visitors::setup_visitor::SetupVisitor;
use lib::xml::walker::Walker;
use lib::xml::walker_ctx::WalkerCtx;
use roxmltree::{Document, ParsingOptions};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::read_to_string;
use std::time::Instant;

#[derive(Args, Debug)]
pub struct RenderArgs {
    #[arg(long)]
    file: String,

    #[arg(long)]
    out: String,

    #[arg(long)]
    meta: String,

    #[arg(long)]
    glyphs: String,

    #[arg(long, short, action)]
    debug: bool,

    #[arg(long)]
    page_color: Option<Color>,

    #[arg(long)]
    foreground_color: Option<Color>,

    #[arg(long)]
    page_orientation: Option<PageOrientation>,

    #[arg(long)]
    horizontal_gutter_even: Option<f32>,

    #[arg(long)]
    horizontal_gutter_uneven: Option<f32>,

    #[arg(long)]
    vertical_gutter: Option<f32>,
}

pub fn run(args: RenderArgs) {
    let file = args.file;
    let out = args.out;
    let meta = args.meta;
    let glyph_names = args.glyphs;
    let debug = args.debug;
    let page_color = args.page_color;
    let foreground_color = args.foreground_color;
    let page_orientation = args.page_orientation;
    let horizontal_gutter_even = args.horizontal_gutter_even;
    let horizontal_gutter_uneven = args.horizontal_gutter_uneven;
    let vertical_gutter = args.vertical_gutter;

    let mut time = Instant::now();

    let data = read_to_string(file).expect("Something went wrong reading the file");
    let meta_content = read_to_string(&meta).expect("Cannot read metadata.json");
    let glyph_names_content = read_to_string(&glyph_names).expect("Cannot read glyphnames.json");
    let font = SmuflFont::load(&meta_content, &glyph_names_content);

    println!("Reading to string: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };

    let document = Document::parse_with_options(&data, options).unwrap();

    println!("Parsing doc tree: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let mut validation_ctx = ValidationCtx::default();

    let root = document.root_element();
    if root.tag_name().name() != "score-partwise" {
        validation_ctx.issues.push(ValidationIssue {
            severity: Severity::Error,
            message: format!(
                "Expected root element <score-partwise>, found <{}>",
                root.tag_name().name()
            ),
            at: root.range().start,
        });
    } else {
        let visitor = DefaultVisitor {}
            .uses(PartConsistencyVisitor::default())
            .uses(PositionVisitor::default());

        Walker::new(visitor).walk(&document, &mut validation_ctx);
    }

    print_issues(&document, &validation_ctx.issues);

    println!("Validation: {}ms", time.elapsed().as_millis());
    time = Instant::now();

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

    println!(
        "First read pass: walking doc tree for layout: {}ms",
        time.elapsed().as_millis()
    );
    time = Instant::now();

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

    println!(
        "Second read pass: walking doc tree for content: {}ms",
        time.elapsed().as_millis()
    );
    time = Instant::now();

    visual.apply_layout(&layout, &user_layout, &app_defaults);

    println!("Applying user layout: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let strat_impl = Box::new(SimpleRebeamStrategy {});
    let strategy = Box::new(OnlyWhenRequiredRebeamStrategy { imp: strat_impl });

    visual.rebeam(strategy.as_ref());

    println!("Rebeaming: {}ms", time.elapsed().as_millis());
    time = Instant::now();

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

    println!("Layout pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    let pass = BaseRenderer {};
    let compositor = RenderCompositor {
        pass: Box::new(pass),
    };
    let mut elements: Vec<DrawableElement<'_>> = compositor.walk(&visual, &font);

    println!("First render pass: {}ms", time.elapsed().as_millis());
    time = Instant::now();

    if debug {
        let pass = DebugRenderer {};
        let compositor = RenderCompositor {
            pass: Box::new(pass),
        };
        elements.extend(compositor.walk(&visual, &font));

        println!("Second render pass: {}ms", time.elapsed().as_millis());
        time = Instant::now();
    }

    let svg = to_svg(elements);
    fs::write(out, svg).unwrap();

    println!("Write to svg: {}ms", time.elapsed().as_millis());
}
