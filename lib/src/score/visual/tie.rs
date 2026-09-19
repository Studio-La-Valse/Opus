//! Ties: the pairing recorded during the content walk, and the pure geometry
//! that turns a pair of endpoints into a drawable arc.
//!
//! Nothing here touches the score tree --
//! [`TieArranger`](crate::score::visual::arranger::TieArranger) does that.
//! Keeping the geometry as free functions over [`XY`] is what lets the
//! cross-system case be tested without a multi-system fixture, and is the seam
//! a future slur implementation reuses: a slur is the same arc between
//! different anchors.

use std::collections::HashMap;

use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::{NoteAnchor, NoteId};
use crate::score::visual::stem::UpDown;
use crate::score::visual::system::{SystemExtent, SystemKey};

/// How many points each of the arc's two edges is sampled at. Bumping this
/// trades output size for smoothness; see the note on flattening in
/// [`tie_arc`].
const TIE_SAMPLES: usize = 16;

/// How far each end pulls its Bezier handle along the arc, as a fraction of the
/// arc's horizontal span. The classic "one third in" that makes a cubic read as
/// a circular arc.
const SHOULDER: f32 = 1. / 3.;

/// The staff-line index of the middle staff line. Indices run 0 (top line) to 9
/// (bottom line) in half-space steps; see `LEDGER_ABOVE_STAFF_LINE` /
/// `LEDGER_BELOW_STAFF_LINE` in
/// [`part_measure`](crate::score::visual::part_measure).
const MIDDLE_STAFF_LINE: i32 = 4;

/// Which side of the noteheads a tie arcs over.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum TieSide {
    /// Bulges upward, away from the noteheads. Negative y, since y grows
    /// downward everywhere in this codebase.
    Over,
    /// Bulges downward.
    Under,
}

impl TieSide {
    /// `-1.0` for [`TieSide::Over`], `+1.0` for [`TieSide::Under`]: multiply a
    /// magnitude by this to get a displacement in the arc's bulge direction.
    pub fn sign(self) -> f32 {
        match self {
            TieSide::Over => -1.,
            TieSide::Under => 1.,
        }
    }

    /// Parses MusicXML's `<tied orientation="over|under">`, falling back to
    /// `placement="above|below"`. Returns `None` for anything else, meaning
    /// "infer it".
    pub fn parse(orientation: Option<&str>, placement: Option<&str>) -> Option<Self> {
        match orientation {
            Some("over") => return Some(TieSide::Over),
            Some("under") => return Some(TieSide::Under),
            _ => {}
        }

        match placement {
            Some("above") => Some(TieSide::Over),
            Some("below") => Some(TieSide::Under),
            _ => None,
        }
    }

    /// The side to use when the document gives no orientation.
    ///
    /// A tie curves away from the stem, so a stem-up chord ties under and a
    /// stem-down chord ties over. A stemless note (a whole note) has no stem to
    /// curve away from, so it curves away from the middle staff line instead.
    ///
    /// Deliberately simple: in a chord where several notes are tied, engraving
    /// practice puts the top note's tie over and the bottom note's under
    /// regardless of the stem, with the inner ones following. Applying the
    /// stem rule uniformly instead gives a parallel stack, which is acceptable
    /// and is what many engravers do for inner voices. Refining this needs the
    /// whole chord, not one note, so it belongs with the caller.
    pub fn infer(stem: Option<UpDown>, staff_line: i32) -> Self {
        match stem {
            Some(UpDown::Up) => TieSide::Under,
            Some(UpDown::Down) => TieSide::Over,
            None => {
                if staff_line < MIDDLE_STAFF_LINE {
                    TieSide::Over
                } else {
                    TieSide::Under
                }
            }
        }
    }
}

