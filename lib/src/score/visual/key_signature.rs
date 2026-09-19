use crate::geometry::xy::XY;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::staff::Staff;

/// Gap between two neighbouring accidentals of a signature, in tenths.
const ACCIDENTAL_SPACING: f32 = 3.;

pub struct KeySignature {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    /// The scale of the staff this signature is written on, mirrored here the
    /// way a clef's and a time signature's are. A signature is drawn in the same
    /// staff spaces as the notes around it, so everything about it scales: the
    /// accidental glyphs, the gaps between them, and the half-spaces that put
    /// each one on its line.
    pub scale: f32,

    pub accidentals: Vec<(i32, Accidental)>,
}

impl Default for KeySignature {
    fn default() -> Self {
        Self {
            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            scale: 1.,

            accidentals: Default::default(),
        }
    }
}

impl KeySignature {
    /// Sets the scale this signature and its accidentals are drawn at. The
    /// counterpart of [`Clef::rescale`](crate::score::visual::clef::Clef::rescale),
    /// called from the same place: the staff measure's `measure`, before
    /// anything is sized against the result.
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;

        for (_, accidental) in self.accidentals.iter_mut() {
            accidental.rescale(scale);
        }
    }

    pub fn line_space(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }

    pub fn accidental_spacing(&self) -> f32 {
        ACCIDENTAL_SPACING * self.scale
    }
}

impl KeySignature {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for (_, accidental) in self.accidentals.iter_mut() {
            accidental.resolve_layout(params);
        }
    }
}
