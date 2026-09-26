//! Beams: the beam-group and fragment vocabulary, and the pure list handling
//! that turns a run of chords into both -- the mirror of
//! [`tie`](crate::score::visual::tie) for the other whole-score pass.
//!
//! Nothing here touches the score tree;
//! [`BeamArranger`](crate::score::visual::arranger::BeamArranger) does
//! that. Keeping the grouping and fragmenting as pure functions over slices is
//! what lets them be unit-tested without an arranged score.

use crate::geometry::color::Color;
use crate::geometry::ray::Ray;
use crate::geometry::xy::XY;
use crate::score::core::note_kind::NoteKind;
use crate::score::layout_options::APP_DEFAULTS;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::stem::{BeamType, Stem, UpDown};
use crate::score::visual::system::SystemKey;

/// One chord in a part's beamable sequence, with what the beam pass needs to
/// know about where it ended up: the system it landed on, and the line space of
/// the staff it sits on.
pub struct Beamable<'a> {
    pub key: SystemKey,
    pub space: f32,
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
    /// The line space of the staff the fragment's first chord sits on, which
    /// the slant rule measures intervals and caps in.
    pub space: f32,
    pub chords: Vec<&'a mut Chord>,
    pub cut: Cut,
}

/// One stem of a beamed fragment, reduced to the three numbers the slant rule
/// reads.
#[derive(Copy, Clone, Debug)]
pub struct BeamAnchor {
    /// Where the stem stands.
    pub x: f32,
    /// The chord's note nearest the beam: its highest for stems up, its lowest
    /// for stems down.
    pub note_y: f32,
    /// How far out the beam has to lie for this stem to keep its natural
    /// length: the natural tip, pushed out by room for every level it carries.
    pub reach_y: f32,
}

/// Everything the drawn beams look like, resolved once from [`LayoutParams`]
/// rather than per measure, since none of it varies per measure. The mirror of
/// [`TieMetrics::resolve`](crate::score::visual::tie::TieMetrics::resolve).
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
            font,
        } = params;

        Self {
            thickness: user_layout
                .beam
                .thickness
                .or(score_defaults.appearance.beam)
                .or(font.layout.beam.thickness)
                .unwrap_or(APP_DEFAULTS.beam.thickness),
            spacing: user_layout
                .beam
                .spacing
                .or(font.layout.beam.spacing)
                .unwrap_or(APP_DEFAULTS.beam.spacing),
            grace_scale: params.note_size(NoteKind::Grace),
            color: params.foreground_color(),
        }
    }
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
/// Consecutive runs are all that is needed: the run a group is built from is
/// appended in document order, so a group's [`SystemKey`]s are non-decreasing
/// and every chord on one system is adjacent to the rest of them.
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
                space: beamable.space,
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

/// How long a hook on the stem at `at` is drawn: half the gap to the nearest
/// stemmed chord on the side the hook points to, `forward` meaning rightward,
/// but never more than `max`.
///
/// Half, because the neighbour may well carry a hook pointing back the other
/// way, and two hooks meeting in the middle would read as a beam. With no
/// neighbour on that side -- a hook on the fragment's outermost stem -- there is
/// no gap to measure and the hook gets its full length.
pub fn hook_length(chords: &[&mut Chord], at: usize, forward: bool, max: f32) -> f32 {
    let Some(stem) = chords[at].stem.as_ref() else {
        return max;
    };

    let neighbour = if forward {
        chords[at + 1..]
            .iter()
            .find_map(|chord| chord.stem.as_ref())
    } else {
        chords[..at]
            .iter()
            .rev()
            .find_map(|chord| chord.stem.as_ref())
    };

    match neighbour {
        Some(neighbour) => max.min((neighbour.xy.x - stem.xy.x).abs() / 2.),
        None => max,
    }
}

/// The line a fragment's outermost beam lies on, for stems that all point
/// `stems`: its slant decided by the notes, its height by the stems.
///
/// The slant, as the vertical distance between the outer stems:
///
/// 1. **Flat** when there is only one stem, when the outer notes sit at the
///    same staff position, or when an inner note lies nearer the beam than both
///    outer ones -- a beam following the outer notes would then run into the
///    inner note's stem.
/// 2. Otherwise **a quarter space per staff step** between the outer notes,
///    rising or falling with them.
/// 3. **At most two spaces** for a two-note group and **one space** for three
///    or more, since a longer group reads as steep at a smaller angle.
/// 4. **At most a quarter** of the horizontal distance between the outer stems,
///    so closely spaced groups do not come out near vertical.
///
/// Both caps are multiplied by `max_scale`, the factor a grace group is drawn
/// at. Staff-line quantization -- nudging each end to sit on, straddle or hang
/// from a line -- is not applied.
///
/// The height is then whatever keeps every stem at least as long as its
/// natural length: the line is pushed out until it clears every anchor's
/// `reach_y`. That is where the notes' position on the staff enters: an inner
/// note standing out towards the beam lifts the whole beam rather than having
/// its stem cut short.
pub fn fit_beam(anchors: &[BeamAnchor], stems: UpDown, space: f32, max_scale: f32) -> Option<Ray> {
    let first = anchors.first()?;
    let last = anchors.last()?;

    // `outward * y` grows the further a point lies towards the beam: upward (a
    // smaller y) for stems up, downward for stems down.
    let outward = match stems {
        UpDown::Up => -1.,
        UpDown::Down => 1.,
    };

    let span = last.x - first.x;
    let slope = if span > f32::EPSILON {
        let dy = slant(anchors, outward, space, max_scale).min(span / 4.);
        (last.note_y - first.note_y).signum() * dy / span
    } else {
        0.
    };

    // The line's height at the first stem, taken as far out as the most
    // demanding anchor requires.
    let height = anchors
        .iter()
        .map(|anchor| anchor.reach_y - slope * (anchor.x - first.x))
        .max_by(|a, b| (outward * a).total_cmp(&(outward * b)))?;

    Some(Ray::from_dir(
        XY {
            x: first.x,
            y: height,
        },
        XY { x: 1., y: slope },
    ))
}

/// The unsigned slant [`fit_beam`] applies, before the span cap: rules 1 to 3.
fn slant(anchors: &[BeamAnchor], outward: f32, space: f32, max_scale: f32) -> f32 {
    let [first, inner @ .., last] = anchors else {
        return 0.;
    };

    let ends = (outward * first.note_y).max(outward * last.note_y);
    if inner.iter().any(|anchor| outward * anchor.note_y > ends) {
        return 0.;
    }

    let steps = ((last.note_y - first.note_y).abs() / (space / 2.)).round();
    let cap = if anchors.len() == 2 { 2. } else { 1. } * space * max_scale;

    (steps * space / 4.).min(cap)
}