/// A tie between two notes, recorded during the content walk.
///
/// Purely a pairing -- it holds no geometry. The drawn arcs are derived after
/// the pages are arranged, by
/// [`TieArranger`](crate::score::visual::arranger::TieArranger), because
/// that is the first moment both endpoints have absolute coordinates.
pub struct Tie {
    pub start: NoteId,
    pub end: NoteId,
    /// From MusicXML `<tied orientation=…>` / `placement=…`. `None` means
    /// "infer from the start note" -- see [`TieSide::infer`].
    pub side: Option<TieSide>,
}

/// One drawn arc, in absolute (page-local tenths) coordinates.
///
/// A tie whose endpoints share a system produces one of these; a tie broken
/// across a system -- or page -- break produces two, one filed under each
/// system.
pub struct TieSegment {
    pub shape: Polygon,
}

/// The resolved appearance knobs a tie is drawn with, folded once per arrange
/// pass rather than per tie.
///
/// Follows the two-source pattern `beam_spacing` and `dot_spacing` use: user
/// override, else app default. There is no `ScoreDefaults` source because no
/// `<line-width type="tie">` appears in any of the bundled samples.
#[derive(Copy, Clone, Debug)]
pub struct TieMetrics {
    pub endpoint_thickness: f32,
    pub midpoint_thickness: f32,
    pub height_ratio: f32,
    /// Well-ordered by construction (`height_min <= height_max`) -- see
    /// [`ordered_bounds`], which every `TieMetrics` built via [`from_sources`]
    /// is folded through.
    pub height_min: f32,
    pub height_max: f32,
    pub note_gap: f32,
    pub vertical_offset: f32,
    pub break_inset: f32,
    pub break_fragment: f32,
}

/// Orders a caller-supplied `(min, max)` pair, substituting the app default for a
/// non-finite bound.
///
/// Both values arrive straight from user input -- a slider in the options pane
/// dragged past its partner, or `--tie-height-min NaN` on the CLI -- and
/// `f32::clamp` panics outright on `min > max` or on a NaN bound. Folding that here
/// is what keeps `height()` total: an inverted pair simply reads as the band between
/// the two values, whichever way round they were given.
fn ordered_bounds(min: f32, max: f32, default_min: f32, default_max: f32) -> (f32, f32) {
    let min = if min.is_finite() { min } else { default_min };
    let max = if max.is_finite() { max } else { default_max };
    (min.min(max), min.max(max))
}

impl TieMetrics {
    pub fn resolve(params: LayoutParams<'_>) -> Self {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        Self::from_sources(user_layout, app_defaults)
    }

    fn from_sources(user: &UserLayout, app: &AppDefaults) -> Self {
        let (height_min, height_max) = ordered_bounds(
            user.tie_height_min.unwrap_or(app.tie_height_min),
            user.tie_height_max.unwrap_or(app.tie_height_max),
            app.tie_height_min,
            app.tie_height_max,
        );

        TieMetrics {
            endpoint_thickness: user
                .tie_endpoint_thickness
                .unwrap_or(app.tie_endpoint_thickness),
            midpoint_thickness: user
                .tie_midpoint_thickness
                .unwrap_or(app.tie_midpoint_thickness),
            height_ratio: user.tie_height_ratio.unwrap_or(app.tie_height_ratio),
            height_min,
            height_max,
            note_gap: user.tie_note_gap.unwrap_or(app.tie_note_gap),
            vertical_offset: user.tie_vertical_offset.unwrap_or(app.tie_vertical_offset),
            break_inset: user.tie_break_inset.unwrap_or(app.tie_break_inset),
            break_fragment: user.tie_break_fragment.unwrap_or(app.tie_break_fragment),
        }
    }

    /// This tie's arc height for a span of `dx` tenths, clamped so very short
    /// ties still read as curves and very long ones do not balloon.
    pub fn height(&self, dx: f32) -> f32 {
        debug_assert!(
            self.height_min <= self.height_max,
            "tie height bounds must be ordered"
        );
        (dx.abs() * self.height_ratio).clamp(self.height_min, self.height_max)
    }

    /// The half-width the outline is offset by at a tapered end, scaled for the
    /// note it meets (`scale` is a note's own `scale`, so grace notes and
    /// reduced staves get proportionate ties).
    fn half_width_end(&self, scale: f32) -> f32 {
        self.endpoint_thickness * scale / 2.
    }

