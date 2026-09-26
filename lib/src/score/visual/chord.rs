use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::Note;
use crate::score::visual::placed::Placed;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Default stem length (in tenths) used when the source has no explicit stem
/// `default-y`; negative points up, positive points down.
const DEFAULT_STEM_LENGTH: f32 = 30.;

#[derive(Default)]
pub struct Chord {
    pub xy: XY,

    pub grace: bool,

    pub notes: Vec<Note>,
    pub stem: Option<Stem>,

    pub color: Color,
}

impl Chord {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.color = params.foreground_color();

        for note in self.notes.iter_mut() {
            note.resolve_layout(params);
        }

        if let Some(stem) = self.stem.as_mut() {
            stem.resolve_layout(params);
        }
    }
}

// ---- placement ----

impl Chord {
    /// Positions this chord's stem and computes its natural length from the
    /// notes' already-arranged positions and `default_y` (or the engine's own
    /// default when the document gives none).
    ///
    /// Called once from
    /// [`arrange_chord_ctx`](crate::score::visual::arranger::ContentArranger::arrange_chord_ctx)
    /// during the ordinary arrange, and a second time by
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// before it fits a beam ray: resetting the stem to this natural length
    /// undoes whatever an earlier beam pass adjusted it to, which is what
    /// keeps beaming idempotent. Pure in the notes and the staff context, so
    /// calling it again always reproduces the same length.
    pub fn place_stem(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        if let Some(stem) = self.stem.as_mut() {
            let staff_top = staff_ctx.get(&stem.staff).unwrap().distance_from_top;

            let key = |n: &&Note| OrderedFloat(n.xy.y);

            let lowest_note = self.notes.iter().max_by_key(key);
            let highest_note = self.notes.iter().min_by_key(key);

            let tail_note = match stem.direction {
                UpDown::Up => lowest_note,
                UpDown::Down => highest_note,
            }
            .unwrap();

            let tail_anchor = match stem.direction {
                UpDown::Up => tail_note.glyph.stem_anchor_right,
                UpDown::Down => tail_note.glyph.stem_anchor_left,
            }
            .unwrap();
            let tail_anchor = tail_note.scale_pt(&tail_anchor);
            stem.place(&tail_anchor);

            let default_y: f32 = if let Some(def_y) = &stem.default_y {
                *def_y
            } else {
                let default_length = match stem.direction {
                    UpDown::Up => -DEFAULT_STEM_LENGTH,
                    UpDown::Down => DEFAULT_STEM_LENGTH,
                };

                let tip_note = match stem.direction {
                    UpDown::Up => highest_note,
                    UpDown::Down => lowest_note,
                }
                .unwrap();

                let tip_anchor = (match stem.direction {
                    UpDown::Up => tip_note.glyph.stem_anchor_right,
                    UpDown::Down => tip_note.glyph.stem_anchor_left,
                })
                .unwrap();
                let tip_anchor = tip_note.scale_pt(&tip_anchor);

                let tip = &tip_anchor.mv(0., default_length);
                let staff_m_origin = &self.xy.mv(0., staff_top);
                staff_m_origin.y - tip.y
            };

            let length = ((self.xy.y + staff_top) - default_y) - tail_anchor.y;
            stem.length = length;

            stem.place_flag();
        }
    }
}
