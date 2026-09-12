//! Resolves one part's beam groups into drawn segments.
//!
//! Beams used to be a [`PartMeasure`]'s own business, and that works right up to
//! the barline and no further: a group cannot outlive the sequence it was built
//! from, and that sequence was one measure long. The fix is to build the sequence
//! from the whole [`Part`] instead -- a beam group never spans two parts, so a
//! part is the smallest thing that can hold a whole one.
//!
//! # Why that is enough for cross-system beams too
//!
//! A `Part` is one system's worth of one instrument, so a group crossing a system
//! break is split across two of them and neither sees the whole thing. There is no
//! need for either to: **a beam group cannot skip a system**, so the two halves
//! are looking at the same break from opposite sides, and each side can recognise
//! its own half on its own.
//!
//! - A group whose `begin` is not in this part opened on the previous system. It
//!   shows up as a `continue` or `end` arriving with nothing open, at the very
//!   start of the run.
//! - A group still open when the run ends closes on the next system.
//!
//! Each of those gets its levels extended by [`HOOK_LENGTH`] out towards the
//! break, which is the stub printed music draws there. So no group needs an
//! identity beyond the part it was collected from -- and the cross-**page** case
//! is not a case at all, since a page break is a system break.
//!
//! # On idempotency
//!
//! This pass is not idempotent on its own: [`Stem::attach_ray`] writes an absolute
//! stem length and [`create_ray`] reads [`Stem::tip`], which depends on that
//! length. It does not have to be. [`Part::arrange_clear_of`] arranges its
//! measures -- and so recomputes every stem's `length` from its `default_y` --
//! immediately before calling this, so repeated `arrange_score` calls on a cached
//! score converge, which is what the wasm render path depends on.

use std::collections::BTreeMap;

use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::ray::Ray;
use crate::geometry::xy::XY;
use crate::score::core::note_kind::NoteKind;
use crate::score::core::voice::Voice;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::stem::{BeamType, Stem, UpDown};

/// Maximum vertical span a beam is allowed to slant before it is clamped.
const MAX_BEAM_SLANT_DY: f32 = 20.;

/// The length of a hook beam, and of the stub a group runs out to a system break.
/// TODO: infer from available space between two stems and clam to a max length.
const HOOK_LENGTH: f32 = 7.5;

/// Whether a beam group reaches beyond the part it was collected from, and so on
/// which side its levels run a stub out to the edge.
///
/// Since a part is one system's worth of one instrument, "beyond this part" means
/// "on another system" -- see the module docs for why that is all the cross-system
/// case needs to know.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Cut {
    /// The group opened before this part, so its levels reach back past the first
    /// stem.
    pub left: bool,
    /// The group is still open where this part ends, so its levels reach on past
    /// the last stem.
    pub right: bool,
}

/// A run of consecutive chords beamed together, and whether the group they belong
/// to carries on past this part.
pub struct BeamGroup<'a> {
    pub chords: Vec<&'a mut Chord>,
    pub cut: Cut,
}

/// Everything the drawn beams look like, resolved from [`LayoutParams`] by
/// [`Part::resolve_layout`] and read back on every arrange. The mirror of
/// [`TieMetrics`](crate::score::visual::tie::TieMetrics), except that it is stored
/// rather than resolved per pass, because `arrange` has no params to resolve it
/// from.
#[derive(Copy, Clone, Default)]
pub struct BeamMetrics {
    pub thickness: f32,
    pub spacing: f32,
    /// The fraction of full size a grace note is drawn at, so a grace group's
    /// beams are reduced by exactly what its noteheads were.
    pub grace_scale: f32,
    pub color: Color,
}

impl BeamMetrics {
    pub fn resolve(params: LayoutParams<'_>) -> Self {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        Self {
            thickness: user_layout
                .beam_thickness
                .or(score_defaults.appearance.beam_thickness)
                .unwrap_or(app_defaults.beam_thickness),
            spacing: user_layout
                .beam_spacing
                .unwrap_or(app_defaults.beam_spacing),
            grace_scale: params.note_size(NoteKind::Grace),
            color: user_layout
                .foreground_color
                .unwrap_or(app_defaults.foreground_color),
        }
    }
}

