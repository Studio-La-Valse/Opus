use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
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

    /// Whether this section draws its symbol: it has to bind more than one
    /// part-group for there to be anything to bind, and the symbol itself has to
    /// be one that draws.
    pub fn shows_symbol(&self) -> bool {
        self.part_groups.len() > 1 && self.symbol.is_drawn()
    }

    /// How far left this section's own ink reaches, which is what the symbols
    /// inside it keep clear of. The system's left edge when nothing is drawn --
    /// an undrawn symbol must not push its part-groups outward by a gap that
    /// nothing occupies.
    fn symbol_left_edge(&self) -> f32 {
        if self.shows_symbol() {
            self.symbol.bounds().x_min()
        } else {
            self.xy.x
        }
    }

    /// Places this section, additionally carrying the page's left margin
    /// (`margin_left`) down the chain so a part / part-group name knows how far
    /// left its box reaches. A section has no name of its own -- it is the
    /// (usually unnamed) bracket around a run of part-groups.
    ///
    /// [`Layoutable::arrange`] delegates here with `origin.x` for the margin,
    /// the way each level's `arrange` delegates today.
    pub fn arrange_within(&mut self, origin: &XY, margin_left: f32) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let (overhang_top, _) = self.barline_overhang();
        let mut measure_origin = self.xy.mv(0., first_visible_staff_distance - overhang_top);

        for measure in self.measures.values_mut() {
            measure.arrange(&measure_origin);
            measure_origin = measure_origin.mv(measure.width, 0.);
        }

        // Arranged before the part-groups, because where they put their own
        // symbols depends on how far left this one reached. A section's symbol
        // is the innermost of the three, so it is the only one measured from
        // the system itself.
        self.symbol
            .arrange(&self.xy.mv(0., first_visible_staff_distance));
        let clear_of = self.symbol_left_edge();

        let mut part_group_origin = self.xy;
        for part_group in self.part_groups.values_mut() {
            part_group.arrange_clear_of(&part_group_origin, clear_of, margin_left);
            part_group_origin = part_group_origin.mv(0., part_group.height);
        }
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
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for pg in self.part_groups.values_mut() {
            pg.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }

        self.symbol.resolve_layout(params);
    }

    fn measure(&mut self, _: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        self.height = 0.;

        for pg in self.part_groups.values_mut() {
            let available = XY::INFINITE;
            pg.measure(&available, params);
            self.height += pg.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        // Top line of the first staff to bottom line of the last: `self.height`
        // is the room the staves reserve, so the padding a short last staff
        // keeps below its line has to come back off.
        let trailing_padding = self
            .visible_staves()
            .last()
            .map(Staff::floor_padding)
            .unwrap_or(0.);
        let staves_height = self.height - first_visible_staff_distance - trailing_padding;

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

        // Measured whatever it turns out to draw: a symbol that has been sized
        // and placed on every pass cannot go stale, and `shows_symbol` is then
        // a question about the current pass rather than about which branch last
        // ran. Only the compositor decides whether to draw it.
        let avail = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.symbol.measure(&avail, params);
    }

    fn arrange(&mut self, origin: &XY) {
        self.arrange_within(origin, origin.x);
    }
}
