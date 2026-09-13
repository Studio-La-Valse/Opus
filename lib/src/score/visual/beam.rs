//! Beams: the beam-group and fragment vocabulary, and the pure list handling
//! that turns a run of chords into both -- the mirror of
//! [`tie`](crate::score::visual::tie) for the other whole-score pass.
//!
//! Nothing here touches the score tree;
//! [`BeamArranger`](crate::score::visual::arranger::BeamArranger) does
//! that. Keeping the grouping and fragmenting as pure functions over slices is
//! what lets them be unit-tested without an arranged score.

use crate::geometry::color::Color;
use crate::score::core::note_kind::NoteKind;
use crate::score::visual::chord::Chord;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::stem::{BeamType, Stem, UpDown};
use crate::score::visual::system::SystemKey;

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
