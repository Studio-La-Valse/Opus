use crate::score::core::pitch::Pitch;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Clef {
    #[default]
    Treble,
    Soprano,
    MezzoSoprano,
    Alto,
    Tenor,
    Baritone,
    Bass,
    Percussion,
    /// A tablature staff's clef. Like [`Percussion`](Self::Percussion) it fixes
    /// no pitch: a tab staff positions a note by string and fret
    /// (`<technical>`), not by where its `<pitch>` lands. The pitch-derived
    /// methods below therefore answer for it only so that a document carrying a
    /// tab staff engraves rather than panicking -- the notes on that staff come
    /// out at treble-ish positions, which is the same limitation percussion
    /// already has. Real tablature (fret numbers in place of noteheads,
    /// `<staff-tuning>`) is a feature of its own; the staff does now get its
    /// declared number of lines, which for a tab staff is its string count.
    Tab,
}

impl Clef {
    /// The line on the staff that denotes the middle alignment of a smufl glyph,
    /// in half-spaces down from the top line of a staff of `staff_lines` lines.
    ///
    /// A pitched clef names a pitch, and a pitch has one place on the staff
    /// whatever the staff looks like: the G of a treble clef is the G that
    /// [`line_index_at_pitch`](Self::line_index_at_pitch) would put a written G4
    /// on, so those anchors are fixed and ignore `staff_lines`. The two clefs
    /// that name no pitch have nothing to be fixed to and are centred on the
    /// staff instead -- which for the usual five lines is the middle line, the
    /// same 4 as before.
    pub fn anchor_line(&self, staff_lines: usize) -> i32 {
        match self {
            Self::Treble => 6,
            Self::Soprano => 8,
            Self::MezzoSoprano => 6,
            Self::Alto => 4,
            Self::Tenor => 2,
            Self::Baritone => 0,
            Self::Bass => 2,
            Self::Percussion | Self::Tab => Self::middle_line(staff_lines),
        }
    }

    /// The half-space index, from the top drawn line, a pitchless clef or a
    /// rest centres on: the middle of the drawn lines -- the middle line when
    /// there is an odd number of them, the middle space when there is an even
    /// number.
    ///
    /// The exception is a one-line staff: its single line stands in for the
    /// middle line of a five-line staff, drawn two spaces down from the top of
    /// the block the staff occupies, so what centres on it is two spaces down
    /// too -- index 4, not 0.
    fn middle_line(staff_lines: usize) -> i32 {
        if staff_lines == 1 {
            // Two spaces down: the middle line of the five-line staff a
            // one-line staff stands in for.
            4
        } else {
            staff_lines.saturating_sub(1) as i32
        }
    }

    /// The line on the staff that denotes the location of the middle c.
    pub fn line_middle_c(&self) -> i32 {
        match self {
            Self::Treble => 10,
            Self::Soprano => 8,
            Self::MezzoSoprano => 6,
            Self::Alto => 4,
            Self::Tenor => 2,
            Self::Baritone => 0,
            Self::Bass => -2,
            Self::Percussion => 4,
            Self::Tab => 4,
        }
    }

    pub fn from_mxml(sign: &str, line: Option<i32>) -> Option<Self> {
        match sign.to_lowercase().as_str() {
            "g" => Some(Self::Treble),
            "c" => match line {
                Some(1) => Some(Self::Soprano),
                Some(2) => Some(Self::MezzoSoprano),
                Some(3) => Some(Self::Alto),
                Some(4) => Some(Self::Tenor),
                Some(5) => Some(Self::Baritone),
                _ => None,
            },
            "f" => Some(Self::Bass),
            "percussion" => Some(Self::Percussion),
            "tab" => Some(Self::Tab),
            _ => None,
        }
    }

    pub fn sharp_lines(&self) -> Vec<i32> {
        // Requires a match because there is no true mathematical pattern
        // as the positions are irregular.
        // We could use a vec of octaves and calculate the pitch for each step,
        // but this will do.
        match self {
            Self::Treble => vec![0, 3, -1, 2, 5, 1, 4],
            Self::Soprano => vec![5, 1, 4, 0, 3, -1, 2],
            Self::MezzoSoprano => vec![3, 6, 2, 5, 8, 4, 7],
            Self::Alto => vec![1, 4, 0, 3, 6, 2, 5],
            Self::Tenor => vec![6, 2, 5, 1, 4, 0, 3],
            Self::Baritone => vec![4, 7, 3, 6, 9, 5, 8],
            Self::Bass => vec![2, 5, 1, 4, 7, 3, 6],
            // Neither carries a key signature at all.
            Self::Percussion | Self::Tab => vec![],
        }
    }

    pub fn flat_lines(&self) -> Vec<i32> {
        // Strictly speaking the arrangement of flats seems to follow a pattern,
        // but since we use a match in the sharp_lines() fn, we use it here as well.
        match self {
            Self::Treble => vec![4, 1, 5, 2, 6, 3, 7],
            Self::Soprano => vec![2, -1, 3, 0, 4, 1, 5],
            Self::MezzoSoprano => vec![0, 4, 1, 5, 2, 6, 3],
            Self::Alto => vec![5, 2, 6, 3, 7, 4, 8],
            Self::Tenor => vec![3, 0, 4, 1, 5, 2, 6],
            Self::Baritone => vec![1, 5, 2, 6, 3, 7, 4],
            Self::Bass => vec![6, 3, 7, 4, 8, 5, 9],
            Self::Percussion | Self::Tab => vec![],
        }
    }

    /// Returns the line on a staff for the requested pitch.
    pub fn line_index_at_pitch(&self, pitch: &Pitch) -> i32 {
        // Calculate total diatonic steps from Middle C (C4)
        let octave_diff = pitch.octave - 4;
        let diatonic_steps_from_c4 = (octave_diff * 7) + pitch.step.steps_from_c;

        // Moving up in pitch moves up the staff (decreasing line index)
        self.line_middle_c() - diatonic_steps_from_c4
    }
}
