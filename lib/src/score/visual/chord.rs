use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;

#[derive(Default)]
pub struct Chord {
    pub notes: Vec<Note>,
}

impl Chord {}

impl ScoreElement for Chord {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        self.notes
            .iter_mut()
            .map(|n| n as &mut dyn ScoreElement)
            .collect()
    }
}

impl Layoutable for Chord {
    fn measure(&mut self, available: &XY) {
        for note in self.notes.iter_mut() {
            note.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        for note in self.notes.iter_mut() {
            note.arrange(origin);
        }
    }
}

impl Content for Chord {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();

        for note in self.notes.iter() {
            result.push(note);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
