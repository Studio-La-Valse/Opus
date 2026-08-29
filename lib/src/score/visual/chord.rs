use crate::drawable::layoutable::Layoutable;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::clef::Clef;
use crate::score::visual::note::Note;
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;
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

    pub clef_change: BTreeMap<StaffIdx, Clef>,
}

impl Chord {
    /// here, origin is the origin of the part measure.
    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.xy = *origin;

        self.arrange_notes(staff_ctx);
        self.arrange_stem(staff_ctx);
        self.arrange_clef_changes(staff_ctx);

        // rearrange the accidentals so that they don't overlap.
        self.rearrange_accidentals();
    }

    fn arrange_notes(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for note in self.notes.iter_mut() {
            let ctx = staff_ctx.get(&note.staff).unwrap();
            note.arrange_ctx(&self.xy, ctx);
        }
    }

    fn arrange_stem(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
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
            stem.arrange(&tail_anchor);

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

            stem.arrange_flag();
        }
    }

    fn arrange_clef_changes(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for (staff_idx, clef) in self.clef_change.iter_mut() {
            let ctx = staff_ctx.get(staff_idx).unwrap();

            clef.rescale(ctx.scaling * Clef::COURTESY_SCALE);

            let dx = -5. + self.notes.first().unwrap().default_x - clef.width;
            let dy = ctx.distance_from_top
                + Staff::DEFAULT_SPACE_SIZE / 2. * clef.clef.line as f32 * ctx.scaling;

            let origin = self.xy.mv(dx, dy);

            clef.arrange(&origin);
        }
    }

    fn rearrange_accidentals(&mut self) {
        let mut accidentals: Vec<&mut Accidental> = self
            .notes
            .iter_mut()
            .filter_map(|v| v.accidental.as_mut())
            .collect();

        rearrange_accidentals(&mut accidentals)
    }
}

impl ScoreElement for Chord {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut children: Vec<&mut dyn ScoreElement> = Vec::new();

        for note in self.notes.iter_mut() {
            children.push(note);
        }

        if let Some(stem) = self.stem.as_mut() {
            children.push(stem);
        }

        for clef in self.clef_change.values_mut() {
            children.push(clef);
        }

        children
    }

    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Chord {
    fn measure(&mut self, available: &XY) {
        for note in self.notes.iter_mut() {
            note.measure(available);
        }

        if let Some(stem) = self.stem.as_mut() {
            stem.measure(available);
        }

        for clef in self.clef_change.values_mut() {
            clef.measure(available);
        }
    }

    /// here, origin is the origin of the part measure.
    fn arrange(&mut self, _origin: &XY) {
        todo!("Use arrange_ctx instead")
    }
}

/// Rearranges accidentals in-place from top to bottom, moving each accidental
/// left by the exact minimal amount to nest into cutouts of accidentals above it.
pub fn rearrange_accidentals(accidentals: &mut Vec<&mut Accidental>) {
    if accidentals.is_empty() {
        return;
    }

    // 1. Sort top to bottom (descending Y coordinate)
    accidentals.sort_by(|a, b| {
        a.xy.y
            .partial_cmp(&b.xy.y)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    // 2. Process top to bottom
    for i in 0..accidentals.len() {
        let mut shift_for_i: f32 = 0.0;

        // Find max shift required relative to all accidentals placed above it
        for j in 0..i {
            let shift = accidentals[i].required_left_shift(accidentals[j]);
            if shift > shift_for_i {
                shift_for_i = shift;
            }
        }

        if shift_for_i > 0.0 {
            accidentals[i].xy.x -= shift_for_i + 1.5;
        }
    }
}
