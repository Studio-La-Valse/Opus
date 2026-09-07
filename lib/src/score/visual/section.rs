use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::brace::Brace;
use crate::score::visual::bracket::Bracket;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::part::Part;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::section_measure::SectionMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
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

    /// Every staff of this section that is drawn, top to bottom.
    pub fn visible_staves(&self) -> impl Iterator<Item = &Staff> {
        self.part_groups
            .values()
            .flat_map(|group| group.visible_staves())
    }

    pub fn shows_bracket(&self) -> bool {
        self.part_groups.len() > 1 && self.visible_staves().next().is_some()
    }

    /// How far the barline at a measure end reaches above the top line of the
    /// section's first visible staff, and below the bottom line of its last.
    /// Both are zero for the staves that enclose spaces of their own; see
    /// [`Staff::barline_overhang`].
    fn barline_overhang(&self) -> (f32, f32) {
        let top = self
            .visible_staves()
            .next()
            .map(Staff::barline_overhang)
            .unwrap_or(0.);

        let bottom = self
            .visible_staves()
            .last()
            .map(Staff::barline_overhang)
            .unwrap_or(0.);

        (top, bottom)
    }
}
impl Layoutable for Section {
    fn measure(&mut self, _: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        self.height = 0.;

        for pg in self.part_groups.values_mut() {
            let available = XY::INFINITE;
            pg.measure(&available, params);
            self.height += pg.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        // The barline a measure draws at its end is taller than the staves it
        // crosses when either outermost one is a single line, which is a staff
        // of no height at all.
        let (overhang_top, overhang_bottom) = self.barline_overhang();
        let barline_height = staves_height + overhang_top + overhang_bottom;

        for measure in self.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: barline_height,
            };
            measure.measure(&available, params);
            self.width += measure.width;
        }

        if self.shows_bracket() {
            let avail = XY {
                x: self.width,
                y: staves_height,
            };
            self.bracket.measure(&avail, params);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let (overhang_top, _) = self.barline_overhang();
        let mut measure_origin = self.xy.mv(0., first_visible_staff_distance - overhang_top);

        for measure in self.measures.values_mut() {
            measure.arrange(&measure_origin);
            measure_origin = measure_origin.mv(measure.width, 0.);
        }

        let mut part_group_origin = self.xy;
        for part_group in self.part_groups.values_mut() {
            part_group.arrange(&part_group_origin);
            part_group_origin = part_group_origin.mv(0., part_group.height);
        }

        let bracket_origin = self.xy.mv(-10., first_visible_staff_distance);
        self.bracket.arrange(&bracket_origin);
    }
}
