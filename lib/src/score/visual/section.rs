use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::LayoutParams;
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

    pub symbol: GroupSymbol,
}

impl Section {
    pub fn new(symbol: GroupSymbol) -> Section {
        Section {
            symbol,

            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            part_groups: Default::default(),
            measures: Default::default(),
        }
    }

    pub fn part_group_or_insert(
        &mut self,
        part_group_id: u32,
        symbol: GroupSymbol,
        name: GroupName,
    ) -> &mut PartGroup {
        self.part_groups
            .entry(part_group_id)
            .or_insert_with(|| PartGroup::new(symbol, name))
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

    pub fn first_visible_staff_distance(&self) -> f32 {
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

    /// The mutable twin of [`visible_staves`](Self::visible_staves).
    pub fn visible_staves_mut(&mut self) -> impl Iterator<Item = &mut Staff> {
        self.part_groups
            .values_mut()
            .flat_map(|group| group.visible_staves_mut())
    }

    /// Whether this section draws its symbol: it has to bind more than one
    /// part-group for there to be anything to bind, and the symbol itself has to
    /// be one that draws.
    pub fn shows_symbol(&self) -> bool {
        self.part_groups.len() > 1 && self.symbol.is_drawn()
    }
}
impl Section {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for pg in self.part_groups.values_mut() {
            pg.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }

        self.symbol.resolve_layout(params);
    }
}
