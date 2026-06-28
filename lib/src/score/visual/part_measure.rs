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

        let beam_thickness = 2.4;
        let beam_spacing = 1.1;

        let mut groups = create_beam_groups(&mut self.chords);

        for group in groups.iter_mut() {
            let ray = match create_ray(group) {
                Some(ray) => ray,
                None => continue,
            };

            let direction = infer_direction(group);
            let beams = arrange_beams(group, &ray, &direction, &beam_thickness, &beam_spacing);
            self.beams.extend(beams);

            adjust_stem_lengths(group, &ray, &direction, &beam_thickness, &beam_spacing);
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

fn create_ray(chords: &mut Vec<&mut Chord>) -> Option<Ray> {
    let len = chords.len();

    if len < 2 {
        return None;
    }

    let first_stem = chords.first().unwrap().stem.as_ref().unwrap();
    let last_stem = chords.last().unwrap().stem.as_ref().unwrap();

    let ray = Ray::from_pt(first_stem.tip(), last_stem.tip());
    Some(ray)
}

fn infer_direction(chords: &mut Vec<&mut Chord>) -> UpDown {
    let stems: Vec<&Stem> = chords.iter().map(|s| s.stem.as_ref().unwrap()).collect();

    if stems.is_empty() {
        panic!("Chords contain no stems, cannot infer beam group direction.");
    }

    let first_dir = stems[0].direction;

    if stems.len() == 1 {
        return first_dir.invert();
    }

    let mut is_cross = false;

    for stem in &stems {
        if stem.direction != first_dir {
            is_cross = true;
            break;
        }
    }

    if !is_cross {
        first_dir.invert()
    } else {
        first_dir
    }
}

pub fn arrange_beams(
    chords: &mut Vec<&mut Chord>,
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
) -> Vec<Line> {
    let mut beams: Vec<Line> = vec![];
    let len = chords.len();

    if len <= 1 {
        return beams;
    }

    for i in 0..len {
        let left_chord = &chords[i];

        let left_stem = match &left_chord.stem {
            Some(stem) => stem,
            None => continue,
        };

        for (beam_idx, left_beam) in left_stem.beams.iter() {
            match left_beam {
                BeamType::Start => {
                    let offset = create_offset(direction, beam_idx, beam_thickness, beam_spacing);

                    let offset_ray = ray.mv(0., offset);
                    let left_point = Ray {
                        origin: left_stem.xy,
                        dir: XY { x: 0., y: 1. },
                    }
                    .intersect(offset_ray)
                    .unwrap();

                    let mut right_point: Option<XY> = None;

                    for right_chord in chords.iter().take(len).skip(i + 1) {
                        let right_stem = match &right_chord.stem {
                            Some(stem) => stem,
                            None => continue,
                        };

                        let right_beam = match right_stem.beams.get(beam_idx) {
                            Some(beam) => beam,
                            None => continue,
                        };

                        match right_beam {
                            BeamType::End => {
                                right_point = Some(
                                    Ray {
                                        origin: right_stem.xy,
                                        dir: XY { x: 0., y: 1. },
                                    }
                                    .intersect(offset_ray)
                                    .unwrap(),
                                );
                                break;
                            }
                            _ => continue,
                        }
                    }

                    beams.push(Line {
                        start: left_point,
                        end: right_point.unwrap(),
                        stroke_color: Color::BLACK,
                        stroke_width: *beam_thickness,
                    })
                }
                _ => continue,
            }
        }
    }

    beams
}

fn create_offset(
    direction: &UpDown,
    beam_idx: &u32,
    beam_thickness: &f32,
    beam_spacing: &f32,
) -> f32 {
    match direction {
        UpDown::Up => ((beam_idx - 1) as f32) * -(beam_thickness + beam_spacing),
        UpDown::Down => ((beam_idx - 1) as f32) * (beam_thickness + beam_spacing),
    }
}

fn adjust_stem_lengths(
    chords: &mut Vec<&mut Chord>,
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
) {
    for chord in chords.iter_mut() {
        let stem = match chord.stem.as_mut() {
            Some(stem) => stem,
            None => continue,
        };

        let beam_idx: &u32 = if &stem.direction != direction {
            match stem.beams.keys().next() {
                Some(beam) => beam,
                None => continue,
            }
        } else {
            match stem.beams.keys().last() {
                Some(beam) => beam,
                None => continue,
            }
        };

        let offset = create_offset(direction, beam_idx, beam_thickness, beam_spacing);

        let offset_ray = ray.mv(0., offset);

        stem.attach_ray(&offset_ray);
    }
}
