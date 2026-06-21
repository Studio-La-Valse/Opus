use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;
use crate::visual::stem::{Stem, UpDown};
use ordered_float::OrderedFloat;

#[derive(Default)]
pub struct Chord {
    pub notes: Vec<Note>,

    pub stem: Option<Stem>,
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

    // here, origin is the origin of the staff measure.
    fn arrange(&mut self, origin: &XY) {
        for note in self.notes.iter_mut() {
            note.arrange(origin);
        }

        if let Some(stem) = self.stem.as_mut() {
            let key = |n: &&Note| OrderedFloat(n.xy.y);

            let note = match stem.direction {
                UpDown::Up => self.notes.iter().max_by_key(key),
                UpDown::Down => self.notes.iter().min_by_key(key),
            }
            .unwrap();

            let anchor = match stem.direction {
                UpDown::Up => note.glyph.as_ref().unwrap().stem_anchor_right,
                UpDown::Down => note.glyph.as_ref().unwrap().stem_anchor_left,
            };
            let anchor = note.scale_pt(&anchor);
            stem.arrange(&anchor);

            let length = (origin.y - stem.default_y) - anchor.y;
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
