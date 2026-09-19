//! Resolves beam groups into drawn segments, after the pages have been
//! arranged.
//!
//! Beams used to be a `PartMeasure`'s own business: it collected its chords,
//! grouped them and emitted the segments into itself. That works right up to the
//! barline and no further -- a group cannot outlive the sequence it was built
//! from, and that sequence was one measure long. So the pass moved out here,
//! beside [`TieArranger`](crate::score::visual::arranger::TieArranger), and runs on the
//! whole score at once for the same reason ties do: once the pages are arranged
//! every coordinate in the tree is absolute, so a group whose ends sit in
//! different measures -- or different systems, or different pages -- is just
//! arithmetic.
//!
//! What it deliberately does *not* borrow from ties is their index. A tie pairs
//! two arbitrary notes, so it needs a [`NoteId`](crate::score::visual::note::NoteId)
//! and a flat `Score::ties` to name them. A beam group is something weaker: a
//! maximal run of consecutive chords in one voice of one part. Document order
//! already carries that relation, so [`BeamArranger::collect_runs`] rebuilds the run by
//! walking the tree and there is no second source of truth to keep in step
//! with it.
//!
//! # Idempotency
//!
//! [`Stem::attach_ray`](crate::score::visual::stem::Stem::attach_ray) writes an
//! absolute stem length, and `create_ray` reads
//! [`Stem::tip`](crate::score::visual::stem::Stem::tip), which depends on it --
//! so fitting a ray from a stem this pass already adjusted would fit against
//! the wrong tip. [`BeamArranger::collect_runs`] avoids that by resetting every beamable
//! chord's stem to its natural length -- via
//! [`Chord::place_stem`](crate::score::visual::chord::Chord::place_stem),
//! the same pure function `ContentArranger` calls during the ordinary
//! arrange -- before the chord is ever handed to a group. Every ray is
//! therefore always fitted from the same tips, however many times
//! `arrange_score` runs on a cached score, which is what the wasm render path
//! depends on.

use std::collections::BTreeMap;

use crate::drawable::elements::line::Line;
use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::ray::Ray;
use crate::geometry::xy::XY;
use crate::score::core::voice::Voice;
use crate::score::visual::arranger::ScoreArranger;
use crate::score::visual::beam::{
    BeamMetrics, Beamable, Cut, LevelEnd, beam_level_ends_at, create_beam_groups, infer_direction,
    split_at_system_breaks,
};
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::score::Score;
use crate::score::visual::stem::{BeamType, UpDown};
use crate::score::visual::system::SystemKey;
use crate::score::walk_cursor::Visibility;

/// Rebuilds every system's beam segments from the chords in the tree.
pub struct BeamArranger;

impl ScoreArranger for BeamArranger {
    /// Assigns rather than appends, so calling it repeatedly leaves the same
    /// segments behind -- the property `PartMeasure` used to get from clearing
    /// `self.beams` first. See the module's `# Idempotency` note for how it
    /// stays idempotent despite adjusting stem lengths as it goes.
    fn arrange(&self, score: &mut Score, params: LayoutParams<'_>) {
        let metrics = BeamMetrics::resolve(params);

        let mut out: BTreeMap<SystemKey, Vec<Polygon>> = BTreeMap::new();

        for ((_, _, grace), run) in self.collect_runs(score) {
            // A grace group's beams are reduced by the same factor its noteheads
            // were; a normal one is drawn at full size.
            let scale = if grace { metrics.grace_scale } else { 1. };
            let thickness = metrics.thickness * scale;
            let spacing = metrics.spacing * scale;

            for group in create_beam_groups(run) {
                // Inferred once for the whole group, so the beam stack grows the
                // same way on either side of a break: it is a fact about the
                // group's stems, and half a group's stems would answer
                // differently from all of them.
                let Some(direction) = infer_direction(&group) else {
                    continue;
                };

                // The ray, though, is fitted per fragment, from that fragment's
                // own stems -- which is what makes the slant right on both
                // systems.
                for mut fragment in split_at_system_breaks(group) {
                    let Some(ray) =
                        self.create_ray(&fragment.chords, &direction, &thickness, &spacing)
                    else {
                        continue;
                    };

                    out.entry(fragment.key)
                        .or_default()
                        .extend(self.create_beams(
                            &fragment.chords,
                            &ray,
                            &direction,
                            &thickness,
                            &spacing,
                            &metrics.color,
                            fragment.cut,
                        ));

                    self.adjust_stem_lengths(
                        &mut fragment.chords,
                        &ray,
                        &direction,
                        &thickness,
                        &spacing,
                    );
                }
            }
        }

        for (page_key, page) in score.pages.iter_mut() {
            for (system_key, system) in page.systems.iter_mut() {
                system.beams = out.remove(&(*page_key, *system_key)).unwrap_or_default();
            }
        }
    }
}

