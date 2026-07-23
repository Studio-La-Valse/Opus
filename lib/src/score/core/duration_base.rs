use crate::visual::stem::UpDown;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BaseDuration {
    Maxima = -3,      // 8 whole notes
    Longa = -2,       // 4 whole notes
    Breve = -1,       // 2 whole notes
    Whole = 0,        // 1
    Half = 1,         // 1/2
    Quarter = 2,      // 1/4
    Eighth = 3,       // 1/8  -> 1 beam/flag
    Sixteenth = 4,    // 1/16 -> 2 beams/flags
    ThirtySecond = 5, // 1/32 -> 3 beams/flags
    SixtyFourth = 6,  // 1/64 -> 4 beams/flags
}

impl Display for BaseDuration {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl BaseDuration {
    /// Infers the number of beams or flags for a standalone, non-beamed note.
    /// Notes larger than or equal to a quarter note have 0 beams/flags.
    pub fn beam_count(&self) -> i8 {
        let val = *self as i8;
        if val > 2 { val - 2 } else { 0 }
    }

    /// Returns the duration as a raw floating-point multiplier of a whole note.
    /// e.g., Quarter = 0.25, Breve = 2.0
    pub fn as_float(&self) -> f32 {
        let exponent = *self as i8;
        // Base value: 2^(-exponent)

        2.0f32.powi(-exponent as i32)
    }

    pub fn as_int(&self) -> i32 {
        match self {
            Self::Whole => 1,
            Self::Half => 2,
            Self::Quarter => 4,
            Self::Eighth => 8,
            Self::Sixteenth => 16,
            Self::ThirtySecond => 32,
            Self::SixtyFourth => 64,
            _ => panic!("Cannot be converted to int: {}", self),
        }
    }

    pub fn notehead_glyph(&self) -> &'static str {
        match self {
            Self::Maxima => "mensuralNoteheadMaximaBlack",
            Self::Longa => "mensuralNoteheadLongaWhite",
            Self::Breve => "noteheadDoubleWhole",
            Self::Whole => "noteheadWhole",
            Self::Half => "noteheadHalf",
            _ => "noteheadBlack",
        }
    }

    pub fn rest_glyph(&self) -> &'static str {
        match self {
            Self::Maxima => "restMaxima",
            Self::Longa => "restLonga",
            Self::Breve => "restDoubleWhole",
            Self::Whole => "restWhole",
            Self::Half => "restHalf",
            Self::Quarter => "restQuarter",
            Self::Eighth => "rest8th",
            Self::Sixteenth => "rest16th",
            Self::ThirtySecond => "rest32nd",
            Self::SixtyFourth => "rest64th",
        }
    }

    pub fn flag_glyph(&self, dir: &UpDown) -> Option<&'static str> {
        match (self, dir) {
            (Self::Eighth, UpDown::Up) => Some("flag8thUp"),
            (Self::Eighth, UpDown::Down) => Some("flag8thDown"),
            (Self::Sixteenth, UpDown::Up) => Some("flag16thUp"),
            (Self::Sixteenth, UpDown::Down) => Some("flag16thDown"),
            (Self::ThirtySecond, UpDown::Up) => Some("flag32ndUp"),
            (Self::ThirtySecond, UpDown::Down) => Some("flag32ndDown"),
            (Self::SixtyFourth, UpDown::Up) => Some("flag64thUp"),
            (Self::SixtyFourth, UpDown::Down) => Some("flag64thDown"),
            _ => None,
        }
    }
}

// Direct SMuFL & Duration conversion mappings on BaseDuration
impl TryFrom<&str> for BaseDuration {
    type Error = String;

    fn try_from(type_str: &str) -> Result<Self, Self::Error> {
        match type_str {
            "maxima" => Ok(BaseDuration::Maxima),
            "longa" => Ok(BaseDuration::Longa),
            "breve" => Ok(BaseDuration::Breve),
            "whole" => Ok(BaseDuration::Whole),
            "half" => Ok(BaseDuration::Half),
            "quarter" => Ok(BaseDuration::Quarter),
            "eighth" => Ok(BaseDuration::Eighth),
            "16th" => Ok(BaseDuration::Sixteenth),
            "32nd" => Ok(BaseDuration::ThirtySecond),
            "64th" => Ok(BaseDuration::SixtyFourth),
            other => Err(format!("Unknown duration type: {other}")),
        }
    }
}

impl TryFrom<u8> for BaseDuration {
    type Error = String;

    fn try_from(type_int: u8) -> Result<Self, Self::Error> {
        match type_int {
            1 => Ok(Self::Whole),
            2 => Ok(Self::Half),
            4 => Ok(Self::Quarter),
            8 => Ok(Self::Eighth),
            16 => Ok(Self::Sixteenth),
            32 => Ok(Self::ThirtySecond),
            64 => Ok(Self::SixtyFourth),
            other => Err(format!("Unknown duration type: {other}")),
        }
    }
}

impl From<u32> for BaseDuration {
    fn from(value: u32) -> Self {
        match value {
            1 => Self::Whole,
            2 => Self::Half,
            4 => Self::Quarter,
            8 => Self::Eighth,
            16 => Self::Sixteenth,
            32 => Self::ThirtySecond,
            64 => Self::SixtyFourth,
            other => panic!("Unknown duration type: {other}"),
        }
    }
}
