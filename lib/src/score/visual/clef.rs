use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::note::NoteId;
use crate::score::visual::placed::Placed;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::clef::Clef as SmuflClef;

/// A clef change written part-way through a measure: which clef, on which staff,
/// and the note or rest it is drawn in front of.
///
/// Recorded flat on [`Score::clef_changes`](crate::score::visual::score::Score)
/// rather than hung on the anchor, for the reason
/// [`Tie`](crate::score::visual::tie::Tie) is: the change belongs to a *staff*,
/// but the only thing that can say where it goes is a note or rest in a
/// `PartMeasure`, and those are different branches of the tree. The drawn clefs
/// are rebuilt from this list by
/// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger).
///
/// Holds the SMuFL glyph rather than the core clef because picking the glyph
/// needs the staff's line count, which is settled during the walk -- see
/// [`Clef::anchor_line`](crate::score::core::clef::Clef).
pub struct ClefChange {
    /// The note or rest this change is drawn in front of.
    pub anchor: NoteId,
    /// The staff the change applies to, which is not necessarily the staff the
    /// anchor sits on: a part's staves share one stream of `<note>` elements.
    pub staff: StaffIdx,
    pub clef: SmuflClef,
}

/// What a clef is pinned to horizontally.
///
/// The two ways a clef meets the music around it, and they pull in opposite
/// directions. A clef that *opens* a measure is aligned into a column shared
/// across the system, so its left edge is the fixed point and it grows
/// rightwards into the key signature's column. A courtesy clef -- the one
/// announcing the next measure's clef at a barline, or a change written in front
/// of a note -- hangs off whatever it announces, so its right edge is the fixed
/// point and it grows leftwards into the space before it.
///
/// Naming the two cases is what lets one placement rule serve all three clefs a
/// staff measure can draw; the vertical rule and the gap never varied between
/// them in the first place.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ClefAnchor {
    /// Left edge at this x.
    LeftEdgeAt(f32),
    /// Right edge [`Clef::COURTESY_GAP`] to the left of this x.
    GapBefore(f32),
}

pub struct Clef {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub scale: f32,

    pub clef: SmuflClef,
}

impl Placed for Clef {
    fn xy(&self) -> XY {
        self.xy
    }

    fn scale(&self) -> f32 {
        self.scale
    }
}

impl Clef {
    /// Scale factor applied to courtesy / mid-measure clef changes relative to
    /// the staff scale.
    pub const COURTESY_SCALE: f32 = 0.8;

    /// Gap in tenths between a courtesy clef and whatever it announces: the
    /// barline at the end of its measure, or the note or rest a mid-measure
    /// change is written in front of.
    pub const COURTESY_GAP: f32 = 5.;

    /// Places this clef against `anchor`, sitting on whichever line its own clef
    /// names measured down from `staff_top`.
    ///
    /// The one rule for every clef the engine draws. Only the anchor differs
    /// between them: an opening clef takes the column
    /// [`System::arrange_measure_starts`](crate::score::visual::system::System)
    /// hands it, a clef at a barline the right edge of its own measure, a
    /// mid-measure change the left edge of the note or rest it precedes.
    ///
    /// Reads `width`, so the clef has to be sized before it is placed --
    /// [`rescale`](Clef::rescale) if it was built after the measure pass, as the
    /// ones
    /// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger) builds
    /// are.
    /// A clef straight out of [`new`](Clef::new) has no width at all.
    ///
    /// `staff_scaling` is the *staff's* factor and not the clef's own, which for
    /// a courtesy clef is already reduced by [`Clef::COURTESY_SCALE`]: the line
    /// it sits on is a fact about the staff, not about how large the glyph is
    /// drawn.
    ///
    /// Reports nothing back. A caller that has to say how far right the ink
    /// reaches -- only `arrange_clef_start` does, for the next column -- works it
    /// out from the offset it passed in, exactly as the key and time signature
    /// beside it do.
    pub fn place(&mut self, anchor: ClefAnchor, staff_top: f32, staff_scaling: f32) {
        let x = match anchor {
            ClefAnchor::LeftEdgeAt(x) => x,
            ClefAnchor::GapBefore(x) => x - Clef::COURTESY_GAP - self.width,
        };

        let dy = self.clef.line as f32 * (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_scaling;

        self.arrange(&XY {
            x,
            y: staff_top + dy,
        });
    }

    pub fn new(clef: crate::smufl::glyphs::clef::Clef) -> Clef {
        Clef {
            xy: Default::default(),
            color: Color::BLACK,
            scale: 1.,

            width: 0.,
            height: 0.,

            clef,
        }
    }

    /// Bounding box scaled to world pixels.
    pub fn scaled_box(&self) -> BoundingBox {
        self.scale_box(&self.clef.bbox)
    }

    fn measure_size(&mut self) {
        let bbox = self.scale_box(&self.clef.bbox);
        self.width = bbox.width();
        self.height = bbox.height();
    }

    /// Sets the scale and re-derives width/height from it, so callers that
    /// rescale a clef after construction (e.g. courtesy clefs at 0.8x) don't
    /// end up positioning against a stale, pre-rescale size.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
        self.measure_size();
    }
}

impl Layoutable for Clef {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }

    fn measure(&mut self, _available: &XY, _params: LayoutParams<'_>) {
        self.measure_size();
    }

    /// Supplied origin x coordinate is left of clef, y coordinate is the line in the staff.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
