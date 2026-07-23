use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::page::Page;
use crate::visual::part_measure::PartMeasure;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff_measure::StaffMeasure;
use crate::visual::system::System;
use crate::visual::system_measure::SystemMeasure;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Score {
    pub pages: BTreeMap<u32, Page>,
}

impl Score {
    pub fn get_page_or_insert(&mut self, page_number: u32) -> &mut Page {
        self.pages.entry(page_number).or_default()
    }

    pub fn locate_system_mut(&mut self, system_idx: &u32) -> Option<&mut System> {
        self.pages
            .values_mut()
            .find_map(|page| page.systems.get_mut(system_idx))
    }

    pub fn locate_system_measure_mut(&mut self, measure_number: u32) -> Option<&mut SystemMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_system_measure_mut(measure_number))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_idx: &StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_staff_measure_mut(part_id, staff_idx, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| {
                if page.locate_system_measure_mut(measure_number).is_some() {
                    Some(page.locate_staff_measures_mut(part_id, measure_number))
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for page in self.pages.values_mut() {
            page.rebeam(strategy);
        }
    }
}

impl ScoreElement for Score {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, page) in self.pages.iter_mut() {
            result.push(page);
        }

        result
    }
}

impl Layoutable for Score {
    fn measure(&mut self, available: &XY) {
        for (_idx, page) in self.pages.iter_mut() {
            page.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        let mut _origin: XY = *origin;

        for (_idx, page) in self.pages.iter_mut() {
            page.arrange(&_origin);
            _origin = _origin.mv(page.width, 0.);
            _origin = _origin.mv(200., 0.);
        }
    }
}

impl DrawableContent for Score {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        self.pages
            .values()
            .map(|s| s as &dyn DrawableContent)
            .collect()
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
