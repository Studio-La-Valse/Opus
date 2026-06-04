use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::section_measure::SectionMeasure;
use std::collections::HashMap;

#[derive(Default)]
pub struct Section {
    pub part_groups: HashMap<u32, PartGroup>,
    pub measures: HashMap<u32, SectionMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl Section {}

impl Layoutable for Section {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        for (_idx, pg) in self.part_groups.iter_mut() {
            let available = XY::INFINITE;
            pg.measure(&available);
            self.height += pg.height;
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
        for (_idx, part_group) in self.part_groups.iter_mut() {
            part_group.arrange(&_origin);
            _origin.mv(0., part_group.height);
        }
    }
}

impl Content for Section {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Content));

        result.extend(self.part_groups.values().map(|s| s as &dyn Content));

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
