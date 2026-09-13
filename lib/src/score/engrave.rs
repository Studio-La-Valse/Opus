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
//! Narrating a walk is not this pipeline's job either: [`BuildLoggingVisitor`]
//! rides along on the first pass and reports through the context, and
//! [`walk_document`] hands those messages back for the caller to print or drop.

use std::collections::HashSet;

use roxmltree::Document;

use crate::geometry::xy::XY;
use crate::musicxml::validation_issue::ValidationIssue;
use crate::musicxml::visitor::{DefaultVisitor, Visitor};
use crate::musicxml::visitors::build_logging_visitor::BuildLoggingVisitor;
use crate::musicxml::visitors::clef_visitor::ClefVisitor;
use crate::musicxml::visitors::content_visitor::ContentVisitor;
use crate::musicxml::visitors::layout_visitor::LayoutVisitor;
use crate::musicxml::visitors::print_layout_visitor::PrintLayoutVisitor;
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
use crate::score::visual::score_arranger::SCORE_ARRANGERS;
use crate::score::walk_cursor::WalkCursor;
use crate::smufl::smufl_font::SmuflFont;

/// A fully laid-out score together with the [`ScoreDefaults`] it was engraved against.
/// PDF output needs `layout.defaults` for its tenths-to-points scaling.
pub struct EngravedScore {
    pub score: Score,
    pub layout: ScoreDefaults,
    /// What the walk had to say for itself, from
    /// [`crate::musicxml::visitors::build_logging_visitor`]. Informational only;
    /// a caller is free to print or drop them.
    pub messages: Vec<ValidationIssue>,
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
    /// Resolving every element's appearance -- colour, thickness, which glyph
    /// it draws -- from the layout params.
    ResolveLayout,
    /// Measuring every element and arranging the pages.
    LayoutPass,
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
    let (mut score, layout, messages) =
        walk_document(document, font, user_layout, app_defaults, progress);
    arrange_score(
        &mut score,
        &layout,
        font,
        user_layout,
        app_defaults,
        progress,
    );
    EngravedScore {
        score,
        layout,
        messages,
    }
}

/// The two document walks that build the [`Score`] and [`ScoreDefaults`]. Nothing here
/// depends on [`UserLayout`] beyond satisfying `WalkerCtx::new`, so the result
/// can be cached and re-arranged for different user layouts.
///
/// Emits [`Stage::FirstPass`] and [`Stage::SecondPass`], and returns whatever
/// [`BuildLoggingVisitor`] had to say about the walk.
pub fn walk_document(
    document: &Document,
    font: &SmuflFont,
    user_layout: &UserLayout,
    app_defaults: &AppDefaults,
    progress: &mut dyn FnMut(Stage),
) -> (Score, ScoreDefaults, Vec<ValidationIssue>) {
    let mut cursor = WalkCursor::default();
    let mut layout = ScoreDefaults::default();
    let mut score = Score::default();
    let mut messages = Vec::new();

    // The logger is chained last so that what it reports is what the visitors
    // ahead of it have already produced.
    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(SetupVisitor {})
        .uses(PrintLayoutVisitor {})
        .uses(LayoutVisitor {
            encountered: HashSet::new(),
        })
        .uses(BuildLoggingVisitor::default());
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
        &mut messages,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::FirstPass);

    let visitor = DefaultVisitor {}
        .uses(WalkCursorVisitor {})
        .uses(ContentVisitor::new())
        .uses(TieVisitor::new())
        .uses(ClefVisitor::new());
    let mut ctx = WalkerCtx::new(
        user_layout,
        &mut layout,
        app_defaults,
        &mut cursor,
        &mut score,
        font,
        &mut messages,
    );
    Walker::new(visitor).walk(document, &mut ctx);

    progress(Stage::SecondPass);

    (score, layout, messages)
}

/// Resolves the user layout onto an already-walked `score`, rebeams it, resolves
/// every element's appearance, measures every element and arranges the pages.
/// Callable on its own to re-lay-out a cached score for a new [`UserLayout`]
/// without re-walking the document -- which is why `font` is a parameter here as
/// well as on [`walk_document`]: an element whose glyph depends on the user
/// layout has to be able to look it up on every arrange, not once during the
/// walk.
///
/// Emits [`Stage::Rebeam`], [`Stage::ResolveLayout`] and [`Stage::LayoutPass`].
/// Appearance resolution is its own downward pass ahead of measuring -- see
/// [`Layoutable::resolve_layout`](crate::score::visual::layoutable::Layoutable::resolve_layout).
pub fn arrange_score(
    score: &mut Score,
    score_defaults: &ScoreDefaults,
    font: &SmuflFont,
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
        font,
        // The first system of the score names in full; `System::resolve_layout`
        // re-stamps this from each system's own index.
        abbreviate_names: false,
    };
    score.resolve_layout(params);

    progress(Stage::ResolveLayout);

    score.measure(&XY::INFINITE, params);
    page_layout_engine(user_layout, app_defaults).arrange_pages(score, &XY::ZERO);

    // Last: every coordinate in the tree is now absolute, which is what beams,
    // ties and mid-measure clef changes need. See `SCORE_ARRANGERS` for why
    // they run in this order.
    for arranger in SCORE_ARRANGERS {
        arranger.arrange(score, params);
    }

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
