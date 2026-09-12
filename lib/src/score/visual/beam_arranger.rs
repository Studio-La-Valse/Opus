//! Resolves beam groups into drawn segments, after the pages have been arranged.
//!
//! Beams used to be a `PartMeasure`'s own business: it collected its chords,
//! grouped them and emitted the segments into itself. That works right up to the
//! barline and no further -- a group cannot outlive the sequence it was built
//! from, and that sequence was one measure long. So the pass moved out here,
//! beside [`tie_arranger`](crate::score::visual::tie_arranger), and runs on the
//! whole score at once for the same reason ties do: once the pages are arranged
//! every coordinate in the tree is absolute, so a group whose ends sit in
//! different measures -- or different systems, or different pages -- is just
//! arithmetic.
//!
//! What it deliberately does *not* borrow from ties is their index. A tie pairs
//! two arbitrary notes, so it needs a [`NoteId`](crate::score::visual::note::NoteId)
//! and a flat `Score::ties` to name them. A beam group is something weaker: a
//! maximal run of consecutive chords in one voice of one part. Document order
//! already carries that relation, so [`collect_runs`] rebuilds the run by
//! walking the tree and there is no second source of truth to keep in step
//! with it.
//!
//! # On idempotency
//!
//! This pass is not idempotent on its own: [`Stem::attach_ray`] writes an
//! absolute stem length and [`create_ray`] reads [`Stem::tip`], which depends on
//! that length. It does not have to be. `Chord::arrange_stem` recomputes
//! `length` from `default_y` on every arrange and still runs first, inside
//! `Part::arrange_clear_of`, so repeated `arrange_score` calls on a cached score
//! converge -- which is what the wasm render path depends on. Worth saying out
//! loud here, because the reset is now a long way from the code relying on it.

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
use crate::score::visual::score::Score;
use crate::score::visual::stem::{BeamType, Stem, UpDown};
use crate::score::visual::system::SystemKey;
use crate::score::walk_cursor::Visibility;

/// Maximum vertical span a beam is allowed to slant before it is clamped.
const MAX_BEAM_SLANT_DY: f32 = 20.;

/// The length of a hook beam. TODO: infer from available space between two stems and clam to a max length.
const HOOK_LENGTH: f32 = 7.5;

/// One chord in a part's beamable sequence, with the one thing the beam pass
/// needs to know about where it ended up.
pub struct Beamable<'a> {
    pub key: SystemKey,
    pub chord: &'a mut Chord,
}

/// Whether a fragment's group carries on past it, and so on which side the
/// levels reaching the fragment's edge run a stub out to the break.
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Cut {
    pub left: bool,
    pub right: bool,
}

/// A maximal run of one beam group within one system: what actually gets a ray
/// fitted and segments drawn.
///
/// A group inside one system is a single fragment and is drawn exactly as it
/// always was -- the ray simply spans more measures than it used to. Only a group
/// crossing a break has more than one, and `cut` is all the geometry needs to
/// know about that.
pub struct Fragment<'a> {
    pub key: SystemKey,
    pub chords: Vec<&'a mut Chord>,
    pub cut: Cut,
}

/// What gets beamed together: one voice of one part, with grace notes kept apart
/// from the rest. Exactly the grouping `BeamGroupVisitor` validates against.
///
/// Notably *not* keyed by measure. That is the whole of what lets a group cross a
/// barline: the sequence handed to [`create_beam_groups`] is now the part's, not
/// the measure's, so a group still open at a barline simply carries on.
type RunKey = (String, Voice, bool);

/// Everything the drawn beams look like, resolved once from [`LayoutParams`]
/// rather than per measure, since none of it varies per measure. The mirror of
/// [`TieMetrics::resolve`](crate::score::visual::tie::TieMetrics).
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

