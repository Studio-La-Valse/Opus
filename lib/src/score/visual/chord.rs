use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::score::core::staff_idx::StaffIdx;
use crate::user_layout::UserLayout;
use crate::visual::clef::Clef;
use crate::visual::note::Note;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::visual::staff_ctx::StaffCtx;
use crate::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

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

            let tail_anchor = (match stem.direction {
                UpDown::Up => tail_note.glyph.stem_anchor_right,
                UpDown::Down => tail_note.glyph.stem_anchor_left,
            })
            .unwrap();
            let tail_anchor = tail_note.scale_pt(&tail_anchor);
            stem.arrange(&tail_anchor);

            let default_y: f32 = if let Some(def_y) = &stem.default_y {
                *def_y
            } else {
                let default_length = match stem.direction {
                    UpDown::Up => -30.,
                    UpDown::Down => 30.,
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
        }
    }

    fn arrange_clef_changes(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for (staff_idx, clef) in self.clef_change.iter_mut() {
            let ctx = staff_ctx.get(staff_idx).unwrap();

            let dx = -5. + self.notes.first().unwrap().default_x - clef.width;
            let dy = ctx.distance_from_top
                + Staff::DEFAULT_SPACE_SIZE / 2. * clef.clef.line as f32 * ctx.scaling;

            let origin = self.xy.mv(dx, dy);

            clef.arrange(&origin);

            clef.scale = ctx.scaling * 0.8;
        }
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
        _layout: &Layout,
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

impl Drawable for Chord {
    fn content(&self) -> Vec<&dyn Drawable> {
        let mut result: Vec<&dyn Drawable> = Vec::new();

        for note in self.notes.iter() {
            result.push(note);
        }

        if let Some(stem) = self.stem.as_ref() {
            result.push(stem);
        }

        for clef in self.clef_change.values() {
            result.push(clef);
        }

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let result: Vec<DrawableElement> = Vec::new();

        result
    }
}