/// Rebuilds `part.beams` from its chords.
///
/// Assigns rather than appends, so calling it repeatedly leaves the same segments
/// behind. Must run once the part's measures have been arranged, since every span
/// here is measured between two stems that have to be in their final place.
pub fn arrange_beams(part: &mut Part) {
    let metrics = part.beam_metrics;
    let mut beams: Vec<Polygon> = Vec::new();

    for grace in [false, true] {
        // A grace group's beams are reduced by the same factor its noteheads were;
        // a normal one is drawn at full size.
        let scale = if grace { metrics.grace_scale } else { 1. };
        let thickness = metrics.thickness * scale;
        let spacing = metrics.spacing * scale;

        for run in collect_runs(&mut part.measures, grace) {
            for mut group in create_beam_groups(run) {
                let Some(direction) = infer_direction(&group.chords) else {
                    continue;
                };

                let Some(ray) = create_ray(&group.chords, &direction, &thickness, &spacing) else {
                    continue;
                };

                beams.extend(create_beams(
                    &group.chords,
                    &ray,
                    &direction,
                    &thickness,
                    &spacing,
                    &metrics.color,
                    group.cut,
                ));

                adjust_stem_lengths(&mut group.chords, &ray, &direction, &thickness, &spacing);
            }
        }
    }

    part.beams = beams;
}

/// One part's beamable chords, gathered into the runs that are beamed as a unit:
/// one voice's chords in measure order, with grace notes kept apart from the rest.
/// Exactly the grouping `BeamGroupVisitor` validates against.
///
/// The cross-measure twin of `PartMeasure::collect_voices`, which the rebeam pass
/// still uses as it is: rebeaming is a decision about one measure's worth of
/// durations, and beaming is not.
fn collect_runs(measures: &mut BTreeMap<u32, PartMeasure>, grace: bool) -> Vec<Vec<&mut Chord>> {
    let mut runs: BTreeMap<Voice, Vec<&mut Chord>> = BTreeMap::new();

    // Measures are keyed by number, so walking them in key order walks them in
    // document order -- which is what lets the grouping below treat "consecutive"
    // as a fact about the sequence rather than something it has to look up.
    for measure in measures.values_mut() {
        for (voice, chords) in measure.chords.iter_mut() {
            for chord in chords.iter_mut() {
                if chord.grace != grace {
                    continue;
                }

                runs.entry(*voice).or_default().push(chord);
            }
        }
    }

    runs.into_values().collect()
}

enum BeamGroupAction {
    Start,
    Continue,
    End,
    Standalone,
    Panic(&'static str),
}

/// Chunks one run into the groups that are beamed together: a group runs from the
/// chord declaring [`BeamType::Start`] at level 1 to the one declaring
/// [`BeamType::End`], and a chord carrying no beam at all stands alone.
///
/// Also decides each group's [`Cut`], which is where the cross-system case is
/// settled. The first group of a run may open without a `begin` of its own, and the
/// last may never close; both mean the group continues on another system. Anywhere
/// else in the run the same two shapes mean only that the document contradicts
/// itself -- `BeamGroupVisitor` reports each as a `Warning` -- so they recover into
/// a group but earn no stub.
pub fn create_beam_groups(chords: Vec<&mut Chord>) -> Vec<BeamGroup<'_>> {
    let mut result: Vec<BeamGroup<'_>> = vec![];
    let mut group: Vec<&mut Chord> = vec![];

    // Whether the group being built opened without a `begin`. Only meaningful for
    // the first group of the run, which is the only one whose `begin` could be on
    // the previous system.
    let mut unopened = false;

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
            // A `begin` while a group is open closes that one: it says a group
            // starts here, so whatever came before it ended.
            BeamGroupAction::Start => {
                if !group.is_empty() {
                    result.push(BeamGroup {
                        chords: std::mem::take(&mut group),
                        cut: Cut {
                            left: unopened,
                            right: false,
                        },
                    });
                }
                unopened = false;
                group.push(chord);
            }
            // A `continue` or `end` with nothing open opens the group it claims to
            // carry on, which is the least the document can be taken to mean.
            BeamGroupAction::Continue => {
                if group.is_empty() {
                    unopened = result.is_empty();
                }
                group.push(chord);
            }
            BeamGroupAction::End => {
                if group.is_empty() {
                    unopened = result.is_empty();
                }
                group.push(chord);
                result.push(BeamGroup {
                    chords: std::mem::take(&mut group),
                    cut: Cut {
                        left: unopened,
                        right: false,
                    },
                });
                unopened = false;
            }
            BeamGroupAction::Standalone => {
                group.push(chord);
                result.push(BeamGroup {
                    chords: std::mem::take(&mut group),
                    cut: Cut {
                        left: unopened,
                        right: false,
                    },
                });
                unopened = false;
            }
            BeamGroupAction::Panic(msg) => {
                panic!("{}", msg);
            }
        }
    }

    // Still open where the part ends, so it closes on the next system.
    if !group.is_empty() {
        result.push(BeamGroup {
            chords: group,
            cut: Cut {
                left: unopened,
                right: true,
            },
        });
    }

    result
}

