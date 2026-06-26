use std::collections::BTreeMap;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;
use crate::visual::note::Note;

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub staff_distances_from_top: BTreeMap<u32, f32>,

    pub notes: Vec<Note>,
}

impl PartMeasure {
    pub fn new() -> Self {
        Self {
            ..  Default::default()
        }
    }
}

impl ScoreElement for PartMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();
        for note in self.notes.iter_mut() {
            result.push(note);
        }

        result
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for note in self.notes.iter_mut() {
            note.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;

        for note in self.notes.iter_mut() {
            if let Some(dy) = self.staff_distances_from_top.get(&note.staff) {
                note.arrange(&origin.mv(0., *dy));
            }
        }
    }
}

impl Content for PartMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();
        for note in &self.notes {
            result.push(note);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
