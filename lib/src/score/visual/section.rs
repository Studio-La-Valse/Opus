use crate::core::xy::XY;
use crate::drawable::layoutable::Layoutable;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::section_measure::SectionMeasure;
use crate::visual::brace::Brace;
use crate::visual::bracket::Bracket;
use crate::visual::part::Part;
use crate::visual::part_measure::PartMeasure;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff_measure::StaffMeasure;
use std::collections::BTreeMap;

pub struct Section {
    pub part_groups: BTreeMap<u32, PartGroup>,
    pub measures: BTreeMap<u32, SectionMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub bracket: Bracket,
}

impl Section {
    pub fn new(bracket: Bracket) -> Section {
        Section {
            bracket,

            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            part_groups: Default::default(),
            measures: Default::default(),
        }
    }

    pub fn part_group_or_insert<F: FnOnce() -> Brace>(
        &mut self,
        part_group_id: u32,
        factory: F,
    ) -> &mut PartGroup {
        self.part_groups.entry(part_group_id).or_insert_with(|| {
            let brace = factory();
            PartGroup::new(brace)
        })
    }

    pub fn locate_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.part_groups
            .values_mut()
            .find_map(|pg| pg.locate_part_mut(part_id))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.part_groups
            .values_mut()
            .find_map(|group| group.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        self.locate_part_mut(part_id)
            .map(|p| p.locate_staff_measures_mut(measure_number))
            .unwrap_or_default()
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_number: StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.part_groups
            .values_mut()
            .find_map(|group| group.locate_staff_measure_mut(part_id, staff_number, measure_number))
    }

    fn first_visible_staff_distance(&self) -> f32 {
        self.part_groups
            .values()
            .next()
            .map(|group| group.first_visible_staff_distance())
            .unwrap_or(0.0)
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for part_group in self.part_groups.values_mut() {
            part_group.rebeam(strategy);
        }
    }

    pub fn visible_staves(&self) -> usize {
        let mut count = 0;
        for part_group in self.part_groups.values() {
            count += part_group.visible_staves();
        }

        count
    }

    pub fn shows_bracket(&self) -> bool {
        self.part_groups.len() > 1 && self.visible_staves() > 0
    }
}

impl ScoreElement for Section {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let shows_bracket = self.shows_bracket();

        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for staff in self.part_groups.values_mut() {
            result.push(staff);
        }

        for measure in self.measures.values_mut() {
            result.push(measure);
        }

        if shows_bracket {
            let bracket: &mut Bracket = &mut self.bracket;
            result.push(bracket);
        }

        result
    }
}

impl Layoutable for Section {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        for pg in self.part_groups.values_mut() {
            let available = XY::INFINITE;
            pg.measure(&available);
            self.height += pg.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        for measure in self.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: staves_height,
            };
            measure.measure(&available);
            self.width += measure.width;
        }

        if self.shows_bracket() {
            let avail = XY {
                x: self.width,
                y: staves_height,
            };
            self.bracket.measure(&avail);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let mut _origin = self.xy.mv(0., first_visible_staff_distance);

        for measure in self.measures.values_mut() {
            measure.arrange(&_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for part_group in self.part_groups.values_mut() {
            part_group.arrange(&_origin);
            _origin = _origin.mv(0., part_group.height);
        }

        let _origin = self.xy.mv(-10., first_visible_staff_distance);
        self.bracket.arrange(&_origin);
    }
}
