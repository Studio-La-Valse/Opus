use crate::geometry::color::Color;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::score_defaults::PageMargins;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system::System;
use crate::score::visual::system_measure::SystemMeasure;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Page {
    pub systems: BTreeMap<u32, System>,

    pub number: u32,

    pub width: f32,
    pub height: f32,

    pub margins: PageMargins,
    pub color: Color,
    pub foreground: Color,
}

impl Page {
    /// The system under `system_id`, creating it if it isn't there yet. The
    /// created system is stamped with its own index, which
    /// [`Score::locate_or_create_part`](crate::score::visual::score::Score::locate_or_create_part)
    /// needs to decide whether names abbreviate.
    pub fn system_or_insert(&mut self, system_id: u32) -> &mut System {
        self.systems.entry(system_id).or_insert_with(|| System {
            index: system_id,
            ..Default::default()
        })
    }

    pub fn locate_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_part_mut(part_id))
    }

    pub fn locate_system_measure_mut(&mut self, measure_number: u32) -> Option<&mut SystemMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_system_measure_mut(measure_number))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_idx: &StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_staff_measure_mut(part_id, *staff_idx, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| {
                if system.locate_system_measure_mut(measure_number).is_some() {
                    Some(system.locate_staff_measures_mut(part_id, measure_number))
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for system in self.systems.values_mut() {
            system.rebeam(strategy);
        }
    }
}

impl Page {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        let page = score_defaults.resolve_page(self.number);
        self.margins = page.margins;
        self.width = page.width;
        self.height = page.height;
        self.color = user_layout.page_color.unwrap_or(app_defaults.page_color);
        self.foreground = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        for system in self.systems.values_mut() {
            system.resolve_layout(params);
        }
    }
}
