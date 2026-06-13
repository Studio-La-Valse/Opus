use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part::Part;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use std::collections::BTreeMap;
use crate::layout_ctx::Visibility;

#[derive(Default)]
pub struct PartGroup {
    pub parts: BTreeMap<String, Part>,
    pub measures: BTreeMap<u32, PartGroupMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl PartGroup {
    pub fn first_visible_staff_distance(&self) -> f32 {
        let mut dist = 0.;
        let mut found = false;

        for (_idx, part) in self.parts.iter() {
            if found {
                break;
            }

            if part.visibility == Visibility::Hidden {
                continue;
            }

            dist = part.first_visible_staff_distance();
            found = true;
        }

        dist
    }
}

impl Layoutable for PartGroup {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        for (_idx, part) in self.parts.iter_mut() {
            let available = XY::INFINITE;
            part.measure(&available);
            self.height += part.height;
        }

        for (_idx, measure) in self.measures.iter_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(&available);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut _origin = self.xy;
        for (_idx, measure) in self.measures.iter_mut() {
            measure.arrange(origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for (_idx, part) in self.parts.iter_mut() {
            part.arrange(&_origin);
            _origin = _origin.mv(0., part.height);
        }
    }
}

impl Content for PartGroup {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Content));

        result.extend(self.parts.values().map(|s| s as &dyn Content));

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