    /// The half-width at the arc's widest point.
    fn half_width_mid(&self, scale: f32) -> f32 {
        self.midpoint_thickness * scale / 2.
    }
}

/// Builds the filled lens shape for one tie arc, from `p0` to `p1`.
///
/// The shape is a cubic-Bezier centre curve offset by a half-width that varies
/// along it: widest in the middle, tapering to a point at both ends. Sampling
/// the two offset edges and joining them gives one closed, filled polygon.
///
/// This one builder serves both the unbroken case and each half of a tie split
/// across a system or page break, because a broken tie is engraved as **two
/// complete arcs** -- each tapered at both of its own ends -- rather than as one
/// arc sliced at its apex. See [`split_tie`].
///
/// `scale` is the endpoint note's own scale factor.
///
/// # Why this returns a `Polygon` and not a curve
///
/// There is no curve primitive in this codebase: `DrawableElement` is
/// `Line | Rect | Circle | Text | Polygon`, and every sink is written against
/// exactly those five. Sampling the arc into a filled `Polygon` therefore
/// requires **zero** changes to `drawable/`, to the SVG / PDF / flat-buffer
/// sinks, or to the decoder in `web/music-xml.js`. It is also the trick beams
/// already use -- `Line::extrude` turns a beam segment into a filled quad, and
/// `PartMeasure::beams` is a `Vec<Polygon>`.
///
/// What it costs: `2 * (TIE_SAMPLES + 1)` points per arc in every output stream
/// (34 at the current sample count), and a polyline rather than a true curve at
/// extreme zoom. At print resolution the difference is invisible; in a deeply
/// zoomed browser canvas it is not.
///
/// ## Migrating to a true path, later
///
/// This function is deliberately written to make that a small change: it
/// computes the four Bezier control points *first* and only then samples them,
/// so a path-emitting version would return the control points and delete the
/// sampling loop. The other end of the change is mechanical but touches every
/// sink:
///
/// 1. `lib/src/drawable/elements/path.rs` -- a `Path` element holding a start
///    point plus a `Vec` of cubic segments, with the fill/stroke fields the
///    other elements carry.
/// 2. `lib/src/drawable/drawable_element.rs` -- a `DrawableElement::Path`
///    variant, plus its arms in `accumulate_bounds` (the control polygon is a
///    safe over-estimate of the curve's bounds) and in `impl Scale`.
/// 3. `lib/src/drawable/canvas.rs` -- a `Canvas::draw_path` method and its
///    `draw_one` dispatch arm.
/// 4. `lib/src/drawable/canvas/svg/mod.rs` -- emit `<path d="M .. C .. Z" />`.
/// 5. `lib/src/drawable/canvas/pdf/mod.rs` -- `Content::cubic_to`, already used
///    by `draw_circle`, so this sink is nearly free.
/// 6. `lib/src/drawable/canvas/flat_buffer/mod.rs` -- a `TAG_PATH = 5.0`
///    constant and its record layout, documented on `FlatBuffer` alongside the
///    others.
/// 7. `web/music-xml.js` -- the mirrored tag constant and a
///    `ctx.bezierCurveTo` arm in the draw loop.
/// 8. `tests/` -- extend `one_of_each()` in `test_canvas.rs`, add a record-layout
///    test to `test_flat_buffer.rs`, an operator assertion to `test_pdf.rs`, and
///    a `Scale` case to `test_scale.rs`.
///
/// That is a self-contained feature of its own, and it benefits slurs, hairpins
/// and every future curved element -- which is exactly why it is not bundled in
/// here.
pub fn tie_arc(
    p0: XY,
    p1: XY,
    side: TieSide,
    scale: f32,
    metrics: &TieMetrics,
    color: Color,
) -> Polygon {
    let dx = p1.x - p0.x;
    let bulge = side.sign() * metrics.height(dx);

    // Each handle pulls along the span and out towards the apex, so the arc
    // leaves both ends on a slant.
    let c0 = p0.mv(dx * SHOULDER, bulge);
    let c1 = p1.mv(-dx * SHOULDER, bulge);

    let w_end = metrics.half_width_end(scale);
    let wm = metrics.half_width_mid(scale);
    let (w0, w1) = (w_end, w_end);

    let mut outer: Vec<XY> = Vec::with_capacity(TIE_SAMPLES + 1);
    let mut inner: Vec<XY> = Vec::with_capacity(TIE_SAMPLES + 1);

    for i in 0..=TIE_SAMPLES {
        let t = i as f32 / TIE_SAMPLES as f32;

        let centre = cubic(p0, c0, c1, p1, t);
        let normal = cubic_normal(p0, c0, c1, p1, t);
        let half = half_width_at(w0, wm, w1, t);

        outer.push(centre + normal.scale(half));
        inner.push(centre - normal.scale(half));
    }

    // Outer edge forward, inner edge back: one closed ring.
    inner.reverse();
    outer.extend(inner);

    Polygon {
        pts: outer,
        color,
        stroke_color: None,
        stroke_width: None,
    }
}

