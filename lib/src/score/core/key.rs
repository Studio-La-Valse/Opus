#[derive(Debug, Copy, Clone)]
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

#[derive(Debug, Copy, Clone)]
pub struct Key {
    pub fifths: i8,
    pub mode: Mode,
}

impl Key {
    /// A positive value denotes number of sharps, a negative value denotes flats.
    /// For example: c minor -> -3 (3 flats), b minor -> 2 (2 sharps).
    pub fn accidentals(&self) -> i8 {
        match self.mode {
            Mode::Major => self.fifths,
            Mode::Minor => self.fifths - 3,
        }
    }
}

impl TryFrom<(i8, &str)> for Key {
    type Error = String;

    fn try_from(value: (i8, &str)) -> Result<Self, Self::Error> {
        const ERR: &str = "Only value between -6 and 6 accepted";

        let fifths = value.0;
        let mut mode = value.1;

        // musicxml defaults to "none" mode when fifths is 0. Very annoying.
        if fifths == 0 {
            mode = "major";
        }

        let mode: Mode = mode.try_into()?;

        match fifths {
            value if value < -6 => Err(ERR.to_string()),
            value if value > 6 => Err(ERR.to_string()),
            _ => Ok(Key { fifths, mode }),
        }
    }
}