// ---- internals ----

/// Maximum vertical span a beam is allowed to slant before it is clamped.
const MAX_BEAM_SLANT_DY: f32 = 20.;

/// The length of a hook beam. TODO: infer from available space between two stems and clam to a max length.
const HOOK_LENGTH: f32 = 7.5;

/// What gets beamed together: one voice of one part, with grace notes kept apart
/// from the rest. Exactly the grouping `BeamGroupVisitor` validates against.
///
/// Notably *not* keyed by measure. That is the whole of what lets a group cross a
/// barline: the sequence handed to [`create_beam_groups`] is now the part's, not
/// the measure's, so a group still open at a barline simply carries on.
type RunKey = (String, Voice, bool);

impl BeamArranger {
    /// Every beamable chord in the score, gathered into the runs that are beamed as
    /// a unit.
    ///
    /// The mutable twin of
    /// [`Score::note_anchors`](crate::score::visual::score::Score::note_anchors):
    /// pages, systems, sections, groups, parts, measures, voices, appending in walk
    /// order so each run comes out in document order -- which is what lets the
    /// grouping below treat "consecutive" as a fact about the sequence rather than
    /// something it has to look up.
    ///
    /// Skips hidden parts, and for a sharper reason than the anchors have. A hidden
    /// part's beams used to be invisible because they hung off a `PartMeasure` that
    /// the compositor's `walk_part` never reached; now that they are filed on the
    /// `System`, nothing downstream filters them.
    ///
    /// Also resets every chord's stem to its natural length before it is collected
    /// -- see the module's `# Idempotency` note.
    fn collect_runs<'a>(&self, score: &'a mut Score) -> BTreeMap<RunKey, Vec<Beamable<'a>>> {
        let mut runs: BTreeMap<RunKey, Vec<Beamable<'a>>> = BTreeMap::new();

        for (page_key, page) in score.pages.iter_mut() {
            for (system_key, system) in page.systems.iter_mut() {
                let key = (*page_key, *system_key);

                for section in system.sections.values_mut() {
                    for group in section.part_groups.values_mut() {
                        for (part_id, part) in group.parts.iter_mut() {
                            if part.visibility == Visibility::Hidden {
                                continue;
                            }

                            let staff_ctx = part.create_staff_ctx();

                            for measure in part.measures.values_mut() {
                                for (voice, chords) in measure.chords.iter_mut() {
                                    for chord in chords.iter_mut() {
                                        chord.place_stem(&staff_ctx);

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

    fn create_ray(
        &self,
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
        &self,
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

                let offset = self.create_offset(direction, beam_idx, beam_thickness, beam_spacing);
                let offset_ray = ray.mv(0., offset);
                let dx = (-left_stem.thickness * left_stem.scale) / 2.;
                let mut left_point = Ray {
                    origin: left_stem.xy.mv(dx, 0.),
                    dir: XY { x: 0., y: 1. },
                }
                .intersect(offset_ray)
                .unwrap();

                if arrives {
                    left_point = self.point_along(left_point, -HOOK_LENGTH, &offset_ray);
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
                            None => self.point_along(left_point, HOOK_LENGTH, &offset_ray),
                        };

                        if departs {
                            self.point_along(point, HOOK_LENGTH, &offset_ray)
                        } else {
                            point
                        }
                    }
                    BeamType::HookStart => self.point_along(left_point, HOOK_LENGTH, &offset_ray),
                    BeamType::HookEnd => self.point_along(left_point, -HOOK_LENGTH, &offset_ray),
                    _ => continue,
                };

                beams.push(self.beam_quad(left_point, right_point, beam_thickness, dy, color));
            }
        }

        beams
    }

    /// The point `dx` to the side of `from` measured along `ray`, so a horizontal
    /// length lands on the beam's slant rather than level with where it started.
    fn point_along(&self, from: XY, dx: f32, ray: &Ray) -> XY {
        let vertical = Ray::from_dir(from.mv(dx, 0.), XY { x: 0., y: -1. });

        vertical.intersect(*ray).unwrap()
    }

    /// One beam segment as a filled quad: the centre line extruded by `dy` and
    /// pulled back half of it, so the beam straddles the line rather than hanging
    /// off one side.
    fn beam_quad(&self, start: XY, end: XY, thickness: &f32, dy: f32, color: &Color) -> Polygon {
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
        &self,
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
        &self,
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

            let offset = self.create_offset(direction, beam_idx, beam_thickness, beam_spacing);
            let offset_ray = ray.mv(0., offset);

            stem.attach_ray(&offset_ray);
        }
    }
}
