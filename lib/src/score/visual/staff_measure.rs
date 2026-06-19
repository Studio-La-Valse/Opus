use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub notes: Vec<Note>,
}

impl StaffMeasure {}

impl ScoreElement for StaffMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        self.notes
            .iter_mut()
            .map(|n| n as &mut dyn ScoreElement)
            .collect()
    }
}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for note in self.notes.iter_mut() {
            note.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        for note in self.notes.iter_mut() {
            note.arrange(origin);
        }
    }
}

impl Content for StaffMeasure {
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
