use crate::score::core::duration_base::BaseDuration;

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