/// Rebuilds every system's beam segments from the chords in the tree.
///
/// Runs after `LayoutEngine::arrange_pages`, and assigns rather than appends, so
/// calling it repeatedly leaves the same segments behind -- the property
/// `PartMeasure` used to get from clearing `self.beams` first.
pub fn arrange_beams(score: &mut Score, params: LayoutParams<'_>) {
    let metrics = BeamMetrics::resolve(params);

    let mut out: BTreeMap<SystemKey, Vec<Polygon>> = BTreeMap::new();

    for ((_, _, grace), run) in collect_runs(score) {
        // A grace group's beams are reduced by the same factor its noteheads
        // were; a normal one is drawn at full size.
        let scale = if grace { metrics.grace_scale } else { 1. };
        let thickness = metrics.thickness * scale;
        let spacing = metrics.spacing * scale;

        for group in create_beam_groups(run) {
            // Inferred once for the whole group, so the beam stack grows the same
            // way on either side of a break: it is a fact about the group's
            // stems, and half a group's stems would answer differently from all
            // of them.
            let Some(direction) = infer_direction(&group) else {
                continue;
            };

            // The ray, though, is fitted per fragment, from that fragment's own
            // stems -- which is what makes the slant right on both systems.
            for mut fragment in split_at_system_breaks(group) {
                let Some(ray) = create_ray(&fragment.chords, &direction, &thickness, &spacing)
                else {
                    continue;
                };

                out.entry(fragment.key).or_default().extend(create_beams(
                    &fragment.chords,
                    &ray,
                    &direction,
                    &thickness,
                    &spacing,
                    &metrics.color,
                    fragment.cut,
                ));

                adjust_stem_lengths(&mut fragment.chords, &ray, &direction, &thickness, &spacing);
            }
        }
    }

    for (page_key, page) in score.pages.iter_mut() {
        for (system_key, system) in page.systems.iter_mut() {
            system.beams = out.remove(&(*page_key, *system_key)).unwrap_or_default();
        }
    }
}