/// Which way the beam stack grows for a whole group: away from the stems when they
/// all point the same way, and towards the odd one out when they do not (a
/// cross-staff group, where the stems on one staff point up and the other's down).
pub fn infer_direction(chords: &[&mut Chord]) -> Option<UpDown> {
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
    // Between the outermost chords that *have* a stem, not the outermost chords: a
    // chord without one carries no beam and so cannot bound the span. It used to be
    // safe to assume the group's own ends had stems, because a group was flushed at
    // the barline; a group carrying on into the next measure can now be closed by a
    // stemless chord arriving there.
    let mut stems = chords.iter().filter_map(|chord| chord.stem.as_ref());
    let first_stem = stems.next()?;
    let last_stem = stems.next_back();

    let sign = if first_stem.direction != UpDown::Up {
        1.0
    } else {
        -1.0
    };

    let mut left = first_stem.tip().mv(
        0.,
        first_stem.beams.len() as f32 * (beam_spacing + beam_thickness) * sign,
    );

    // One stem is no span at all, so the ray is level: there is nothing to slant
    // between.
    let Some(last_stem) = last_stem else {
        return Some(Ray::from_dir(left, XY { x: 1.0, y: 0.0 }));
    };

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
    cut: Cut,
) -> Vec<Polygon> {
    let mut beams: Vec<Polygon> = vec![];
    if chords.is_empty() {
        return beams;
    }

    let len = chords.len();

    // One stem on its own has no span to beam across -- unless the group carries on
    // past this part, in which case a chord stranded alone on the far side of a
    // break still owes its stubs.
    if len == 1 && !cut.left && !cut.right {
        return beams;
    }

    for i in 0..len {
        let left_chord = &chords[i];
        let left_stem = match &left_chord.stem {
            Some(stem) => stem,
            None => continue,
        };

        for (beam_idx, left_beam) in left_stem.beams.iter() {
            // A level the group's *first* stem merely carries -- `continue` or
            // `end`, never `begin` -- was opened on the previous system, so this
            // stem stands in for the `begin` that is not here and the beam reaches
            // a stub back past it to the break. `end` says the level closes here,
            // so its span is this one stem.
            let arrives =
                cut.left && i == 0 && matches!(left_beam, BeamType::Continue | BeamType::End);
            let closes_on_arrival = arrives && matches!(left_beam, BeamType::End);

            let offset = create_offset(direction, beam_idx, beam_thickness, beam_spacing);
            let offset_ray = ray.mv(0., offset);
            let dx = (-left_stem.thickness * left_stem.scale) / 2.;
            let mut left_point = Ray {
                origin: left_stem.xy.mv(dx, 0.),
                dir: XY { x: 0., y: 1. },
            }
            .intersect(offset_ray)
            .unwrap();

            if arrives {
                left_point = point_along(left_point, -HOOK_LENGTH, &offset_ray);
            }

            let dy: f32 = match direction {
                UpDown::Up => -beam_thickness,
                UpDown::Down => *beam_thickness,
            };

            let effective = if arrives { BeamType::Start } else { *left_beam };

            let right_point = match effective {
                BeamType::Start => {
                    let end = if closes_on_arrival {
                        LevelEnd::ClosedAt(left_stem)
                    } else {
                        beam_level_ends_at(chords, i, beam_idx)
                    };

                    // The level runs out to the next system when it has no `end` in
                    // this group *and* the group's last stem is still carrying it.
                    // A level that stops inside the group -- a pair of sixteenths
                    // early in a longer group -- gets no stub however the group was
                    // cut.
                    let departs = cut.right
                        && matches!(end, LevelEnd::OpenAt(_))
                        && chords
                            .last()
                            .and_then(|chord| chord.stem.as_ref())
                            .and_then(|stem| stem.beams.get(beam_idx))
                            .is_some_and(|beam| {
                                matches!(beam, BeamType::Start | BeamType::Continue)
                            });

                    let right_stem = match end {
                        LevelEnd::ClosedAt(stem) => Some(stem),
                        LevelEnd::OpenAt(Some(stem)) => Some(stem),
                        // Nothing else in the group carries the level. When the
                        // group runs past this part the level spans this stem alone
                        // and the stubs do the rest; when it does not, a level
                        // spanning one stem is a hook, which is what it would have
                        // been written as.
                        LevelEnd::OpenAt(None) => (arrives || departs).then_some(left_stem),
                    };

                    let point = match right_stem {
                        Some(right_stem) => {
                            let dx = right_stem.thickness * right_stem.scale / 2.;
                            Ray {
                                origin: right_stem.xy.mv(dx, 0.),
                                dir: XY { x: 0., y: 1. },
                            }
                            .intersect(offset_ray)
                            .unwrap()
                        }
                        None => point_along(left_point, HOOK_LENGTH, &offset_ray),
                    };

                    if departs {
                        point_along(point, HOOK_LENGTH, &offset_ray)
                    } else {
                        point
                    }
                }
                BeamType::HookStart => point_along(left_point, HOOK_LENGTH, &offset_ray),
                BeamType::HookEnd => point_along(left_point, -HOOK_LENGTH, &offset_ray),
                _ => continue,
            };

            beams.push(beam_quad(
                left_point,
                right_point,
                beam_thickness,
                dy,
                color,
            ));
        }
    }

    beams
}

