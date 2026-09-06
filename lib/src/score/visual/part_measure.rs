use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::ray::Ray;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::voice::Voice;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::Note;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::stem::{BeamType, Stem, UpDown};
use ordered_float::OrderedFloat;
use std::collections::BTreeMap;

/// Staff-line index of the top staff line; notes with a lower index sit above the
/// staff and need ledger lines.
const LEDGER_ABOVE_STAFF_LINE: i32 = 0;

/// Staff-line index of the bottom staff line; notes with a higher index sit below
/// the staff and need ledger lines.
const LEDGER_BELOW_STAFF_LINE: i32 = 9;

/// Maximum vertical span a beam is allowed to slant before it is clamped.
const MAX_BEAM_SLANT_DY: f32 = 20.;

/// The length of a hook beam. TODO: infer from available space between two stems and clam to a max length.
const HOOK_LENGTH: f32 = 7.5;

/// Which side of the staff a note (and therefore its ledger lines) sits on.
enum LedgerSide {
    Above,
    Below,
}

#[derive(Default)]
pub struct PartMeasure {
    pub part_id: String,
    pub number: u32,

    pub specified_width: Option<f32>,
    pub final_width: f32,

    pub width: f32,
    pub height: f32,
    pub origin: XY,

    pub chords: BTreeMap<Voice, Vec<Chord>>,
    pub beams: Vec<Polygon>,
    pub ledgers: Vec<Line>,

    pub color: Color,

    pub beam_thickness: f32,
    pub beam_spacing: f32,
    pub note_size_grace: f32,

    pub ledger_thickness: f32,
    pub ledger_width: f32,
}

impl PartMeasure {
    pub fn new(part_id: String, number: u32) -> Self {
        Self {
            part_id,
            number,
            ..Default::default()
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for grace in [false, true] {
            let chord_groups = collect_voices(&mut self.chords, grace);

            for mut chords in chord_groups {
                strategy.rebeam(&mut chords)
            }
        }
    }

    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.origin = *origin;

        self.arrange_chords(staff_ctx);
        self.arrange_ledgers(staff_ctx);

        self.beams.clear();
        self.arrange_beams(true);
        self.arrange_beams(false);
    }

    fn arrange_chords(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        for chord in self.chords.values_mut().flatten() {
            chord.arrange_ctx(&self.origin, staff_ctx);
        }
    }

    fn arrange_ledgers(&mut self, staff_ctx: &BTreeMap<StaffIdx, StaffCtx>) {
        self.ledgers.clear();

        for chord in self.chords.values().flatten() {
            for (idx, staff_ctx) in staff_ctx.iter() {
                let each_line = (Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling;
                let key = |n: &&Note| OrderedFloat(n.xy.y);

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .min_by_key(key)
                    && note.staff_line < LEDGER_ABOVE_STAFF_LINE
                {
                    let lines = self.ledger_lines(note, LedgerSide::Above, each_line);
                    self.ledgers.extend(lines);
                }

                if let Some(note) = chord
                    .notes
                    .iter()
                    .filter(|n| n.staff == *idx)
                    .max_by_key(key)
                    && note.staff_line > LEDGER_BELOW_STAFF_LINE
                {
                    let lines = self.ledger_lines(note, LedgerSide::Below, each_line);
                    self.ledgers.extend(lines);
                }
            }
        }
    }

    /// The ledger lines for a single note that sits `side` of its staff: one
    /// short horizontal line on every even staff-line index between the note and
    /// the staff edge, stepping `each_line` back towards the staff each line.
    fn ledger_lines(&self, note: &Note, side: LedgerSide, each_line: f32) -> Vec<Line> {
        let ledger_width = note.width + 5.;
        let anchor = note.xy.mv(note.width / 2., 0.);
        let left = anchor.mv(ledger_width / -2., 0.);
        let right = anchor.mv(ledger_width / 2., 0.);

        // Staff-line indices from the note inward to the staff edge, plus the
        // per-line dy step (towards the staff, so away from the note).
        let (lines, step): (Vec<i32>, f32) = match side {
            LedgerSide::Above => (
                (note.staff_line..=LEDGER_ABOVE_STAFF_LINE - 1).collect(),
                each_line,
            ),
            LedgerSide::Below => (
                (LEDGER_BELOW_STAFF_LINE + 1..=note.staff_line)
                    .rev()
                    .collect(),
                -each_line,
            ),
        };

        let mut out = Vec::new();
        let mut dy = 0.;
        for line in lines {
            if line % 2 == 0 {
                out.push(Line {
                    start: left.mv(0., dy),
                    end: right.mv(0., dy),
                    stroke_width: self.ledger_thickness,
                    stroke_color: self.color,
                });
            }
            dy += step;
        }
        out
    }

    fn arrange_beams(&mut self, grace: bool) {
        let chord_groups = collect_voices(&mut self.chords, grace);

        let mut beam_thickness = self.beam_thickness;
        let mut beam_spacing = self.beam_spacing;

        if grace {
            beam_thickness *= self.note_size_grace;
            beam_spacing *= self.note_size_grace;
        }

        for chords in chord_groups {
            let groups = create_beam_groups(chords);

            for mut group in groups {
                let direction = match infer_direction(&group) {
                    Some(direction) => direction,
                    None => continue,
                };

                let ray = match create_ray(&group, &direction, &beam_thickness, &beam_spacing) {
                    Some(ray) => ray,
                    None => continue,
                };

                let beams = create_beams(
                    &group,
                    &ray,
                    &direction,
                    &beam_thickness,
                    &beam_spacing,
                    &self.color,
                );

                self.beams.extend(beams);

                adjust_stem_lengths(&mut group, &ray, &direction, &beam_thickness, &beam_spacing);
            }
        }
    }
}

impl PartMeasure {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
        } = params;

