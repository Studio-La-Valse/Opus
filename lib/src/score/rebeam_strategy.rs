use crate::visual::chord::Chord;
use crate::visual::stem::BeamType;

pub trait RebeamStrategy {
    fn rebeam(&self, chords: &mut [&mut Chord]);
}

pub struct OnlyWhenRequiredRebeamStrategy {
    pub imp: Box<dyn RebeamStrategy>,
}

impl RebeamStrategy for OnlyWhenRequiredRebeamStrategy {
    fn rebeam(&self, chords: &mut [&mut Chord]) {
        if !_requires_rebeam(chords) {
            return;
        }

        self.imp.rebeam(chords);
    }
}

pub struct SimpleRebeamStrategy;

impl RebeamStrategy for SimpleRebeamStrategy {
    fn rebeam(&self, chords: &mut [&mut Chord]) {
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
}

fn _requires_rebeam(chords: &[&mut Chord]) -> bool {
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