/// Where a beam level opened at `start` stops within the group.
///
/// Two outcomes that used to be one `Option`, and the caller has to be able to tell
/// them apart: a level that is *closed* here is finished, while a level still *open*
/// where the group ends may be running out to the next system.
#[derive(Copy, Clone)]
pub enum LevelEnd<'a> {
    /// The stem declaring [`BeamType::End`] for this level.
    ClosedAt(&'a Stem),
    /// No `End` in this group: the last stem carrying the level, if any at all.
    OpenAt(Option<&'a Stem>),
}

/// Where a beam level opened at `start` stops: the stem declaring
/// [`BeamType::End`] for it, or -- failing that -- the last stem that carries it.
///
/// A document can open a level and never close it -- `<beam number="2">` running
/// `begin`, `continue`, `continue` with no `end` -- and it survives the rebeam pass
/// whenever each note still declares as many beams as its duration warrants, since
/// that pass compares counts rather than types. Validation reports the defect (see
/// `BeamGroupVisitor`); here the level is taken to run to the last stem that
/// carries it.
///
/// Stopping at the last carrier rather than at the group's last stem matters when
/// the run is genuinely shorter than the group: a pair of sixteenths followed by an
/// eighth describes level 2 on the first two notes only, and extending the
/// secondary beam over the eighth would be wrong in a way the document gave us the
/// information to avoid.
///
/// Since this only ever looks inside one part, an [`OpenAt`](LevelEnd::OpenAt) is
/// ambiguous on its own: either the document left the level unclosed, or the level
/// closes on the next system. Only the caller knows which, because only the caller
/// knows whether the group was cut.
pub fn beam_level_ends_at<'a>(
    chords: &'a [&mut Chord],
    start: usize,
    beam_idx: &u32,
) -> LevelEnd<'a> {
    let mut last_carrier = None;

    for chord in chords.iter().skip(start + 1) {
        let Some(stem) = chord.stem.as_ref() else {
            continue;
        };
        let Some(beam) = stem.beams.get(beam_idx) else {
            continue;
        };

        last_carrier = Some(stem);

        if let BeamType::End = beam {
            return LevelEnd::ClosedAt(stem);
        }
    }

    LevelEnd::OpenAt(last_carrier)
}

/// The point `dx` to the side of `from` measured along `ray`, so a horizontal
/// length lands on the beam's slant rather than level with where it started.
fn point_along(from: XY, dx: f32, ray: &Ray) -> XY {
    let vertical = Ray::from_dir(from.mv(dx, 0.), XY { x: 0., y: -1. });

    vertical.intersect(*ray).unwrap()
}

/// One beam segment as a filled quad: the centre line extruded by `dy` and pulled
/// back half of it, so the beam straddles the line rather than hanging off one
/// side.
fn beam_quad(start: XY, end: XY, thickness: &f32, dy: f32, color: &Color) -> Polygon {
    Line {
        start,
        end,
        stroke_color: *color,
        stroke_width: *thickness,
    }
    .extrude(XY { x: 0., y: dy })
    .mv(0., dy / -2.)
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
