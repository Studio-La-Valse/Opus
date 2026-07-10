use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;
use crate::visual::staff::Staff;
use crate::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Chord {
    pub notes: Vec<Note>,

    pub stem: Option<Stem>,

    pub staff_distances_from_top: BTreeMap<u32, f32>,
    pub staff_scaling: BTreeMap<u32, f32>,

    pub color: Color,
    pub leger_thickness: f32,
    pub leger_width: f32,
}

impl Chord {
    fn legers(&self) -> Vec<Line> {
        let mut lines = Vec::new();

        for (idx, _dx) in self.staff_distances_from_top.iter() {
            let staff_scale = self.staff_scaling.get(idx).unwrap_or(&1.);
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

                    lines.push(_line);

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

                    lines.push(_line);

                    dy -= each_line;
                    line -= 1;
                }
            }
        }

        lines
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
    fn arrange(&mut self, origin: &XY) {
        for note in self.notes.iter_mut() {
            if let Some(dy) = self.staff_distances_from_top.get(&note.staff) {
                note.arrange(&origin.mv(0., *dy));
            }
        }

        if let Some(stem) = self.stem.as_mut() {
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
                let staff_m_origin =
                    &origin.mv(0., *self.staff_distances_from_top.get(&stem.staff).unwrap());
                staff_m_origin.y - tip.y
            };

            let length = ((origin.y + self.staff_distances_from_top.get(&stem.staff).unwrap())
                - default_y)
                - tail_anchor.y;
            stem.length = length;
        }
    }
}

impl Content for Chord {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();

        for note in self.notes.iter() {
            if self.staff_distances_from_top.contains_key(&note.staff) {
                result.push(note);
            }
        }

        if let Some(stem) = self.stem.as_ref() {
            result.push(stem);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();

        for line in self.legers() {
            result.push(line.into());
        }

        result
    }
}
