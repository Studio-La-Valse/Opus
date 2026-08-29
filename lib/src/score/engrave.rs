//! The engraving pipeline: turn an already-parsed MusicXML document into a
//! fully laid-out [`Score`].
//!
//! [`engrave`] runs the whole sequence. It is split into two reusable halves:
//! [`walk_document`] (the two document walks that build the [`Score`] and
//! [`ScoreDefaults`], independent of [`UserLayout`]) and [`arrange_score`] (resolve the
//! user layout, rebeam, measure and arrange the pages). The wasm bindings keep
//! these apart on purpose -- `load_score` walks once and caches the result, then
//! every `render` re-runs only [`arrange_score`].
//!
//! The pipeline is presentation-free: it reports progress through a
//! `progress: &mut dyn FnMut(Stage)` callback and never prints.

use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};

use roxmltree::Document;

use crate::drawable::layoutable::Layoutable;
use crate::geometry::xy::XY;
use crate::musicxml::visitor::{DefaultVisitor, Visitor};
use crate::musicxml::visitors::content_visitor::ContentVisitor;
use crate::musicxml::visitors::layout_visitor::LayoutVisitor;
use crate::musicxml::visitors::setup_visitor::SetupVisitor;
use crate::musicxml::visitors::walk_cursor_visitor::WalkCursorVisitor;
use crate::musicxml::walker::Walker;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::app_defaults::AppDefaults;
use crate::score::page_orientation::PageOrientation;
use crate::score::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layout_engine::{HorizontalPageLayout, LayoutEngine, VerticalPageLayout};
use crate::score::visual::score::Score;
use crate::score::visual::score_element::ScoreElement;
use crate::score::walk_cursor::WalkCursor;
use crate::smufl::smufl_font::SmuflFont;

/// A fully laid-out score together with the [`ScoreDefaults`] it was engraved against.
/// PDF output needs `layout.defaults` for its tenths-to-points scaling.
pub struct EngravedScore {
    pub score: Score,
    pub layout: ScoreDefaults,
}

/// A boundary the pipeline has just crossed, carrying the wall-clock time spent
/// in the step that led up to it. Consumers use this for progress reporting
/// only.
pub enum Stage {
    /// First document walk: layout context, part-list setup and staff layout.
    FirstPass(Duration),
    /// Second document walk: note / rest / clef content.
    SecondPass(Duration),
    /// Resolving [`UserLayout`] and [`AppDefaults`] onto the score.
    ApplyLayout(Duration),
    /// Re-deriving beam groups.
    Rebeam(Duration),
    /// Measuring every element and arranging the pages.
    LayoutPass(Duration),
}

/// Runs the full engraving sequence on an already-parsed `document`: the two
/// walk passes followed by layout resolution, rebeaming, measuring and page
/// arrangement. `progress` is invoked once at every [`Stage`] boundary.
pub fn engrave(
    document: &Document,
    font: &SmuflFont,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
) -> EngravedScore {
    let (mut score, layout) = walk_document(document, font, user_layout, app_defaults, progress);
    arrange_score(&mut score, &layout, user_layout, app_defaults, progress);
    EngravedScore { score, layout }
}

/// The two document walks that build the [`Score`] and [`ScoreDefaults`]. Nothing here
/// depends on [`UserLayout`] beyond satisfying `WalkerCtx::new`, so the result
/// can be cached and re-arranged for different user layouts.
///
/// Emits [`Stage::FirstPass`] and [`Stage::SecondPass`].
pub fn walk_document(
    document: &Document,
    font: &SmuflFont,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
) -> (Score, ScoreDefaults) {
    let mut cursor = WalkCursor::default();
    let mut layout = ScoreDefaults::default();
    let mut score = Score::default();

    let mut time = Instant::now();

    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(SetupVisitor {})
        .uses(LayoutVisitor {
            encountered: HashSet::new(),
        });
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::FirstPass(time.elapsed()));
    time = Instant::now();

    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(ContentVisitor {
            clef_change: HashMap::new(),
        });
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::SecondPass(time.elapsed()));

    (score, layout)
}

/// Resolves the user layout onto an already-walked `score`, rebeams it, measures
/// every element and arranges the pages. Callable on its own to re-lay-out a
/// cached score for a new [`UserLayout`] without re-walking the document.
///
/// Emits [`Stage::ApplyLayout`], [`Stage::Rebeam`] and [`Stage::LayoutPass`].
pub fn arrange_score(
    score: &mut Score,
    layout: &ScoreDefaults,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
) {
    let mut time = Instant::now();

    score.apply_layout(layout, user_layout, app_defaults);

    progress(Stage::ApplyLayout(time.elapsed()));
    time = Instant::now();

    let strategy = OnlyWhenRequiredRebeamStrategy {
        inner: Box::new(SimpleRebeamStrategy {}),
    };
    score.rebeam(&strategy);

    progress(Stage::Rebeam(time.elapsed()));
    time = Instant::now();

    score.measure(&XY::INFINITE);
    page_layout_engine(user_layout, app_defaults).arrange_pages(score, &XY::ZERO);

    progress(Stage::LayoutPass(time.elapsed()));
}

/// Picks the page-layout engine for the effective [`PageOrientation`], resolving
/// each gutter against `user` then `defaults`.
fn page_layout_engine(user: &UserLayout, defaults: &AppDefaults) -> Box<dyn LayoutEngine> {
    let orientation = user.page_orientation.unwrap_or(defaults.page_orientation);
    match orientation {
        PageOrientation::Horizontal => Box::new(HorizontalPageLayout {
            gutter_even: user
                .horizontal_gutter_even
                .unwrap_or(defaults.horizontal_gutter_even),
            gutter_uneven: user
                .horizontal_gutter_uneven
                .unwrap_or(defaults.horizontal_gutter_uneven),
        }),
        PageOrientation::Vertical => Box::new(VerticalPageLayout {
            gutter: user.vertical_gutter.unwrap_or(defaults.vertical_gutter),
        }),
    }
}
