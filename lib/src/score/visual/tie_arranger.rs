//! Resolves one part's ties into drawn arcs.
//!
//! A tie is bound to a part -- in fact to a note and the note that follows it --
//! so this is an ordinary local layout step, the twin of
//! [`arrange_beams`](crate::score::visual::beam_arranger::arrange_beams).
//!
//! Where a tie ends is not something the document has to be asked. A tie joins a
//! note to the *next* note of that pitch in its voice, so the end is simply the
//! next note ahead on the same staff line: the content walk records only that a
//! note is tied, and the search below finds what it is tied to.
//!
//! # Ties across a system break
//!
//! A [`Part`] is one system's worth of one instrument, so a tie whose end note
//! falls on the next system finds nothing ahead of it here. That is not an error
//! and needs nothing from the other side: running out of notes *is* the signal,
//! and the tie is drawn as a complete arc leaving its note and running out to
//! the end of the part.
//!
//! Only that half is drawn. Printed music also puts a short courtesy arc in
//! front of the note the tie arrives at on the next system, and this does not.
//! That half is the one a part genuinely cannot work out alone: nothing in the
//! next part records that a tie is arriving, and the note itself does not know
//! it was already sounding. Settling it means either deciding at walk time which
//! ties are broken -- which freezes the system breaking into the cached walk --
//! or giving this pass a look at the part before it. Both are worth doing;
//! neither is here.

use std::collections::BTreeMap;

use crate::score::core::voice::Voice;
use crate::score::visual::chord::Chord;
use crate::score::visual::note::Note;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::tie::TieAnchor;

/// Rebuilds every tie arc in `part` from its own notes.
///
/// Runs from [`Part::arrange_clear_of`] once its measures have been arranged, so
/// that every note in it has its final position -- and assigns rather than
/// appends, so calling it repeatedly is idempotent, which is the property the
/// wasm render path depends on when it re-arranges a cached score for a new
/// `UserLayout`.
pub fn arrange_ties(part: &mut Part) {
    let metrics = part.tie_metrics;

    // Where a tie with no note left to reach runs out to: the end of the part,
    // less a margin so it clears the final barline rather than touching it.
    let part_end = part.xy.x + part.width - metrics.break_inset;

    for mut chords in collect_voices(part.measures.values_mut()) {
        for i in 0..chords.len() {
            // Everything after this chord is where the ties leaving it look for
            // their ends, and splitting the run keeps that search readable from
            // the chord itself.
            let (head, ahead) = chords.split_at_mut(i + 1);
            let chord = &mut *head[i];

            let stem = chord.stem.as_ref().map(|stem| stem.direction);

            for note in chord.notes.iter_mut() {
                let start = note.tie_anchor();

                let Some(tie) = note.tie.as_mut() else {
                    continue;
                };

                match target(ahead, &start) {
                    Some(end) => tie.arrange(&start, &end, stem, &metrics),
                    // Nothing ahead to tie to: the end note is on the next
                    // system, so the tie runs off the end of this part instead.
                    None => tie.arrange_open(&start, stem, part_end, &metrics),
                }
            }
        }
    }
}

/// One part's chords, one run per voice, in document order.
///
/// The order is what makes the search above work: measures are keyed by number,
/// so walking them in key order walks them in document order, and "the next note
/// of this pitch" is then a fact about the sequence rather than something that
/// has to be looked up.
///
/// Grace notes are kept in place rather than split off the way
/// [`arrange_beams`](crate::score::visual::beam_arranger::arrange_beams) splits
/// them, because a grace note can be tied to the note it decorates.
fn collect_voices<'a>(
    measures: impl Iterator<Item = &'a mut PartMeasure>,
) -> Vec<Vec<&'a mut Chord>> {
    let mut voices: BTreeMap<Voice, Vec<&mut Chord>> = BTreeMap::new();

    for measure in measures {
        for (voice, chords) in measure.chords.iter_mut() {
            voices.entry(*voice).or_default().extend(chords.iter_mut());
        }
    }

    voices.into_values().collect()
}

/// The note a tie arrives at: the first one ahead of it on the same line of the
/// same staff, which is what "the next note of that pitch" looks like once the
/// pitches are gone.
///
/// `None` means this part has no such note left, so the tie's end is on the next
/// system -- see the module docs.
fn target(ahead: &[&mut Chord], start: &TieAnchor) -> Option<TieAnchor> {
    ahead
        .iter()
        .flat_map(|chord| chord.notes.iter())
        .find(|note| note.staff == start.staff && note.staff_line == start.staff_line)
        .map(Note::tie_anchor)
}
