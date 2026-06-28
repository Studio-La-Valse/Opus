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

    pub fn requires_rebeam(&mut self) -> bool {
        let chord_groups = collect(&mut self.chords);

        for chords in chord_groups {
            if requires_rebeam(&chords) {
                return true;
            }
        }

        false
    }

    pub fn rebeam(&mut self) {
        let chord_groups = collect(&mut self.chords);

        for mut chords in chord_groups {
            rebeam(&mut chords)
        }
    }

    pub fn arrange_beams(&mut self) {
        self.beams.clear();

        let beam_thickness = 2.4;
        let beam_spacing = 1.1;

        // Fix 1: Pass mutably to allow mutable extraction
        let chord_groups = collect(&mut self.chords);

        for chords in chord_groups {
            let groups = create_beam_groups(chords);

            for mut group in groups {
                let ray = match create_ray(&group) {
                    Some(ray) => ray,
                    None => continue,
                };

                let direction = infer_direction(&group);
                let beams = arrange_beams(&group, &ray, &direction, &beam_thickness, &beam_spacing);

                // Rust allows split-borrowing here: mutating self.beams while self.chords is borrowed
                self.beams.extend(beams);

                // Fix 2: Pass a mutable slice to allow mutation of inner chords
                adjust_stem_lengths(&mut group, &ray, &direction, &beam_thickness, &beam_spacing);
            }
        }
    }
}

impl ScoreElement for PartMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();
        // Fix 4: Use values_mut().flatten() to bypass closure lifetime quirks
        for chord in self.chords.values_mut().flatten() {
            result.push(chord);
        }
        result
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for chord in self.chords.values_mut().flatten() {
            chord.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.origin = *origin;

        for chord in self.chords.values_mut().flatten() {
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
        for chord in self.chords.values().flatten() {
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

// Fix 1: Accept &mut BTreeMap to safely yield out &mut Chords
fn collect(chord_groups: &mut BTreeMap<u32, Vec<Chord>>) -> Vec<Vec<&mut Chord>> {
    let mut result: Vec<Vec<&mut Chord>> = Vec::new();

    for (_idx, chords) in chord_groups.iter_mut() {
        let mut group: Vec<&mut Chord> = Vec::new();
        for chord in chords {
            group.push(chord);
        }
        result.push(group);
    }

    result
}

fn requires_rebeam(chords: &[&mut Chord]) -> bool {
    for chord in chords {
        if let Some(stem) = &chord.stem {
            let expected_beams = stem.duration.beam_count();
            let beams = stem.beams.len() as i8;

            if expected_beams != beams {
                return true;
            }
        }
    }
    false
}

fn rebeam(chords: &mut [&mut Chord]) {
    // Step 1: Collect indices of chords that actually have stems.
    // This safely separates our structural inspection from our mutations.
    let stem_indices: Vec<usize> = chords
        .iter()
        .enumerate()
        .filter(|(_, chord)| chord.stem.is_some())
        .map(|(idx, _)| idx)
        .collect();

    let len = stem_indices.len();

    // If there are fewer than 2 stems, a beam group cannot be formed.
    // We clear any rogue beams and turn them back into standalone notes.
    if len < 2 {
        for &idx in &stem_indices {
            if let Some(stem) = chords[idx].stem.as_mut() {
                stem.beams.clear();
            }
        }
        return;
    }

    // Step 2: Apply the beam types based on position in the group
    for (pos, &idx) in stem_indices.iter().enumerate() {
        // We can safely extract a mutable reference to the stem index-by-index
        let stem = chords[idx].stem.as_mut().unwrap();
        stem.beams.clear();

        let expected_beams = stem.duration.beam_count() as u32;

        // Assign the appropriate BeamType for every beam level required by this note
        let beam_type = if pos == 0 {
            BeamType::Start
        } else if pos == len - 1 {
            BeamType::End
        } else {
            BeamType::Continue
        };

        for beam_idx in 1..=expected_beams {
            stem.beams.insert(beam_idx, beam_type);
        }
    }
}

// Local enum to break the "match-and-borrow" lifecycle completely
enum BeamGroupAction {
    Start,
    Continue,
    End,
    Standalone,
    Panic(&'static str),
}

fn create_beam_groups(chords: Vec<&mut Chord>) -> Vec<Vec<&mut Chord>> {
    let mut result = vec![];
    let mut group: Vec<&mut Chord> = vec![];

    for chord in chords {
        // Fix 3: Read what we need into an independent state enum, letting the borrow on chord die instantly
        let action = match &chord.stem {
            Some(stem) => match stem.beams.get(&1) {
                Some(BeamType::Start) => BeamGroupAction::Start,
                Some(BeamType::Continue) => BeamGroupAction::Continue,
                Some(BeamType::End) => BeamGroupAction::End,
                Some(BeamType::HookStart) | Some(BeamType::HookEnd) => {
                    BeamGroupAction::Panic("First beam cannot be a hook")
                }
                None => BeamGroupAction::Standalone,
            },
            None => BeamGroupAction::Standalone,
        };

        // Now we can safely push and consume `chord` without the compiler worrying about active borrows
        match action {
            BeamGroupAction::Start => {
                if !group.is_empty() {
                    panic!("Cannot start a beam group when one is already open");
                }
                group.push(chord);
            }
            BeamGroupAction::Continue => {
                if group.is_empty() {
                    panic!("Cannot continue a beam group when none is open");
                }
                group.push(chord);
            }
            BeamGroupAction::End => {
                if group.is_empty() {
                    panic!("Cannot end a beam group when none is open");
                }
                group.push(chord);
                result.push(group);
                group = vec![];
            }
            BeamGroupAction::Standalone => {
                group.push(chord);
                result.push(group);
                group = vec![];
            }
            BeamGroupAction::Panic(msg) => {
                panic!("{}", msg);
            }
        }
    }

    if !group.is_empty() {
        result.push(group);
    }
    result
}

fn create_ray(chords: &[&mut Chord]) -> Option<Ray> {
    let len = chords.len();
    if len < 2 {
        return None;
    }

    let first_stem = chords.first().unwrap().stem.as_ref().unwrap();
    let last_stem = chords.last().unwrap().stem.as_ref().unwrap();

    Some(Ray::from_pt(first_stem.tip(), last_stem.tip()))
}

fn infer_direction(chords: &[&mut Chord]) -> UpDown {
    let stems: Vec<&Stem> = chords.iter().map(|s| s.stem.as_ref().unwrap()).collect();

    if stems.is_empty() {
        panic!("Chords contain no stems, cannot infer beam group direction.");
    }

    let first_dir = stems[0].direction;
    if stems.len() == 1 {
        return first_dir.invert();
    }

    for stem in &stems {
        if stem.direction != first_dir {
            return first_dir;
        }
    }

    first_dir.invert()
}

fn arrange_beams(
    chords: &[&mut Chord],
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

                        if let BeamType::End = right_beam {
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

// Fix 2: Changed signature to take a mutable slice `&mut [&mut Chord]`
fn adjust_stem_lengths(
    chords: &mut [&mut Chord],
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
) {
    for chord in chords {
        // Now allowed, because iterating over `&mut [&mut Chord]` gives us `&mut &mut Chord`
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
