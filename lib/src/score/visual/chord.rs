use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::Layout;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;
use crate::visual::staff::Staff;
use crate::visual::staff_meta::StaffCtx;
use crate::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Chord {
    pub xy: XY,
    pub notes: Vec<Note>,

    pub stem: Option<Stem>,
    pub legers: Vec<Line>,

    pub color: Color,
    pub leger_thickness: f32,
    pub leger_width: f32,
}

impl Chord {

    /// here, origin is the origin of the part measure.
    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.xy = *origin;

        for note in self.notes.iter_mut() {
            let ctx = staff_ctx.get(&note.staff).unwrap();
            note.arrange_ctx(origin, ctx);
        }

        self.arrange_stem(staff_ctx);
        self.arrange_legers(staff_ctx);
    }

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

    fn arrange_legers(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.legers.clear();

        for (idx, staff_ctx) in staff_ctx.iter() {
            let staff_scale = staff_ctx.scaling;
            let each_line = (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_scale;

            let key = |n: &&Note| OrderedFloat(n.xy.y);

            let highest_note = self
                .notes
                .iter()
                .filter(|n| n.staff == *idx)
                .min_by_key(key);

            if let Some(highest_note) = highest_note
                && highest_note.staff_line < 0
            {
                let middle = highest_note.xy.mv(highest_note.width / 2., 0.);
                let bottom = middle.mv(0., 0.);

                let left = bottom.mv(self.leger_width / -2., 0.);
                let right = bottom.mv(self.leger_width / 2., 0.);

                let mut dy = 0.;

                for line in highest_note.staff_line..-1 {
                    if line % 2 != 0 {
                        dy += each_line;
                        continue;
                    }

                    let _line: Line = Line {
                        start: left.mv(0., dy),
                        end: right.mv(0., dy),
                        stroke_width: self.leger_thickness,
                        stroke_color: self.color,
                    };

                    self.legers.push(_line);

                    dy += each_line;
                }
            }

            let lowest_note = self
                .notes
                .iter()
                .filter(|n| n.staff == *idx)
                .max_by_key(key);
            if let Some(lowest_note) = lowest_note
                && lowest_note.staff_line > 9
            {
                let middle = lowest_note.xy.mv(lowest_note.width / 2., 0.);
                let top = middle.mv(0., 0.);

                let left = top.mv(self.leger_width / -2., 0.);
                let right = top.mv(self.leger_width / 2., 0.);

                let mut dy = 0.;
                let mut line = lowest_note.staff_line;

                while line >= 10 {
                    if line % 2 != 0 {
                        dy -= each_line;
                        line -= 1;
                        continue;
                    }

                    let _line: Line = Line {
                        start: left.mv(0., dy),
                        end: right.mv(0., dy),
                        stroke_width: self.leger_thickness,
                        stroke_color: self.color,
                    };

                    self.legers.push(_line);

                    dy -= each_line;
                    line -= 1;
                }
            }
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

        children
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
        self.color = _user_layout
            .foreground_color
            .unwrap_or(_app_defaults.foreground_color);

        self.leger_thickness = _user_layout
            .staff
            .unwrap_or(_app_defaults.staff_line_thickness);

        self.leger_width = 1.875 * Staff::DEFAULT_SPACE_SIZE;
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
    }

    /// here, origin is the origin of the part measure.
    fn arrange(&mut self, _origin: &XY) {
        todo!("Use arrange_ctx instead")
    }
}

impl Content for Chord {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();

        for note in self.notes.iter() {
            result.push(note);
        }

        if let Some(stem) = self.stem.as_ref() {
            result.push(stem);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();

        for line in self.legers.iter() {
            result.push((*line).into());
        }

        result
    }
}
