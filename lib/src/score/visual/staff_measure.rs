use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::chord::Chord;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ChordIndex {
    pub position: u32,
    pub voice: u32,
}

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub chords: BTreeMap<ChordIndex, Chord>,
}

impl StaffMeasure {}

impl ScoreElement for StaffMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        self.chords
            .iter_mut()
            .map(|n| n.1 as &mut dyn ScoreElement)
            .collect()
    }
}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for chord in self.chords.values_mut() {
            chord.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        for chord in self.chords.values_mut() {
            chord.arrange(origin);
        }
    }
}

impl Content for StaffMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();

        for chord in self.chords.values() {
            result.push(chord);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