        self.beam_thickness = user_layout
            .beam_thickness
            .or(score_defaults.appearance.beam_thickness)
            .unwrap_or(app_defaults.beam_thickness);

        self.beam_spacing = user_layout
            .beam_spacing
            .unwrap_or(app_defaults.beam_spacing);

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.ledger_thickness = user_layout
            .staff
            .unwrap_or(app_defaults.staff_line_thickness);

        self.note_size_grace = user_layout
            .note_size_grace
            .or(score_defaults.appearance.note_size_grace)
            .unwrap_or(app_defaults.note_size_grace);
    }
}

impl PartMeasure {
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        self.height = available.y;

        for chord in self.chords.values_mut().flatten() {
            chord.measure(available, params);
        }
    }
}

fn collect_voices(
    chord_groups: &mut BTreeMap<Voice, Vec<Chord>>,
    grace: bool,
) -> Vec<Vec<&mut Chord>> {
    let mut result: Vec<Vec<&mut Chord>> = Vec::new();

    for chords in chord_groups.values_mut() {
        let mut group: Vec<&mut Chord> = Vec::new();

        for chord in chords {
            if chord.grace != grace {
                continue;
            }

            group.push(chord);
        }

        result.push(group);
    }

    result
}

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
                // This can actually happen when a grace group is in between two
                // notes of a 'regular' beam group.
                // so....
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

    let dy = (right.y - left.y).abs();

    if dy > MAX_BEAM_SLANT_DY {
        let overshoot = dy - MAX_BEAM_SLANT_DY;
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

fn create_beams(
    chords: &[&mut Chord],
    ray: &Ray,
    direction: &UpDown,
    beam_thickness: &f32,
    beam_spacing: &f32,
    color: &Color,
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
            let offset = create_offset(direction, beam_idx, beam_thickness, beam_spacing);
            let offset_ray = ray.mv(0., offset);
            let dx = (-left_stem.thickness * left_stem.scale) / 2.;
            let left_point = Ray {
                origin: left_stem.xy.mv(dx, 0.),
                dir: XY { x: 0., y: 1. },
            }
            .intersect(offset_ray)
            .unwrap();

            let dy: f32 = match direction {
                UpDown::Up => -beam_thickness,
                UpDown::Down => *beam_thickness,
            };

            match left_beam {
                BeamType::Start => {
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
                            let dx = right_stem.thickness * right_stem.scale / 2.;
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

                    beams.push(
                        Line {
                            start: left_point,
                            end: right_point.unwrap(),
                            stroke_color: *color,
                            stroke_width: *beam_thickness,
                        }
                        .extrude(XY { x: 0., y: dy })
                        .mv(0., dy / -2.),
                    )
                }
                BeamType::HookStart => {
                    let right_point = left_point.mv(HOOK_LENGTH, 0.);
                    let vert_ray = Ray::from_dir(right_point, XY { x: 0., y: -1. });
                    let right_pt = offset_ray.intersect(vert_ray).unwrap();
                    let poly = Line {
                        start: left_point,
                        end: right_pt,
                        stroke_color: *color,
                        stroke_width: *beam_thickness,
                    }
                    .extrude(XY { x: 0., y: dy })
                    .mv(0., dy / -2.);

                    beams.push(poly);
                }
                BeamType::HookEnd => {
                    let right_point = left_point.mv(-HOOK_LENGTH, 0.);
                    let vert_ray = Ray::from_dir(right_point, XY { x: 0., y: -1. });
                    let right_pt = vert_ray.intersect(offset_ray).unwrap();
                    let poly = Line {
                        start: left_point,
                        end: right_pt,
                        stroke_color: *color,
                        stroke_width: *beam_thickness,
                    }
                    .extrude(XY { x: 0., y: dy })
                    .mv(0., dy / -2.);

                    beams.push(poly);
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
