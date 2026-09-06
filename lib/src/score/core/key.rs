use crate::score::core::step::Step;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Major,
    Minor,
}

impl TryFrom<&str> for Mode {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "major" => Ok(Mode::Major),
            "minor" => Ok(Mode::Minor),
            _ => Err(format!("invalid mode: {}", value)),
        }
    }
}

/// A key, held as its **tonic** plus mode rather than as a signed accidental
/// count.
///
/// The count is what MusicXML writes (`<fifths>`) and what gets engraved, but it
/// is ambiguous as a field: "3" reads equally well as "A major" and as "the key
/// with three sharps", and a `Key` carrying a mode alongside it invites the
/// reader to apply the mode twice. C minor is `<fifths>-3</fifths>` *and* three
/// flats -- the mode is already baked in. Storing the tonic removes the
/// ambiguity: [`Key::from_mxml`] is the one place the document's count is turned
/// into a tonic, and [`Key::accidentals`] is the one place a tonic is turned
/// back into a count.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct Key {
    /// The tonic. For C minor this is C (not E flat, its relative major).
    pub step: Step,
    pub mode: Mode,
}

impl Key {
    /// C major -- no accidentals. The key in force until a document says otherwise.
    pub const C_MAJOR: Key = Key {
        step: Step {
            steps_from_c: 0,
            alter: 0,
        },
        mode: Mode::Major,
    };

    /// Builds a key from MusicXML's `<key>`: `fifths` is the number of sharps
    /// (positive) or flats (negative) in the printed signature, and already
    /// accounts for the mode.
    ///
    /// `mode` therefore only decides *which* tonic that signature belongs to --
    /// three flats is E flat major or C minor -- and never how many accidentals
    /// come back out of [`accidentals`](Self::accidentals).
    ///
    /// A `<mode>` the document omits, or spells `none` / as a church mode,
    /// should be read as [`Mode::Major`] by the caller. That names the wrong
    /// tonic for, say, a dorian key, but a mode's signature is its relative
    /// major's, so the engraved accidentals come out identical either way.
    pub fn from_mxml(fifths: i8, mode: Mode) -> Key {
        // Undo the mode: a minor key's tonic sits three fifths above the major
        // tonic that shares its signature (0 flats is C major, but A minor).
        let tonic_fifths = i32::from(fifths)
            + match mode {
                Mode::Major => 0,
                Mode::Minor => 3,
            };

        Key {
            step: step_at_fifths(tonic_fifths),
            mode,
        }
    }

    /// The printed signature: a positive value is that many sharps, a negative
    /// value that many flats. For example C minor -> -3 (3 flats), B minor -> 2
    /// (2 sharps).
    pub fn accidentals(&self) -> i8 {
        let tonic_fifths = fifths_at_step(&self.step);

        let fifths = tonic_fifths
            - match self.mode {
                Mode::Major => 0,
                Mode::Minor => 3,
            };

        fifths.clamp(i32::from(i8::MIN), i32::from(i8::MAX)) as i8
    }
}

impl Default for Key {
    fn default() -> Key {
        Key::C_MAJOR
    }
}

/// Position of a natural note letter on the circle of fifths, counting C as 0:
/// F is one flat-ward step (-1), G one sharp-ward (1), and so on.
///
/// A `match` rather than arithmetic for the same reason
/// [`Clef::sharp_lines`](crate::score::core::clef::Clef::sharp_lines) is one:
/// the sequence is a permutation of the letters, not a progression, and spelling
/// it out is what makes it checkable.
fn letter_fifths(steps_from_c: i32) -> i32 {
    match steps_from_c.rem_euclid(7) {
        0 => 0,  // C
        1 => 2,  // D
        2 => 4,  // E
        3 => -1, // F
        4 => 1,  // G
        5 => 3,  // A
        _ => 5,  // B
    }
}

/// The `steps_from_c` of the `n`th letter in flat-to-sharp order, F C G D A E B.
/// Only the seven natural letters are reachable; `alter` carries the rest.
fn nth_letter(n: i32) -> i32 {
    match n.rem_euclid(7) {
        0 => 3, // F
        1 => 0, // C
        2 => 4, // G
        3 => 1, // D
        4 => 5, // A
        5 => 2, // E
        _ => 6, // B
    }
}

/// The circle-of-fifths position of a spelled note. Each sharp adds seven steps
/// (F is -1, F sharp is 6), each flat subtracts seven.
fn fifths_at_step(step: &Step) -> i32 {
    letter_fifths(step.steps_from_c) + 7 * step.alter
}

/// The note spelling `fifths` steps around the circle from C, the inverse of
/// [`fifths_at_step`]. The `+ 1` puts F at index 0, so the seven naturals occupy
/// -1..=5 and everything outside that picks up a sharp or a flat.
fn step_at_fifths(fifths: i32) -> Step {
    Step {
        steps_from_c: nth_letter(fifths + 1),
        alter: (fifths + 1).div_euclid(7),
    }
}
