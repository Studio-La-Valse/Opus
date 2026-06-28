use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::ray::Ray;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::chord::Chord;
use crate::visual::element::ScoreElement;
use crate::visual::stem::{BeamType, Stem, UpDown};
use std::collections::BTreeMap;

#[derive(Default)]
pub struct PartMeasure {
    pub specified_width: Option<f32>,
    pub final_width: f32,
    pub number: u32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub staff_distances_from_top: BTreeMap<u32, f32>,

    /// Chords for each voice
    pub chords: BTreeMap<u32, Vec<Chord>>,
    pub beams: Vec<Line>,
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

        let mut groups = create_beam_groups(&mut self.chords);

        for group in groups.iter_mut() {
            let beams = arrange_beams(group);
            self.beams.extend(beams);
        }
    }
}

impl ScoreElement for PartMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();
        for chord in self.chords.iter_mut().flat_map(|v| v.1) {
            result.push(chord);
        }

        result
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for chord in self.chords.iter_mut().flat_map(|v| v.1) {
            chord.measure(available);
        }
    }

    /// Here, origin is the origin of the part measure
    fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;

        for chord in self.chords.iter_mut().flat_map(|v| v.1) {
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
        for chord in self.chords.iter().flat_map(|v| v.1) {
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

pub fn create_beam_groups(chords: &mut BTreeMap<u32, Vec<Chord>>) -> Vec<Vec<&mut Chord>> {
    let mut result = vec![];

    for chords in chords.values_mut() {
        let mut group: Vec<&mut Chord> = vec![];

        for chord in chords.iter_mut() {
            if let Some(stem) = &chord.stem {
                if let Some(first_beam) = stem.beams.get(&1) {
                    match first_beam {
                        BeamType::Start => {
                            if !group.is_empty() {
                                panic!("Cannot start a beam group when one is already open");
                            }
                            group.push(chord);
                        }
                        BeamType::Continue => {
                            if group.is_empty() {
                                panic!("Cannot continue a beam group when none is open");
                            }
                            group.push(chord);
                        }
                        BeamType::End => {
                            if group.is_empty() {
                                panic!("Cannot end a beam group when none is open");
                            }
                            group.push(chord);
                            result.push(group);
                            group = vec![];
                        }
                        BeamType::HookStart | BeamType::HookEnd => {
                            panic!("First beam cannot be a hook");
                        }
                    }
                } else {
                    // no beams → standalone
                    group.push(chord);
                    result.push(group);
                    group = vec![];
                }
            } else {
                // no stem → standalone
                group.push(chord);
                result.push(group);
                group = vec![];
            }
        }

        // flush leftover group
        if !group.is_empty() {
            result.push(group);
        }
    }

    result
}

pub fn arrange_beams(chords: &mut Vec<&mut Chord>) -> Vec<Line> {
    let mut beams: Vec<Line> = vec![];

    if chords.len() <= 1 {
        return beams;
    }

    let first_stem = chords.first().unwrap().stem.as_ref().unwrap();
    let last_stem = chords.last().unwrap().stem.as_ref().unwrap();

    let ray = Ray::from_pt(first_stem.tip(), last_stem.tip());

    let mut lines: BTreeMap<u32, Line> = BTreeMap::new();
    let beam_thickness = 1.5;
    let beam_spacing = 0.8;

    // Attach ray to stems
    for chord in chords.iter_mut() {
        let stem = chord.stem.as_mut().unwrap();
        stem.attach_ray(&ray);
    }

    let mut offset_ray = Ray { ..ray };

    // Build beams
    for chord in chords.iter_mut() {
        let stem = chord.stem.as_mut().unwrap();

        for (idx, beam_type) in stem.beams.iter() {
            match beam_type {
                BeamType::Start => {
                    let offset = match stem.direction {
                        UpDown::Up => ((idx - 1) as f32) * (beam_thickness + beam_spacing),
                        UpDown::Down => ((idx - 1) as f32) * -(beam_thickness + beam_spacing),
                    };

                    offset_ray = ray.mv(0., offset);
                    let tip = Ray {
                        origin: stem.xy,
                        dir: XY { x: 0., y: 1. },
                    }
                    .intersect(offset_ray)
                    .unwrap();

                    lines.insert(
                        *idx,
                        Line {
                            start: tip,
                            end: tip,
                            stroke_width: beam_thickness,
                            stroke_color: Color::BLACK,
                        },
                    );
                }
                BeamType::End => {
                    let tip = Ray {
                        origin: stem.xy,
                        dir: XY { x: 0., y: 1. },
                    }
                    .intersect(offset_ray)
                    .unwrap();
                    let mut line = lines.remove(idx).unwrap();
                    line.end = tip;
                    beams.push(line);
                }
                _ => {}
            }
        }
    }

    beams
}

pub fn between_two_stems(_left: &Stem, _right: &Stem) {
    todo!()
}
