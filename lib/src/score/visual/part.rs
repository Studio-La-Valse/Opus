use crate::geometry::xy::XY;
use crate::score::core::clef::Clef as CoreClef;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::clef::Clef as DrawableClef;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::walk_cursor::Visibility;
use std::collections::{BTreeMap, HashSet};

pub struct Part {
    pub measures: BTreeMap<u32, PartMeasure>,
    pub staves: BTreeMap<StaffIdx, Staff>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub visibility: Visibility,

    pub symbol: GroupSymbol,
    /// The `<part-name>` drawn to the left of the symbol -- or where the symbol
    /// would have been, for the common single-staff part that draws none.
    pub name: GroupName,
}

impl Part {
    pub fn new(symbol: GroupSymbol, name: GroupName) -> Self {
        Self {
            measures: BTreeMap::new(),
            staves: BTreeMap::new(),

            xy: XY::ZERO,
            width: 0.0,
            height: 0.0,

            visibility: Visibility::Unset,

            symbol,
            name,
        }
    }

    pub fn staff_or_insert(&mut self, idx: &StaffIdx) -> &mut Staff {
        self.staves.entry(*idx).or_default()
    }

    pub fn locate_part_measure_mut(&mut self, measure_number: u32) -> Option<&mut PartMeasure> {
        self.measures.get_mut(&measure_number)
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        staff_number: StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.staves
            .get_mut(&staff_number)
            .and_then(|staff| staff.measures.get_mut(&measure_number))
    }

    pub fn locate_staff_measures_mut(&mut self, measure_number: u32) -> Vec<&mut StaffMeasure> {
        self.staves
            .values_mut()
            .filter_map(|staff| staff.locate_measure_mut(&measure_number))
            .collect()
    }

    pub fn set_visibility(&mut self, visibility: Visibility) {
        match visibility {
            Visibility::Unset => {
                // Auto means: do nothing
            }

            Visibility::Hidden | Visibility::Shown => {
                match self.visibility {
                    Visibility::Shown => {
                        // previously explicitly shown, not allowed to change.
                    }

                    Visibility::Unset | Visibility::Hidden => {
                        // first time set, allowed to set visibility, OR:
                        // hidden: allowed to hide or show.
                        self.visibility = visibility;
                    }
                }
            }
        }
    }

    pub fn ensure_staves(&mut self, staves: &HashSet<StaffIdx>) {
        for staff in staves {
            self.staves.entry(*staff).or_default();
        }
    }

    pub fn hide_staves(&mut self, staves: &HashSet<StaffIdx>) {
        for &staff in staves {
            let staff = self.staves.entry(staff).or_default();
            staff.hidden = true;
        }
    }

    pub fn show_staves(&mut self, staves: &HashSet<StaffIdx>) {
        for &staff in staves {
            let staff = self.staves.entry(staff).or_default();
            staff.hidden = false;
        }
    }

    pub fn set_distances(&mut self, distances: &BTreeMap<StaffIdx, f32>, default: &f32) {
        for (&idx, &dist) in distances.iter() {
            let staff = self.staves.entry(idx).or_default();
            staff.distance_specified = Some(dist);
        }

        for (&_idx, staff) in self.staves.iter_mut() {
            let distance = staff.distance_specified.unwrap_or(*default);
            staff.distance_final = distance;
        }
    }

    /// `f` is handed the staff's line count alongside the clef, because a clef
    /// that names no pitch is centred on the staff rather than fixed to a line
    /// of it. The staves already carry the count by this point: it is settled on
    /// the layout walk, and opening clefs are set on the content one.
    pub fn set_opening_clef<F: Fn(CoreClef, usize) -> DrawableClef>(
        &mut self,
        clefs: &BTreeMap<StaffIdx, CoreClef>,
        f: F,
    ) {
        for (idx, clef) in clefs {
            let staff = self.staves.entry(*idx).or_default();
            let drawable = f(*clef, staff.lines);
            let measure = staff.measures.values_mut().next().unwrap();
            measure.clef_start = Some(drawable);
        }
    }

    pub fn set_staff_lines(&mut self, staff_lines: &BTreeMap<StaffIdx, usize>) {
        for (&idx, &lines) in staff_lines.iter() {
            let staff = self.staves.entry(idx).or_default();
            staff.set_lines(lines);
        }
    }

    pub fn set_staff_scale(&mut self, staff_scales: &BTreeMap<StaffIdx, f32>) {
        for (&idx, staff_scale) in staff_scales.iter() {
            let staff = self.staves.entry(idx).or_default();
            staff.set_scale(*staff_scale)
        }
    }

    pub fn first_visible_staff_distance(&self) -> f32 {
        self.staves
            .values()
            .find(|staff| !staff.hidden)
            .map(|staff| staff.distance_final)
            .unwrap_or(0.0)
    }

    pub fn staff_measures_mut(&mut self, measure_number: &u32) -> Vec<&mut StaffMeasure> {
        // First check if this part even contains this measure
        if !self.measures.contains_key(measure_number) {
            return vec![];
        }

        self.staves
            .values_mut()
            .map(|staff| staff.measures.get_mut(measure_number).unwrap())
            .collect()
    }

