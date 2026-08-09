use crate::core::xy::XY;
use crate::drawable::layoutable::Layoutable;
use crate::score::core::clef::Clef as CoreClef;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::layout_ctx::Visibility;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff::Staff;
use crate::visual::brace::Brace;
use crate::visual::clef::Clef as DrawableClef;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff_ctx::StaffCtx;
use crate::visual::staff_measure::StaffMeasure;
use std::collections::{BTreeMap, HashSet};

pub struct Part {
    pub measures: BTreeMap<u32, PartMeasure>,
    pub staves: BTreeMap<StaffIdx, Staff>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub visibility: Visibility,

    pub brace: Brace,
}

impl Part {
    pub fn new(brace: Brace) -> Self {
        Self {
            measures: BTreeMap::new(),
            staves: BTreeMap::new(),

            xy: XY::ZERO,
            width: 0.0,
            height: 0.0,

            visibility: Visibility::Unset,

            brace,
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

    pub fn set_opening_clef<F: Fn(CoreClef) -> DrawableClef>(
        &mut self,
        clefs: &BTreeMap<StaffIdx, CoreClef>,
        f: F,
    ) {
        for (idx, clef) in clefs {
            let staff = self.staves.entry(*idx).or_default();
            let drawable = f(*clef);
            let measure = staff.measures.values_mut().next().unwrap();
            measure.clef_start = Some(drawable);
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

    pub fn create_staff_ctx(&self) -> BTreeMap<StaffIdx, StaffCtx> {
        let mut res = BTreeMap::new();

        let mut distance_travelled = 0.;
        for (_idx, staff) in self.staves.iter() {
            distance_travelled += staff.distance_final;

            let meta = StaffCtx {
                hidden: staff.hidden,
                scaling: staff.scale,
                distance_from_top: distance_travelled,
            };
            res.insert(*_idx, meta);

            distance_travelled += staff.height;
        }

        res
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for measure in self.measures.values_mut() {
            measure.rebeam(strategy);
        }
    }

    pub fn visible_staves(&self) -> Vec<&Staff> {
        self.staves.values().filter(|staff| !staff.hidden).collect()
    }

    pub fn shows_brace(&self) -> bool {
        self.visible_staves().len() > 1
    }
}

impl ScoreElement for Part {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let shows_brace = self.shows_brace();

        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for staff in self.staves.values_mut() {
            result.push(staff);
        }

        for measure in self.measures.values_mut() {
            result.push(measure);
        }

        if shows_brace {
            result.push(&mut self.brace);
        }

        result
    }
}

impl Layoutable for Part {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        if self.visibility == Visibility::Hidden {
            return;
        }

        for staff in self.staves.values_mut() {
            let available = &XY::INFINITE;
            // a staff knows its own size (sum of measure widths, staff height)
            staff.measure(available);

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
            measure.measure(available);
            self.width += measure.width;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        let shows_brace = self.shows_brace();
        if shows_brace {
            let available = XY {
                x: self.width,
                y: staves_height,
            };
            self.brace.measure(&available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
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
        let shows_brace = self.shows_brace();
        if shows_brace {
            let origin = origin.mv(-5., first_visible_staff_distance);
            self.brace.arrange(&origin);
        }
    }
}
