//! Ties: what the content walk hangs on a note, and the geometry that turns a
//! pair of endpoints into a drawable arc.
//!
//! Nothing here touches the score tree --
//! [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties) does
//! that. Working from
//! [`TieAnchor`]s rather than from notes in place is what lets the broken case
//! be tested without a multi-system fixture, and is the seam a future slur
//! implementation reuses: a slur is the same arc between different anchors.

use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::user_layout::UserLayout;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::stem::UpDown;

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

/// A tie leaving a note for the next note of its pitch, hung on the note it
/// leaves -- see [`Note::tie`](crate::score::visual::note::Note).
///
/// All the content walk records is *that* the note is tied and how the document
/// wants the arc to look; which note it reaches, and where the arc runs, is
/// settled later by
/// [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties), once the
/// part holding the note has been arranged and the notes in it have positions.
pub struct Tie {
    /// Which way the arc bulges, from `<tied orientation=…>` / `placement=…`.
    /// `None` means the document did not say, and the side is inferred from the
    /// note the tie leaves -- see [`TieSide::infer`].
    pub side: Option<TieSide>,

    /// The drawn arc, in absolute (global tenths) coordinates, assigned on every
    /// arrange pass. `None` before the first one, and for a tie with no room to
    /// draw in -- a span that does not run left to right.
    pub shape: Option<Polygon>,
}

impl Tie {
    pub fn new(side: Option<TieSide>) -> Self {
        Tie { side, shape: None }
    }

    /// The arc from the note this tie leaves to the note it arrives at.
    ///
    /// `stem` is the *start* chord's stem direction, the other half of what
    /// [`TieSide::infer`] needs when the document named no side.
    pub fn arrange(
        &mut self,
        start: &TieAnchor,
        end: &TieAnchor,
        stem: Option<UpDown>,
        metrics: &TieMetrics,
    ) {
        let side = self.resolve_side(start, stem);
        let p0 = start.tip_right(side, metrics);
        let p1 = end.tip_left(side, metrics);

        self.draw(p0, p1, side, start, metrics);
    }

    /// The opening half of a tie with no note left to reach: a complete arc
    /// leaving the note and running out to `limit`, the end of the part less a
    /// margin that clears the final barline.
    ///
    /// It ends level with the note it left, not raised to an apex, because it is
    /// a whole tie shape in its own right -- tapered at both of its own ends --
    /// which is how printed music engraves a tie running into a break. What it
    /// does *not* do is draw the other half: there is no courtesy arc in front
    /// of the note on the next system. See the module docs of
    /// [`tie_arranger`](crate::score::visual::tie_arranger).
    pub fn arrange_open(
        &mut self,
        start: &TieAnchor,
        stem: Option<UpDown>,
        limit: f32,
        metrics: &TieMetrics,
    ) {
        let side = self.resolve_side(start, stem);
        let p0 = start.tip_right(side, metrics);

        self.draw(p0, XY { x: limit, y: p0.y }, side, start, metrics);
    }

    /// Which way this tie bulges: what the document named, else what the note it
    /// leaves implies.
    fn resolve_side(&self, start: &TieAnchor, stem: Option<UpDown>) -> TieSide {
        self.side
            .unwrap_or_else(|| TieSide::infer(stem, start.staff_line))
    }

    /// Assigns the arc between two resolved endpoints, or nothing at all when
    /// there is no room between them. `start` is the note the tie leaves: its
    /// scale sets the arc's thickness and its colour fills it, so a grace note
    /// gets a proportionate tie.
    fn draw(&mut self, p0: XY, p1: XY, side: TieSide, start: &TieAnchor, metrics: &TieMetrics) {
        self.shape =
            (p1.x > p0.x).then(|| tie_arc(p0, p1, side, start.scale, metrics, start.color));
    }
}

/// One note's finished geometry, as much of it as a tie needs.
///
/// Copied out of the tree so that a tie can be arranged while the note it hangs
/// off is borrowed mutably, and so that the search for a tie's other end can
/// answer with a value rather than a reference into the measure it sits in.
#[derive(Copy, Clone, Debug)]
pub struct TieAnchor {
    /// Left edge of the notehead at its vertical centre -- exactly what
    /// `Note::xy` is, per `Note::arrange_ctx` and `PartMeasure::ledger_lines`.
    pub left: XY,
    pub width: f32,
    /// The note's own scale factor, so grace notes get proportionate ties.
    pub scale: f32,
    pub color: Color,
    /// Staff and line, which is how a tie's other end is recognised once the
    /// pitches are gone: two tied notes are the same pitch, so they sit on the
    /// same line of the same staff.
    pub staff: StaffIdx,
    pub staff_line: i32,
}

impl TieAnchor {
    /// Where a tie leaving this note to the right begins.
    fn tip_right(&self, side: TieSide, metrics: &TieMetrics) -> XY {
        self.left.mv(
            self.width + metrics.note_gap,
            side.sign() * metrics.vertical_offset,
        )
    }

    /// Where a tie arriving at this note from the left ends.
    fn tip_left(&self, side: TieSide, metrics: &TieMetrics) -> XY {
        self.left
            .mv(-metrics.note_gap, side.sign() * metrics.vertical_offset)
    }
}

/// The resolved appearance knobs a tie is drawn with, folded once per arrange
/// pass rather than per tie.
///
/// Follows the two-source pattern `beam_spacing` and `dot_spacing` use: user
/// override, else app default. There is no `ScoreDefaults` source because no
/// `<line-width type="tie">` appears in any of the bundled samples.
#[derive(Copy, Clone, Debug, Default)]
pub struct TieMetrics {
    pub endpoint_thickness: f32,
    pub midpoint_thickness: f32,
    pub height_ratio: f32,
    pub height_min: f32,
    pub height_max: f32,
    pub note_gap: f32,
    pub vertical_offset: f32,
    pub break_inset: f32,
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
        TieMetrics {
            endpoint_thickness: user
                .tie_endpoint_thickness
                .unwrap_or(app.tie_endpoint_thickness),
            midpoint_thickness: user
                .tie_midpoint_thickness
                .unwrap_or(app.tie_midpoint_thickness),
            height_ratio: user.tie_height_ratio.unwrap_or(app.tie_height_ratio),
            height_min: user.tie_height_min.unwrap_or(app.tie_height_min),
            height_max: user.tie_height_max.unwrap_or(app.tie_height_max),
            note_gap: user.tie_note_gap.unwrap_or(app.tie_note_gap),
            vertical_offset: user.tie_vertical_offset.unwrap_or(app.tie_vertical_offset),
            break_inset: user.tie_break_inset.unwrap_or(app.tie_break_inset),
        }
    }

    /// This tie's arc height for a span of `dx` tenths, clamped so very short
    /// ties still read as curves and very long ones do not balloon.
    pub fn height(&self, dx: f32) -> f32 {
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
