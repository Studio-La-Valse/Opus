use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::layout::Layout;
use crate::ray::Ray;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::user_layout::UserLayout;
use crate::visual::chord::Chord;
use crate::visual::element::ScoreElement;
use crate::visual::rest::Rest;
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
    pub beams: Vec<Polygon>,

    pub rests: Vec<Rest>,

    pub stem_thickness: f32,
    pub beam_thickness: f32,
    pub beam_spacing: f32,
}

impl PartMeasure {
    pub fn new(number: u32) -> Self {
        Self {
            number,
            ..Default::default()
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        let chord_groups = collect(&mut self.chords);

        for mut chords in chord_groups {
            strategy.rebeam(&mut chords)
        }
    }

    pub fn arrange_beams(&mut self) {
        self.beams.clear();

        let chord_groups = collect(&mut self.chords);

        for chords in chord_groups {
            let groups = create_beam_groups(chords);

            for mut group in groups {
                let direction = match infer_direction(&group) {
                    Some(direction) => direction,
                    None => continue,
                };

                let ray = match create_ray(
                    &group,
                    &direction,
                    &self.beam_thickness,
                    &self.beam_spacing,
                ) {
                    Some(ray) => ray,
                    None => continue,
                };

                let beams = arrange_beams(
                    &group,
                    &ray,
                    &direction,
                    &self.beam_thickness,
                    &self.beam_spacing,
                    &self.stem_thickness,
                );

                self.beams.extend(beams);

                adjust_stem_lengths(
                    &mut group,
                    &ray,
                    &direction,
                    &self.beam_thickness,
                    &self.beam_spacing,
                );
            }
        }
    }
}

impl ScoreElement for PartMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();
        for chord in self.chords.values_mut().flatten() {
            result.push(chord);
        }
        for rest in self.rests.iter_mut() {
            result.push(rest);
        }
        result
    }

    fn _apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.stem_thickness = user_layout
            .stem_thickness
            .or(layout.appearance.stem_thickness)
            .unwrap_or(app_defaults.stem_thickness);

        self.beam_thickness = user_layout
            .beam_thickness
            .or(layout.appearance.beam_thickness)
            .unwrap_or(app_defaults.beam_thickness);

        self.beam_spacing = user_layout
            .beam_spacing
            .unwrap_or(app_defaults.beam_spacing);
    }
}

impl Layoutable for PartMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        for chord in self.chords.values_mut().flatten() {
            chord.measure(available);
        }

        for rest in self.rests.iter_mut() {
            rest.measure(available);
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

        for rest in self.rests.iter_mut() {
            if let Some(dy) = self.staff_distances_from_top.get(&rest.staff) {
                let dy = dy + rest.staff_line as f32 * 5.;

                let dx: f32 = if rest.is_measure {
                    self.width / 2.
                } else {
                    rest.default_x.unwrap()
                };

                let glyph_origin = self.origin.mv(dx, dy);

                rest.arrange(&glyph_origin);
            }
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
        for rest in self.rests.iter() {
            result.push(rest);
        }
        result
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();
        for beam in &self.beams {
            let line: Element = beam.clone().into();
            result.push(line);
        }
        result
    }
}

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

fn infer_direction(chords: &[&mut Chord]) -> Option<UpDown> {
    let stems: Vec<&Stem> = chords.iter().filter_map(|s| s.stem.as_ref()).collect();

    if stems.is_empty() {
        return None;
    }

    let first_dir = stems[0].direction;
    if stems.len() == 1 {
        return Some(first_dir.invert());
    }

    let mut is_cross = false;
    for stem in &stems {
        if stem.direction != first_dir {
            is_cross = true;
            break;
        }
    }

    if !is_cross {
        Some(first_dir.invert())
    } else {
        Some(first_dir)
    }
}

fn create_ray(
    chords: &[&mut Chord],
    _direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
) -> Option<Ray> {
    if chords.is_empty() {
        return None;
    }

    let first_stem = chords.first().unwrap().stem.as_ref().unwrap();
    let sign = if first_stem.direction != UpDown::Up {
        1.0
    } else {
        -1.0
    };

    let mut left = first_stem.tip().mv(
        0.,
        first_stem.beams.len() as f32 * (beam_spacing + beam_thickness) * sign,
    );

    let len = chords.len();
    if len == 1 {
        return Some(Ray::from_dir(left, XY { x: 1.0, y: 0.0 }));
    }

    let last_stem = chords.last().unwrap().stem.as_ref().unwrap();

    let mut right = last_stem.tip().mv(
        0.,
        first_stem.beams.len() as f32 * (beam_spacing + beam_thickness) * sign,
    );

    let max_dy = 20.;
    let dy = (right.y - left.y).abs();

    if dy > max_dy {
        let overshoot = dy - max_dy;
        let adjust = overshoot / 2.;

        if left.y > right.y {
            left = left.mv(0., -adjust);
            right = right.mv(0., adjust);
        } else {
            left = left.mv(0., adjust);
            right = right.mv(0., -adjust);
        }
    }

    Some(Ray::from_pts(left, right))
}

fn arrange_beams(
    chords: &[&mut Chord],
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
    stem_thickness: &f32,
) -> Vec<Polygon> {
    let mut beams: Vec<Polygon> = vec![];
    if chords.is_empty() {
        return beams;
    }

    let len = chords.len();

    if len == 1 {
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
                    let dx = -stem_thickness / 2.;
                    let left_point = Ray {
                        origin: left_stem.xy.mv(dx, 0.),
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
                            let dx = stem_thickness / 2.;
                            right_point = Some(
                                Ray {
                                    origin: right_stem.xy.mv(dx, 0.),
                                    dir: XY { x: 0., y: 1. },
                                }
                                .intersect(offset_ray)
                                .unwrap(),
                            );
                            break;
                        }
                    }

                    let dy: f32 = match direction {
                        UpDown::Up => -beam_thickness,
                        UpDown::Down => *beam_thickness,
                    };
                    beams.push(
                        Line {
                            start: left_point,
                            end: right_point.unwrap(),
                            stroke_color: Color::BLACK,
                            stroke_width: *beam_thickness,
                        }
                        .extrude(&XY { x: 0., y: dy })
                        .mv(0., dy / -2.),
                    )
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
    chords: &mut [&mut Chord],
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
) {
    for chord in chords {
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
