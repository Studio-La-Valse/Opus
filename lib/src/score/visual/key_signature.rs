use crate::geometry::xy::XY;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;

#[derive(Default)]
pub struct KeySignature {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub accidentals: Vec<(i32, Accidental)>,
}

impl KeySignature {}

impl ScoreElement for KeySignature {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut children: Vec<&mut dyn ScoreElement> = Vec::new();
        for (_, acc) in self.accidentals.iter_mut() {
            children.push(acc);
        }
        children
    }
}

impl Layoutable for KeySignature {
    fn measure(&mut self, _available: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        for (_, accidental) in self.accidentals.iter_mut() {
            accidental.measure(_available, params);
            self.width += accidental.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut x = self.xy.x;
        for (line, acc) in self.accidentals.iter_mut() {
            let dy = Staff::DEFAULT_SPACE_SIZE / 2. * (*line as f32);
            let y = self.xy.y + dy;
            let xy = XY { x, y };
            acc.arrange(&xy);

            x += acc.width + 2.
        }
    }
}
