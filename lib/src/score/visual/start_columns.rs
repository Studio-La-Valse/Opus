//! Where the elements that open a measure -- the clef, the key signature and
//! the time signature -- are drawn, as one set of columns shared by every staff
//! of a system.
//!
//! Left to itself a staff would place these one after the other from its own
//! left edge, and a staff whose key signature is narrower than its neighbour's
//! would then start its time signature further left than theirs. That happens in
//! any score with transposing instruments: a score in C flat major carries seven
//! flats, the trumpets in B flat five, and an unpitched percussion staff none at
//! all, so three different time-signature positions down the same system.
//!
//! The columns fix that. Each of the three is decided once for the whole
//! system, from the widest contribution any staff makes to it, and every staff
//! then draws against the same three offsets. Only the *start* of each column
//! is shared: what a staff draws there is still its own, so the narrower key
//! signature simply leaves more air before the next column.

use crate::score::visual::staff_measure::StaffMeasure;

/// Where one measure's opening clef, key signature and time signature are
/// drawn, as x offsets from the measure's own left edge.
///
/// Offsets rather than absolute positions because every staff measure with the
/// same number shares a left edge -- see
/// [`System::consolidate_measure_width`](crate::score::visual::system::System) --
/// so the same three offsets align the elements down the whole system.
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct StartColumns {
    pub clef: f32,
    pub key: f32,
    pub time: f32,
}

impl StartColumns {
    /// Folds what every staff of a measure needs into the three columns they
    /// all draw against.
    ///
    /// Resolved outwards, one column at a time, because each column's position
    /// depends on the one before it: the key signatures cannot be placed until
    /// the widest clef is known, and the time signatures not until the widest
    /// key signature has been measured *from that shared clef column*. Taking
    /// the maximum of what each staff would have done on its own would be a
    /// different, and wrong, answer -- a staff whose clef is narrow computes its
    /// key column close in, and that answer does not survive the clef column
    /// moving right.
    pub fn fold(metrics: &[StartMetrics]) -> StartColumns {
        let clef = metrics.iter().map(|m| m.padding).fold(0., f32::max);
        let key = metrics
            .iter()
            .map(|m| m.key_column(clef))
            .fold(0., f32::max);
        let time = metrics
            .iter()
            .map(|m| m.time_column(key))
            .fold(0., f32::max);

        StartColumns { clef, key, time }
    }
}

/// What one staff measure contributes to its system's [`StartColumns`]: the
/// widths that have to be cleared, and the padding this staff wants after each
/// of them.
///
/// Padding is per staff because it scales with the staff -- a cue-sized staff
/// asks for a smaller gap than a full-sized one -- and the column has to satisfy
/// the most demanding staff.
#[derive(Clone, Copy, Debug)]
pub struct StartMetrics {
    /// Gap this staff wants between one opening element and the next.
    pub padding: f32,
    /// Width of the opening clef, or `None` when this measure opens without one.
    pub clef_width: Option<f32>,
    /// Width of the opening key signature: zero for a staff that carries no
    /// accidentals, such as an unpitched percussion staff.
    pub key_width: f32,
}

impl StartMetrics {
    /// Everything a staff measure's opening elements need to be placed.
    pub fn of(measure: &StaffMeasure) -> StartMetrics {
        StartMetrics {
            padding: measure.padding(),
            clef_width: measure.clef_start.as_ref().map(|clef| clef.width),
            key_width: measure.key_signature_start.width,
        }
    }

    /// Where this staff would start its key signature, given the shared clef
    /// column: clear of its own clef, or a padding in from the measure's left
    /// edge when it has no clef to clear.
    fn key_column(&self, clef_column: f32) -> f32 {
        match self.clef_width {
            Some(width) => clef_column + width + self.padding,
            None => self.padding,
        }
    }

    /// Where this staff would start its time signature, given the shared key
    /// column.
    fn time_column(&self, key_column: f32) -> f32 {
        key_column + self.key_width + self.padding
    }
}
