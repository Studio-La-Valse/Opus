use crate::geometry::xy::XY;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::staff::Staff;

const ACCIDENTAL_SPACING: f32 = 3.;

#[derive(Default)]
pub struct KeySignature {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub accidentals: Vec<(i32, Accidental)>,
}

impl Layoutable for KeySignature {
    fn measure(&mut self, _available: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        for (_, accidental) in self.accidentals.iter_mut() {
            accidental.measure(_available, params);
            self.width += accidental.width;
        }

        if self.accidentals.len() > 1 {
            self.width += (self.accidentals.len() - 1) as f32 * ACCIDENTAL_SPACING;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut x = self.xy.x;
        for (line, acc) in self.accidentals.iter_mut() {
            let dy = Staff::DEFAULT_SPACE_SIZE / 2. * (*line as f32);
            let y = self.xy.y + dy;
            let xy = XY { x, y }.mv(acc.width, 0.);
            acc.arrange(&xy);

            x += acc.width + ACCIDENTAL_SPACING;
        }
    }
}
