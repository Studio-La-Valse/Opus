use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::chord::Chord;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub staff_distances_from_top: BTreeMap<u32, f32>,

    pub chords: Vec<Chord>,
}

impl PartMeasure {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}

impl ScoreElement for PartMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();
        for note in self.chords.iter_mut() {
            result.push(note);
        }

        result
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for note in self.chords.iter_mut() {
            note.measure(available);
        }
    }

    /// Here, origin is the origin of the part measure
    fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;

        for chord in self.chords.iter_mut() {
            chord.staff_distances_from_top.clear();

            for (idx, dy) in self.staff_distances_from_top.iter() {
                chord.staff_distances_from_top.insert(*idx, *dy);
            }

            chord.arrange(origin);
        }
    }
}

impl Content for PartMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();
        for note in &self.chords {
            result.push(note);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
