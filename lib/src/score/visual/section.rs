use crate::app_defaults::AppDefaults;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::layout::Layout;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::section_measure::SectionMeasure;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Section {
    pub part_groups: BTreeMap<u32, PartGroup>,
    pub measures: BTreeMap<u32, SectionMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl Section {
    pub fn first_visible_staff_distance(&self) -> f32 {
        let mut dist = 0.;
        let mut found = false;

        for (_idx, part_group) in self.part_groups.iter() {
            if found {
                break;
            }

            dist = part_group.first_visible_staff_distance();
            found = true;
        }

        dist
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for part_group in self.part_groups.values_mut() {
            part_group.rebeam(strategy);
        }
    }
}

impl ScoreElement for Section {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.part_groups.iter_mut() {
            result.push(staff);
        }

        for (_idx, measure) in self.measures.iter_mut() {
            result.push(measure);
        }

        result
    }

    fn _apply_layout(
        &mut self,
        _layout: &Layout,
        _user_layout: &UserLayout,
        _app_defaults: &AppDefaults,
    ) {
    }
}

impl Layoutable for Section {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        for (_idx, pg) in self.part_groups.iter_mut() {
            let available = XY::INFINITE;
            pg.measure(&available);
            self.height += pg.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();

        for (_idx, measure) in self.measures.iter_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height - first_visible_staff_distance,
            };
            measure.measure(&available);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let mut _origin = self.xy.mv(0., first_visible_staff_distance);

        for (_idx, measure) in self.measures.iter_mut() {
            measure.arrange(&_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for (_idx, part_group) in self.part_groups.iter_mut() {
            part_group.arrange(&_origin);
            _origin = _origin.mv(0., part_group.height);
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
