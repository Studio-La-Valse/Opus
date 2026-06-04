use crate::core::xy::XY;
use crate::score::drawable::content::Content;
use crate::score::drawable::element::Element;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::section::Section;
use crate::score::visual::system_measure::SystemMeasure;
use std::collections::HashMap;

#[derive(Default)]
pub struct System {
    pub sections: HashMap<u32, Section>,
    pub measures: HashMap<u32, SystemMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub m_left: f32,
    pub m_right: f32,
    pub distance: f32,
    pub top: f32,
}

impl System {
    fn consolidate_measure_widths(&mut self) {
        for (idx, measure) in self.measures.iter_mut() {
            let width = measure.width;

            for (_, section) in self.sections.iter_mut() {
                let measure = section.measures.get_mut(idx).unwrap();
                measure.width = width;

                for (_, part_group) in section.part_groups.iter_mut() {
                    let measure = part_group.measures.get_mut(idx).unwrap();
                    measure.width = width;

                    for (_, part) in part_group.parts.iter_mut() {
                        let measure = part.measures.get_mut(idx).unwrap();
                        measure.width = width;

                        for (_, staff) in part.staves.iter_mut() {
                            let measure = staff.measures.get_mut(idx).unwrap();
                            measure.width = width;
                        }
                    }
                }
            }
        }
    }
}

impl Layoutable for System {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        self.consolidate_measure_widths();

        for (_idx, section) in self.sections.iter_mut() {
            let available = XY::INFINITE;
            section.measure(&available);
            self.height += section.height;
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
        for (_idx, section) in self.sections.iter_mut() {
            section.arrange(&_origin);
            _origin.mv(0., section.height);
        }
    }
}

impl Content for System {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Content));

        result.extend(self.sections.values().map(|s| s as &dyn Content));

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