/// Every beamable chord in the score, gathered into the runs that are beamed as
/// a unit.
///
/// The mutable twin of
/// [`collect_note_anchors`](crate::score::visual::tie_arranger::collect_note_anchors):
/// pages, systems, sections, groups, parts, measures, voices, appending in walk
/// order so each run comes out in document order -- which is what lets the
/// grouping below treat "consecutive" as a fact about the sequence rather than
/// something it has to look up.
///
/// Skips hidden parts, and for a sharper reason than the anchors have. A hidden
/// part's beams used to be invisible because they hung off a `PartMeasure` that
/// the compositor's `walk_part` never reached; now that they are filed on the
/// `System`, nothing downstream filters them.
fn collect_runs(score: &mut Score) -> BTreeMap<RunKey, Vec<Beamable<'_>>> {
    let mut runs: BTreeMap<RunKey, Vec<Beamable<'_>>> = BTreeMap::new();

    for (page_key, page) in score.pages.iter_mut() {
        for (system_key, system) in page.systems.iter_mut() {
            let key = (*page_key, *system_key);

            for section in system.sections.values_mut() {
                for group in section.part_groups.values_mut() {
                    for (part_id, part) in group.parts.iter_mut() {
                        if part.visibility == Visibility::Hidden {
                            continue;
                        }

                        for measure in part.measures.values_mut() {
                            for (voice, chords) in measure.chords.iter_mut() {
                                for chord in chords.iter_mut() {
                                    let run = (part_id.clone(), *voice, chord.grace);
                                    runs.entry(run).or_default().push(Beamable { key, chord });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    runs
}

enum BeamGroupAction {
    Start,
    Continue,
    End,
    Standalone,
    Panic(&'static str),
}

/// Chunks one run into the groups that are beamed together: a group runs from
/// the chord declaring [`BeamType::Start`] at level 1 to the one declaring
/// [`BeamType::End`], and a chord carrying no beam at all stands alone.
///
/// The three ways a document can contradict itself here -- opening a group while
/// one is open, continuing or ending one that was never opened -- used to be
/// panics, on the reading that a group could not legally cross the barline so
/// any of them meant the document was broken. Now that a run is a whole part's,
/// they mean only that the document is broken, and `BeamGroupVisitor` already
/// reports each as a `Warning`. A repairable document should draw something
/// rather than abort the render, so each one recovers.
pub fn create_beam_groups(chords: Vec<Beamable<'_>>) -> Vec<Vec<Beamable<'_>>> {
    let mut result = vec![];
    let mut group: Vec<Beamable<'_>> = vec![];

    for beamable in chords {
        let action = match &beamable.chord.stem {
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
            // A second `Start` closes whatever was open: the level-1 `begin`
            // says a group starts here, so whatever came before it ended.
            BeamGroupAction::Start => {
                if !group.is_empty() {
                    result.push(group);
                    group = vec![];
                }
                group.push(beamable);
            }
            // A `Continue` or `End` with nothing open opens the group it claims
            // to carry on, which is the least the document can be taken to mean.
            BeamGroupAction::Continue => {
                group.push(beamable);
            }
            BeamGroupAction::End => {
                group.push(beamable);
                result.push(group);
                group = vec![];
            }
            BeamGroupAction::Standalone => {
                group.push(beamable);
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

/// Chunks one group into its fragments, one per system it reaches into.
///
/// Consecutive runs are all that is needed: `collect_runs` appends in document
/// order, so a group's [`SystemKey`]s are non-decreasing and every chord on one
/// system is adjacent to the rest of them.
///
/// A group that never leaves its system comes out as one uncut fragment, which is
/// the overwhelmingly common case and the one that has to stay exactly as it was.
pub fn split_at_system_breaks<'a>(group: Vec<Beamable<'a>>) -> Vec<Fragment<'a>> {
    let mut fragments: Vec<Fragment<'a>> = Vec::new();

    for beamable in group {
        match fragments.last_mut() {
            Some(fragment) if fragment.key == beamable.key => fragment.chords.push(beamable.chord),
            _ => fragments.push(Fragment {
                key: beamable.key,
                chords: vec![beamable.chord],
                cut: Cut::default(),
            }),
        }
    }

    // A fragment is cut wherever another fragment of the same group lies: every
    // one but the first arrives from a break, every one but the last runs out
    // into one.
    let last = fragments.len().saturating_sub(1);
    for (i, fragment) in fragments.iter_mut().enumerate() {
        fragment.cut = Cut {
            left: i > 0,
            right: i < last,
        };
    }

    fragments
}

/// Which way the beam stack grows for a whole group: away from the stems when
/// they all point the same way, and towards the odd one out when they do not
/// (a cross-staff group, where the stems of one staff point up and the other's
/// down).
pub fn infer_direction(group: &[Beamable<'_>]) -> Option<UpDown> {
    let stems: Vec<&Stem> = group.iter().filter_map(|b| b.chord.stem.as_ref()).collect();

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
    // Between the outermost chords that *have* a stem, not the outermost chords:
    // a chord without one carries no beam and so cannot bound the span. It used
    // to be safe to assume the group's own ends had stems, because a group was
    // flushed at the barline; a group carrying on into the next measure can now
    // be closed by a stemless chord arriving there.
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

    // One stem on its own has no span to beam across -- unless the group carries
    // on past this fragment, in which case a chord stranded alone on the far side
    // of a break still owes its stubs.
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
            // A level the fragment's *first* stem merely carries -- `continue` or
            // `end`, never `begin` -- was opened on the previous system, so this
            // stem stands in for the `begin` that is not in this fragment and the
            // beam reaches a stub back past it to the break. `end` says the level
            // closes here, so its span is this one stem.
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

                    // The level runs out to the next system when it has no `end`
                    // in this fragment *and* the fragment's last stem is still
                    // carrying it. A level that stops inside the fragment -- a
                    // pair of sixteenths early in a longer group -- gets no stub
                    // however the fragment was cut.
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
                        // Nothing else in the fragment carries the level. When
                        // the group runs past this fragment the level spans this
                        // stem alone and the stubs do the rest; when it does not,
                        // a level spanning one stem is a hook, which is what it
                        // would have been written as.
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

/// Where a beam level opened at `start` stops within the fragment.
///
/// Two outcomes that used to be one `Option`, and the caller has to be able to
/// tell them apart: a level that is *closed* here is finished, while a level
/// still *open* at the fragment's edge may be running out to the next system.
#[derive(Copy, Clone)]
pub enum LevelEnd<'a> {
    /// The stem declaring [`BeamType::End`] for this level.
    ClosedAt(&'a Stem),
    /// No `End` in this fragment: the last stem carrying the level, if any at
    /// all.
    OpenAt(Option<&'a Stem>),
}

/// Where a beam level opened at `start` stops: the stem declaring
/// [`BeamType::End`] for it, or -- failing that -- the last stem that carries it.
///
/// A document can open a level and never close it -- `<beam number="2">` running
/// `begin`, `continue`, `continue` with no `end` -- and it survives the rebeam
/// pass whenever each note still declares as many beams as its duration warrants,
/// since that pass compares counts rather than types. Validation reports the
/// defect (see `BeamGroupVisitor`); here the level is taken to run to the last
/// stem that carries it.
///
/// Stopping at the last carrier rather than at the fragment's last stem matters
/// when the run is genuinely shorter than the group: a pair of sixteenths
/// followed by an eighth describes level 2 on the first two notes only, and
/// extending the secondary beam over the eighth would be wrong in a way the
/// document gave us the information to avoid.
///
/// Since this only ever looks inside one fragment, an [`OpenAt`](LevelEnd::OpenAt)
/// is now ambiguous on its own: either the document left the level unclosed, or
/// the level closes on the far side of a system break. Only the caller knows
/// which, because only the caller knows whether the fragment was cut.
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

/// One beam segment as a filled quad: the centre line extruded by `dy` and
/// pulled back half of it, so the beam straddles the line rather than hanging
/// off one side.
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
