//! The engraving pipeline: turn an already-parsed MusicXML document into a
//! fully laid-out [`Score`].
//!
//! [`engrave`] runs the whole sequence. It is split into two reusable halves:
//! [`walk_document`] (the two document walks that build the [`Score`] and
//! [`ScoreDefaults`], independent of [`UserLayout`]) and [`arrange_score`] (resolve the
//! user layout, rebeam, measure and arrange the pages). The wasm bindings keep
//! these apart on purpose -- constructing a `Score` walks once and caches the
//! result, then every `render` on it re-runs only [`arrange_score`].
//!
//! The pipeline is presentation-free and platform-free: it reports progress
//! through a `progress: &mut dyn FnMut(Stage)` callback, never prints, and does
//! no timing of its own -- a caller that wants durations measures between
//! callbacks itself. That matters because `std::time::Instant` is unimplemented
//! on `wasm32-unknown-unknown` and panics on first use.
//!
//! Anything a caller wants to observe about the walk it supplies as `extra`, a
//! visitor chained after the built-in ones on the first pass. That is how the
//! CLI gets its `--debug` part-list dump
//! ([`crate::musicxml::visitors::part_list_log_visitor`]) without `lib` gaining
//! a print, and how the wasm bindings opt out of it by passing
//! [`DefaultVisitor`].

use std::collections::HashSet;

use roxmltree::Document;

use crate::geometry::xy::XY;
use crate::musicxml::visitor::{DefaultVisitor, Visitor};
use crate::musicxml::visitors::content_visitor::ContentVisitor;
use crate::musicxml::visitors::layout_visitor::LayoutVisitor;
use crate::musicxml::visitors::setup_visitor::SetupVisitor;
use crate::musicxml::visitors::tie_visitor::TieVisitor;
use crate::musicxml::visitors::walk_cursor_visitor::WalkCursorVisitor;
use crate::musicxml::walker::Walker;
use crate::musicxml::walker_ctx::WalkerCtx;
use crate::score::app_defaults::AppDefaults;
use crate::score::page_orientation::PageOrientation;
use crate::score::rebeam_strategy::{OnlyWhenRequiredRebeamStrategy, SimpleRebeamStrategy};
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layout_engine::{HorizontalPageLayout, LayoutEngine, VerticalPageLayout};
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::score::Score;
use crate::score::visual::tie_arranger::arrange_ties;
use crate::score::walk_cursor::WalkCursor;
use crate::smufl::smufl_font::SmuflFont;

/// A fully laid-out score together with the [`ScoreDefaults`] it was engraved against.
/// PDF output needs `layout.defaults` for its tenths-to-points scaling.
pub struct EngravedScore {
    pub score: Score,
    pub layout: ScoreDefaults,
}

/// A boundary the pipeline has just crossed, reported as soon as the step that
/// led up to it finishes. Consumers use this for progress reporting only; one
/// that wants wall-clock durations times the gaps between callbacks itself (see
/// `cli::commands::render`).
pub enum Stage {
    /// First document walk: layout context, part-list setup and staff layout.
    FirstPass,
    /// Second document walk: note / rest / clef content.
    SecondPass,
    /// Re-deriving beam groups.
    Rebeam,
    /// Resolving each element's appearance from the layout params, measuring
    /// every element and arranging the pages.
    LayoutPass,
}

/// Runs the full engraving sequence on an already-parsed `document`: the two
/// walk passes followed by layout resolution, rebeaming, measuring and page
/// arrangement. `progress` is invoked once at every [`Stage`] boundary, and
/// `extra` is the caller's own first-pass visitor ([`DefaultVisitor`] for a
/// caller that has none).
pub fn engrave<V>(
    document: &Document,
    font: &SmuflFont,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
    extra: V,
) -> EngravedScore
where
    V: for<'a> Visitor<WalkerCtx<'a>>,
{
    let (mut score, layout) =
        walk_document(document, font, user_layout, app_defaults, progress, extra);
    arrange_score(&mut score, &layout, user_layout, app_defaults, progress);
    EngravedScore { score, layout }
}

/// The two document walks that build the [`Score`] and [`ScoreDefaults`]. Nothing here
/// depends on [`UserLayout`] beyond satisfying `WalkerCtx::new`, so the result
/// can be cached and re-arranged for different user layouts.
///
/// Emits [`Stage::FirstPass`] and [`Stage::SecondPass`]. `extra` is chained
/// last on the first pass, so it observes a context the built-in visitors have
/// already filled in.
pub fn walk_document<V>(
    document: &Document,
    font: &SmuflFont,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
    extra: V,
) -> (Score, ScoreDefaults)
where
    V: for<'a> Visitor<WalkerCtx<'a>>,
{
    let mut cursor = WalkCursor::default();
    let mut layout = ScoreDefaults::default();
    let mut score = Score::default();

    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(SetupVisitor {})
        .uses(LayoutVisitor {
            encountered: HashSet::new(),
        })
        .uses(extra);
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::FirstPass);

    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(ContentVisitor::new())
        .uses(TieVisitor::new());
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::SecondPass);

    (score, layout)
}

/// Resolves the user layout onto an already-walked `score`, rebeams it, measures
/// every element and arranges the pages. Callable on its own to re-lay-out a
/// cached score for a new [`UserLayout`] without re-walking the document.
///
/// Emits [`Stage::Rebeam`] and [`Stage::LayoutPass`]. Layout resolution is no
/// longer its own pass -- each element resolves its appearance from the layout
/// params at the top of its `measure`, so it is folded into `LayoutPass`.
pub fn arrange_score(
    score: &mut Score,
    score_defaults: &ScoreDefaults,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
) {
    let strategy = OnlyWhenRequiredRebeamStrategy {
        inner: Box::new(SimpleRebeamStrategy {}),
    };
    score.rebeam(&strategy);

    progress(Stage::Rebeam);

    let params = LayoutParams {
        score_defaults,
        user_layout,
        app_defaults,
    };
    score.measure(&XY::INFINITE, params);
    page_layout_engine(user_layout, app_defaults).arrange_pages(score, &XY::ZERO);

    // Last: a tie's two endpoints can be measures, systems or pages apart, so it
    // is the one element that cannot be arranged until every note in the score
    // has its final position.
    arrange_ties(score, params);

    progress(Stage::LayoutPass);
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