/// The arcs one tie contributes, each tagged with the system it belongs to.
///
/// One segment when both endpoints share a system; two when they do not -- an
/// *opening* fragment leaving the start note and running out to the end of its
/// measure, and a short *closing* fragment (the "courtesy tie") arriving at the
/// end note from the left. The asymmetry is intentional: the opening fragment
/// takes the space available to it, the closing one is a fixed stub.
///
/// Each fragment is a **complete tie shape**, tapered to a point at both of its
/// own ends, which is how printed music engraves a broken tie. It is tempting to
/// instead slice one long arc at its apex and let the two blunt halves line up
/// across the break, but that is not what the convention looks like on the page.
/// Because both fragments are ordinary arcs between two tapered endpoints at the
/// same height, [`tie_arc`] needs no notion of a cut end and serves the unbroken
/// and broken cases identically.
///
/// Takes anchors and extents rather than a `&Score` on purpose: it keeps the
/// split rule a pure function, testable without building a multi-system
/// document, and reusable as-is when slurs arrive.
///
/// Returns nothing at all for a tie that cannot be drawn: an endpoint with no
/// anchor (its note was skipped for lacking `default-x`), an end that sorts
/// before its start, or a same-system pair whose end is not to the right of its
/// start.
pub fn split_tie(
    tie: &Tie,
    anchors: &HashMap<NoteId, NoteAnchor>,
    extents: &HashMap<SystemKey, SystemExtent>,
    metrics: &TieMetrics,
) -> Vec<(SystemKey, TieSegment)> {
    let (Some(start), Some(end)) = (anchors.get(&tie.start), anchors.get(&tie.end)) else {
        return Vec::new();
    };

    if end.key < start.key {
        return Vec::new();
    }

    let side = tie
        .side
        .unwrap_or_else(|| TieSide::infer(start.stem, start.staff_line));

    let p0 = tip_right(start, side, metrics);
    let p1 = tip_left(end, side, metrics);

    if start.key == end.key {
        if p1.x <= p0.x {
            return Vec::new();
        }

        let shape = tie_arc(p0, p1, side, start.scale, metrics, start.color);
        return vec![(start.key, TieSegment { shape })];
    }

    let (Some(start_system), Some(end_system)) = (extents.get(&start.key), extents.get(&end.key))
    else {
        return Vec::new();
    };

    let mut segments = Vec::with_capacity(2);

    // Opening fragment: a complete arc leaving the note and taking up the space
    // available to it, which is the rest of its own measure less a margin so it
    // clears the barline. It ends level with the note it left, not raised to the
    // apex, because it is a whole tie shape in its own right.
    //
    // The two fragments are deliberately not the same length: this one spans
    // what is left of the measure, while the closing one below is short and
    // fixed. Floored at `break_fragment` so a note falling right at the end of a
    // measure still gets a visible arc rather than a sliver.
    //
    // The cap at the system's right edge is a *typographic* limit, not a
    // technical one -- there is plenty of real page past the final barline (the
    // right margin, and nothing clips before the page edge), but a tie reaching
    // into the margin reads as a mistake. The trade-off it forces: for a note
    // crowded against the last barline of a system, the cap wins over the floor
    // and the fragment comes out shorter than `break_fragment`. Lifting the cap
    // is the alternative if that ever looks worse than the overhang would.
    let measure_end = start.measure_right.min(start_system.right) - metrics.break_inset;
    let out_x = measure_end
        .max(p0.x + metrics.break_fragment)
        .min(start_system.right);
    if out_x > p0.x {
        let shape = tie_arc(
            p0,
            XY { x: out_x, y: p0.y },
            side,
            start.scale,
            metrics,
            start.color,
        );
        segments.push((start.key, TieSegment { shape }));
    }

    // Closing fragment: likewise a complete arc, arriving at the note from the
    // left. Its length is fixed rather than "back to the system edge", because
    // the next system opens with a clef and key signature it must not run
    // through.
    let in_x = (p1.x - metrics.break_fragment).max(end_system.left);
    if p1.x > in_x {
        let shape = tie_arc(
            XY { x: in_x, y: p1.y },
            p1,
            side,
            end.scale,
            metrics,
            end.color,
        );
        segments.push((end.key, TieSegment { shape }));
    }

    segments
}