    /// Where each staff sits relative to the top of the part, for the elements
    /// that are positioned against a staff without belonging to it -- notes and
    /// rests, which hang off a `PartMeasure` and only name their staff by index.
    ///
    /// Walks the staves the same way [`Part::arrange`] does, hidden ones
    /// included in the map but contributing no distance of their own. If they
    /// contributed here but not there, every staff below a hidden one would have
    /// its notes pushed down past its own staff lines.
    pub fn create_staff_ctx(&self) -> BTreeMap<StaffIdx, StaffCtx> {
        let mut res = BTreeMap::new();

        let mut distance_travelled = 0.;
        for (idx, staff) in self.staves.iter() {
            if !staff.hidden {
                distance_travelled += staff.distance_final;
            }

            let meta = StaffCtx {
                hidden: staff.hidden,
                scaling: staff.scale,
                distance_from_top: distance_travelled,
                lines: staff.lines,
            };
            res.insert(*idx, meta);

            if !staff.hidden {
                distance_travelled += staff.height;
            }
        }

        res
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for measure in self.measures.values_mut() {
            measure.rebeam(strategy);
        }
    }

    /// The staves of this part that are drawn, top to bottom.
    pub fn visible_staves(&self) -> impl Iterator<Item = &Staff> {
        self.staves.values().filter(|staff| !staff.hidden)
    }

    /// Whether this part draws its own symbol: more than one staff to join, and
    /// a symbol that draws.
    pub fn shows_symbol(&self) -> bool {
        self.visible_staves().count() > 1 && self.symbol.is_drawn()
    }

    /// Whether this part draws its name: it has at least one staff drawn -- a
    /// single-staff part with no symbol still gets its name, which is the common
    /// case -- and the name has something to draw.
    pub fn shows_name(&self) -> bool {
        self.visible_staves().count() > 0 && self.name.is_drawn()
    }

    /// How far left this part's own ink reaches, which is what its name aligns
    /// against: the symbol's left edge when it draws, and the incoming
    /// `clear_of` when it does not. The counterpart of
    /// [`Section::symbol_left_edge`](crate::score::visual::section::Section).
    fn symbol_left_edge(&self, clear_of: f32) -> f32 {
        if self.shows_symbol() {
            self.symbol.bounds().x_min()
        } else {
            clear_of
        }
    }

    /// Places this part, with `clear_of` the left edge of whatever its
    /// part-group drew and `margin_left` the page's left margin. A part's
    /// symbol is the outermost of the three, so nothing keeps clear of it in
    /// turn; its name ends a padding left of it and reaches back to the margin.
    pub fn arrange_clear_of(&mut self, origin: &XY, clear_of: f32, margin_left: f32) {
        self.xy = *origin;

        let mut _origin = self.xy;
        for staff in self.staves.values_mut() {
            if staff.hidden {
                continue;
            }

            _origin = _origin.mv(0., staff.distance_final);

            staff.arrange(&_origin);
            _origin = _origin.mv(0., staff.height);
        }

        let staff_ctx = self.create_staff_ctx();

        let mut _origin = self.xy;
        for measure in self.measures.values_mut() {
            measure.arrange_ctx(&_origin, &staff_ctx);
            _origin = _origin.mv(measure.width, 0.);
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let symbol_top = XY {
            x: clear_of,
            y: self.xy.y + first_visible_staff_distance,
        };
        self.symbol.arrange(&symbol_top);

        let name_right = self.symbol_left_edge(clear_of);
        self.name
            .arrange_between(symbol_top, name_right, margin_left);
    }
}
impl Layoutable for Part {
    /// Recurses into every child unconditionally, a hidden part included: it
    /// still has a colour and a symbol to resolve, even though `measure` will
    /// leave its size at zero and nothing ends up drawing them.
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for staff in self.staves.values_mut() {
            staff.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }

        self.symbol.resolve_layout(params);
        self.name.resolve_layout(params);
    }

    fn measure(&mut self, _: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        self.height = 0.;

        if self.visibility == Visibility::Hidden {
            return;
        }

        for staff in self.staves.values_mut() {
            let available = &XY::INFINITE;
            // a staff knows its own size (sum of measure widths, staff height)
            staff.measure(available, params);

            if staff.hidden {
                continue;
            }

            self.height += staff.height;
            self.height += staff.distance_final;
        }

        for measure in self.measures.values_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(available, params);
            self.width += measure.width;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        // Sized on every pass whatever it draws; see `Section::measure`. The
        // name is sized with the same staves height the symbol is.
        let available = XY {
            x: f32::INFINITY,
            y: staves_height,
        };
        self.symbol.measure(&available, params);
        self.name.measure(&available);
    }

    /// Places this part as [`arrange_clear_of`](Part::arrange_clear_of) does,
    /// with nothing to its left to keep clear of.
    fn arrange(&mut self, origin: &XY) {
        self.arrange_clear_of(origin, origin.x, origin.x);
    }
}
