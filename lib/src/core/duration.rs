#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
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

impl BaseDuration {
    /// Infers the number of beams or flags for a standalone, non-beamed note.
    /// Notes larger than or equal to a quarter note have 0 beams/flags.
    pub fn beam_count(&self) -> i8 {
        let val = *self as i8;
        if val > 2 { val - 2 } else { 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub base: BaseDuration,
    pub dots: u8, // 0 = normal, 1 = dotted, 2 = double-dotted
}

impl Duration {
    pub fn new(base: BaseDuration, dots: u8) -> Self {
        Self { base, dots }
    }

    /// Returns the duration as a raw floating-point multiplier of a whole note.
    /// e.g., Quarter = 0.25, Dotted Quarter = 0.375
    pub fn as_float(&self) -> f32 {
        let exponent = self.base as i8;
        // Base value: 2^(-exponent)
        let base_val = 2.0f32.powi(-exponent as i32);

        // Apply dots: each dot adds half of the previous value
        let mut total = base_val;
        let mut current_dot_val = base_val;
        for _ in 0..self.dots {
            current_dot_val *= 0.5;
            total += current_dot_val;
        }
        total
    }

    /// Delegate beam count to the base duration (dots don't alter beam counts)
    pub fn beam_count(&self) -> i8 {
        self.base.beam_count()
    }
}