/// Where a tie leaving `anchor` to the right begins.
fn tip_right(anchor: &NoteAnchor, side: TieSide, metrics: &TieMetrics) -> XY {
    anchor.left.mv(
        anchor.width + metrics.note_gap,
        side.sign() * metrics.vertical_offset,
    )
}

/// Where a tie arriving at `anchor` from the left ends.
fn tip_left(anchor: &NoteAnchor, side: TieSide, metrics: &TieMetrics) -> XY {
    anchor
        .left
        .mv(-metrics.note_gap, side.sign() * metrics.vertical_offset)
}

/// A point on the cubic Bezier through `p0 -> c0 -> c1 -> p1` at `t`.
fn cubic(p0: XY, c0: XY, c1: XY, p1: XY, t: f32) -> XY {
    let u = 1. - t;
    p0.scale(u * u * u) + c0.scale(3. * u * u * t) + c1.scale(3. * u * t * t) + p1.scale(t * t * t)
}

/// The unit normal of that cubic at `t`, rotated a quarter turn from the
/// tangent. Degenerate tangents (a zero-length span) fall back to straight up,
/// which keeps the outline from collapsing to a line.
fn cubic_normal(p0: XY, c0: XY, c1: XY, p1: XY, t: f32) -> XY {
    let u = 1. - t;
    let tangent =
        (c0 - p0).scale(3. * u * u) + (c1 - c0).scale(6. * u * t) + (p1 - c1).scale(3. * t * t);

    let len = tangent.length();
    if len <= f32::EPSILON {
        return XY { x: 0., y: -1. };
    }

    XY {
        x: -tangent.y / len,
        y: tangent.x / len,
    }
}

/// The arc's half-width at `t`: a scalar quadratic that starts at `a`, ends at
/// `b`, and actually *passes through* `m` at the halfway point.
///
/// A plain quadratic Bezier `(a, m, b)` would not -- a Bezier is pulled towards
/// its middle control point without reaching it, landing at
/// `a/4 + m/2 + b/4`. Since `m` here is
/// [`TieMetrics::midpoint_thickness`] halved, and that is a measured engraving
/// value (Bravura's `tieMidpointThickness`) rather than a hint, the control
/// value is solved for instead: `c = 2m - (a + b) / 2` is the value that puts
/// the curve exactly on `m` at `t = 0.5`.
fn half_width_at(a: f32, m: f32, b: f32, t: f32) -> f32 {
    let c = 2. * m - (a + b) / 2.;
    let u = 1. - t;
    u * u * a + 2. * u * t * c + t * t * b
}
