use crate::app_defaults::AppDefaults;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;
use crate::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Chord {
    pub notes: Vec<Note>,

    pub stem: Option<Stem>,

    pub staff_distances_from_top: BTreeMap<u32, f32>,
}

impl Chord {}

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
            let dy = self.staff_distances_from_top.get(&note.staff).unwrap();
            note.arrange(&origin.mv(0., *dy));
        }

        if let Some(stem) = self.stem.as_mut() {
            let key = |n: &&Note| OrderedFloat(n.xy.y);

            let lowest_note = self.notes.iter().max_by_key(key);
            let highest_note = self.notes.iter().min_by_key(key);

            let tale_note = match stem.direction {
                UpDown::Up => lowest_note,
                UpDown::Down => highest_note,
            }
            .unwrap();

            let tale_anchor = match stem.direction {
                UpDown::Up => tale_note.glyph.as_ref().unwrap().stem_anchor_right,
                UpDown::Down => tale_note.glyph.as_ref().unwrap().stem_anchor_left,
            };
            let tale_anchor = tale_note.scale_pt(&tale_anchor);
            stem.arrange(&tale_anchor);

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

                let tip_anchor = match stem.direction {
                    UpDown::Up => tip_note.glyph.as_ref().unwrap().stem_anchor_right,
                    UpDown::Down => tip_note.glyph.as_ref().unwrap().stem_anchor_left,
                };
                let tip_anchor = tip_note.scale_pt(&tip_anchor);

                let tip = &tip_anchor.mv(0., default_length);
                let staff_m_origin =
                    &origin.mv(0., *self.staff_distances_from_top.get(&stem.staff).unwrap());
                staff_m_origin.y - tip.y
            };

            let length = ((origin.y + self.staff_distances_from_top.get(&stem.staff).unwrap())
                - default_y)
                - tale_anchor.y;
            stem.length = length;
        }
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
        vec![]
    }
}
