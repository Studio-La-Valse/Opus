use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::NoteId;
use crate::score::visual::placed::Placed;
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

    pub fn measure_size(&mut self) {
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

impl Clef {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}
