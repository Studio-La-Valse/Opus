use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::chord::Chord;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;
use crate::color::Color;
use crate::drawable::elements::line::Line;
use crate::visual::stem::{BeamType, UpDown};

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f32>,
    pub final_width: f32,
    pub number: u32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub staff_distances_from_top: BTreeMap<u32, f32>,

    pub chords: Vec<Chord>,
    pub beams: Vec<Line>
}

impl PartMeasure {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            ..Default::default()
        }
    }

    pub fn arrange_beams(&mut self) {
        self.beams.clear();

        let mut lines: BTreeMap<u32, Line> = BTreeMap::new();
        let beam_thickness = 1.5;
        let beam_spacing = 0.8;

        for chord in self.chords.iter_mut() {
            if let Some(stem) = &chord.stem {
                for (idx, beam) in stem.beams.iter() {
                    match beam {
                        BeamType::Start => {
                            let mut tip = stem.tip();

                            match stem.direction {
                                UpDown::Up => tip = tip.mv(0., ((idx - 1) as f32) * (beam_thickness + beam_spacing)),
                                UpDown::Down => tip = tip.mv(0., ((idx - 1) as f32) * -(beam_thickness + beam_spacing))
                            }

                            lines.insert(*idx, Line { start: tip, end: tip, stroke_width: beam_thickness, stroke_color: Color::BLACK });
                        },
                        BeamType::End => {
                            let mut tip = stem.tip();

                            match stem.direction {
                                UpDown::Up => tip = tip.mv(0., ((idx - 1) as f32) * (beam_thickness + beam_spacing)),
                                UpDown::Down => tip = tip.mv(0., ((idx - 1) as f32) * -(beam_thickness + beam_spacing))
                            }

                            let mut line = lines.remove(idx).unwrap();
                            line.end = tip;

                            self.beams.push(line);
                        },
                        _ => continue
                    }
                }
            }
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

        self.arrange_beams();
    }
}

impl Content for PartMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result: Vec<&dyn Content> = Vec::new();
        for chord in &self.chords {
            result.push(chord);
        }

        result
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();

        for beam in &self.beams {
            let line: Line = *beam;
            result.push(line.into());
        }

        result
    }
}
