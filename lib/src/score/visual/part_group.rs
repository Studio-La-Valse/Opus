use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::part::Part;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::walk_cursor::Visibility;
use std::collections::BTreeMap;

pub struct PartGroup {
    pub parts: BTreeMap<String, Part>,
    pub measures: BTreeMap<u32, PartGroupMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub symbol: GroupSymbol,
}

impl PartGroup {
    pub fn new(symbol: GroupSymbol) -> Self {
        PartGroup {
            parts: Default::default(),
            measures: Default::default(),

            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            symbol,
        }
    }

    pub fn part_or_insert(&mut self, part_id: String, symbol: GroupSymbol) -> &mut Part {
        self.parts
            .entry(part_id)
            .or_insert_with(|| Part::new(symbol))
    }

    pub fn locate_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.parts.get_mut(part_id)
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        self.parts
            .get_mut(part_id)
            .map(|p| p.locate_staff_measures_mut(measure_number))
            .unwrap_or_default()
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.parts
            .get_mut(part_id)?
            .locate_part_measure_mut(measure_number)
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_number: StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.parts
            .get_mut(part_id)?
            .locate_staff_measure_mut(staff_number, measure_number)
    }

    pub fn first_visible_staff_distance(&self) -> f32 {
        let mut dist = 0.;
        let mut found = false;

        for part in self.parts.values() {
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

    /// Every staff of this group that is drawn, top to bottom: the staves of
    /// its visible parts, each part's hidden staves left out.
    pub fn visible_staves(&self) -> impl Iterator<Item = &Staff> {
        self.parts
            .values()
            .filter(|part| part.visibility != Visibility::Hidden)
            .flat_map(|part| part.visible_staves())
    }

    /// Whether this group draws its symbol: more than one part to bind, more
    /// than one staff actually drawn, and a symbol that draws.
    pub fn shows_symbol(&self) -> bool {
        self.parts.len() > 1 && self.visible_staves().count() > 1 && self.symbol.is_drawn()
    }

    /// Places this group, with `clear_of` the left edge of whatever the
    /// enclosing section drew. This group's symbol sits its own gap further out
    /// than that, and hands its own left edge to its parts in turn -- so the
    /// three levels stack outward from the system without any of them knowing
    /// how wide the others are.
    pub fn arrange_clear_of(&mut self, origin: &XY, clear_of: f32) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();

        let mut _origin = self.xy;
        for measure in self.measures.values_mut() {
            measure.arrange(&_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        // Before the parts, whose own symbols keep clear of this one.
        self.symbol.arrange(&XY {
            x: clear_of,
            y: self.xy.y + first_visible_staff_distance,
        });
        let clear_of = if self.shows_symbol() {
            self.symbol.bounds().x_min()
        } else {
            clear_of
        };

        let mut _origin = self.xy;
        for part in self.parts.values_mut() {
            part.arrange_clear_of(&_origin, clear_of);
            _origin = _origin.mv(0., part.height);
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for part in self.parts.values_mut() {
            part.rebeam(strategy);
        }
    }
}
impl Layoutable for PartGroup {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for part in self.parts.values_mut() {
            part.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }

        self.symbol.resolve_layout(params);
    }

    fn measure(&mut self, _: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        self.height = 0.;

        for part in self.parts.values_mut() {
            let available = XY::INFINITE;
            part.measure(&available, params);
            self.height += part.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        for measure in self.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(&available, params);
            self.width += measure.width;
        }

        // Sized on every pass whatever it draws; see `Section::measure`.
        let available = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.symbol.measure(&available, params);
    }

    /// Places this group as [`arrange`](Layoutable::arrange) does, with its
    /// enclosing section's symbol already at `clear_of`. Used on its own when
    /// there is nothing to keep clear of.
    fn arrange(&mut self, origin: &XY) {
        self.arrange_clear_of(origin, origin.x);
    }
}
