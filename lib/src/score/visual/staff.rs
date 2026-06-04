use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::staff_measure::StaffMeasure;
use std::collections::HashMap;

#[derive(Default)]
pub struct Staff {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub measures: HashMap<u32, StaffMeasure>,

    pub hidden: bool,
    pub distance_specified: Option<f32>,
    pub distance_final: f32,
}

impl Staff {
    // 5 lines, 4 spaces, 10 tenths for each space according to MusicXML spec.
    pub const SIZE: i8 = 40;
}

impl Layoutable for Staff {
    fn measure(&mut self, available: &XY) {
        self.height = f32::from(Self::SIZE);
        if self.hidden {
            self.height = 0.;
        }
        self.width = available.x;

        for (_idx, measure) in self.measures.iter_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(available);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for Staff {
    fn content(&self) -> Vec<&dyn Content> {
        self.measures.values().map(|m| m as &dyn Content).collect()
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
