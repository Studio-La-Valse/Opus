use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::note::Note;
use crate::score::visual::placed::Placed;
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
}

impl Chord {
    /// here, origin is the origin of the part measure.
    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.xy = *origin;

        self.arrange_notes(staff_ctx);
        self.arrange_stem(staff_ctx);
        self.arrange_dots(staff_ctx);

        // rearrange the accidentals so that they don't overlap.
        self.rearrange_accidentals();
    }

    fn arrange_notes(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for note in self.notes.iter_mut() {
            let ctx = staff_ctx.get(&note.staff).unwrap();
            note.arrange_ctx(&self.xy, ctx);
        }
    }

    /// Positions this chord's stem and computes its natural length from the
    /// notes' already-arranged positions and `default_y` (or the engine's own
    /// default when the document gives none).
    ///
    /// Called once from [`arrange_ctx`](Self::arrange_ctx) during the ordinary
    /// arrange, and a second time by
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// before it fits a beam ray: resetting the stem to this natural length
    /// undoes whatever an earlier beam pass adjusted it to, which is what
    /// keeps beaming idempotent. Pure in the notes and the staff context, so
    /// calling it again always reproduces the same length.
    pub fn arrange_stem(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
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

    /// Places the augmentation dots of every note in the chord. Dots of all
    /// notes align to one x column (just right of the widest notehead), and a
    /// dot that would land on a staff line is nudged half a space in the
    /// chord's stem direction - up if the stem points up, down if it points
    /// down, up by default when the chord has no stem.
    fn arrange_dots(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        let dir_sign = match &self.stem {
            Some(stem) => match stem.direction {
                UpDown::Up => -1.,
                UpDown::Down => 1.,
            },
            None => -1.,
        };

        let column_x = self
            .notes
            .iter()
            .map(|note| note.xy.x + note.width)
            .fold(f32::MIN, f32::max);

        for note in self.notes.iter_mut() {
            if note.dots.is_empty() {
                continue;
            }

            let ctx = staff_ctx.get(&note.staff).unwrap();
            let on_staff_line = note.staff_line.rem_euclid(2) == 0;
            let dy = if on_staff_line {
                dir_sign * (Staff::DEFAULT_SPACE_SIZE / 2.) * ctx.scaling
            } else {
                0.
            };

            let base = XY {
                x: column_x,
                y: note.xy.y + dy,
            };
            for (i, dot) in note.dots.iter_mut().enumerate() {
                let center = base.mv((i as f32 + 1.) * note.dot_spacing, 0.);
                dot.arrange(&center);
            }
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

impl Chord {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        for note in self.notes.iter_mut() {
            note.resolve_layout(params);
        }

        if let Some(stem) = self.stem.as_mut() {
            stem.resolve_layout(params);
        }
    }
}

impl Chord {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        for note in self.notes.iter_mut() {
            note.measure(available, params);
        }

        if let Some(stem) = self.stem.as_mut() {
            stem.measure(available, params);
        }
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
