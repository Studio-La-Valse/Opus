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
}

impl Clef {
    pub fn anchor_line(&self) -> i32 {
        match self {
            Self::Treble => 6,
            Self::Soprano => 8,
            Self::MezzoSoprano => 6,
            Self::Alto => 4,
            Self::Tenor => 2,
            Self::Baritone => 0,
            Self::Bass => 2,
            Self::Percussion => 4,
        }
    }

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
            _ => None,
        }
    }

    pub fn line_index_at_pitch(&self, pitch: &Pitch) -> i32 {
        self.line_middle_c() + (3 - pitch.octave) * 7 + (7 - pitch.step.steps_from_c)
    }
}
