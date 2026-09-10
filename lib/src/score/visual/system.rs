use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::section::Section;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::tie::TieSegment;
use crate::score::walk_cursor::Visibility;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct System {
    pub sections: BTreeMap<u32, Section>,
    pub measures: BTreeMap<u32, SystemMeasure>,

    /// The tie arcs that fall inside this system, rebuilt from `Score::ties` by
    /// [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties) once
    /// the pages have been arranged.
    ///
    /// A tie broken across a system break contributes one segment here and one
    /// to the next system; a tie broken across a *page* break is the same case,
    /// since the two systems are simply on different pages.
    pub ties: Vec<TieSegment>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub line_width: f32,
    pub staff_line_width: f32,

    pub m_left: f32,
    pub m_right: f32,
    pub distance: f32,
    pub top: f32,
}

impl System {
    pub fn section_or_insert(&mut self, section_id: u32, symbol: GroupSymbol) -> &mut Section {
        self.sections
            .entry(section_id)
            .or_insert_with(|| Section::new(symbol))
    }

    pub fn locate_system_measure_mut(&mut self, measure_number: u32) -> Option<&mut SystemMeasure> {
        self.measures.get_mut(&measure_number)
    }

    pub fn locate_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.sections
            .values_mut()
            .find_map(|pg| pg.locate_part_mut(part_id))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        // First check if the system even contains this measure
        if !self.measures.contains_key(&measure_number) {
            return None;
        }

        self.sections
            .values_mut()
            .find_map(|section| section.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        if !self.measures.contains_key(&measure_number) {
            return Vec::new();
        }

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
        // First check if the system even contains this measure
        if !self.measures.contains_key(&measure_number) {
            return None;
        }

        self.sections.values_mut().find_map(|section| {
            section.locate_staff_measure_mut(part_id, staff_number, measure_number)
        })
    }

    pub fn consolidate_measure_width(&mut self, measure_number: u32) {
        let measure = self.measures.get_mut(&measure_number).unwrap();
        let width = measure.width;

        for section in self.sections.values_mut() {
            let measure = section.measures.entry(measure_number).or_default();
            measure.width = width;

            for part_group in section.part_groups.values_mut() {
                let measure = part_group.measures.entry(measure_number).or_default();
                measure.width = width;

                for part in part_group.parts.values_mut() {
                    let measure = part.measures.entry(measure_number).or_default();
                    measure.width = width;

                    for staff in part.staves.values_mut() {
                        let measure = staff.measures.entry(measure_number).or_default();
                        measure.width = width;
                    }
                }
            }
        }
    }

    /// Every staff of this system that is drawn, top to bottom.
    pub fn visible_staves(&self) -> impl Iterator<Item = &Staff> {
        self.sections
            .values()
            .flat_map(|section| section.visible_staves())
    }

    pub fn find_first_visible_staff(&self) -> &Staff {
        self.visible_staves().next().unwrap()
    }

    pub fn find_last_visible_staff(&self) -> &Staff {
        self.visible_staves().last().unwrap()
    }

    /// Where the systemic barline down the left edge starts and ends, as
    /// offsets from [`System::xy`], which is the top line of the first visible
    /// staff.
    ///
    /// That is the height of the system, save that a staff of a single line at
    /// either end has no height for the barline to take: it overhangs such a
    /// staff by a space at each end instead. See [`Staff::barline_overhang`].
    pub fn barline_span(&self) -> (f32, f32) {
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

        (-top, self.height + bottom)
    }

    /// Every first visible staff in a system must have a 0-distance to the top of the system.
    fn first_visible_staff(&mut self) -> Option<&mut Staff> {
        for section in self.sections.values_mut() {
            for part_group in section.part_groups.values_mut() {
                for part in part_group.parts.values_mut() {
                    if part.visibility == Visibility::Hidden {
                        continue;
                    }

                    for staff in part.staves.values_mut() {
                        if staff.hidden {
                            continue;
                        }

                        // first visible staff found.
                        return Some(staff);
                    }
                }
            }
        }

        None
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for section in self.sections.values_mut() {
            section.rebeam(strategy);
        }
    }
}

impl Layoutable for System {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_width = user_layout
            .light_barline
            .or(score_defaults.appearance.light_barline)
            .unwrap_or(app_defaults.barline_light);

        self.staff_line_width = user_layout
            .staff
            .or(score_defaults.appearance.staff)
            .unwrap_or(app_defaults.staff_line_thickness);

        for section in self.sections.values_mut() {
            section.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }
    }

    fn measure(&mut self, _: &XY, params: LayoutParams<'_>) {
        self.width = 0.;
        self.height = 0.;

        if let Some(staff) = self.first_visible_staff() {
            staff.distance_final = 0.;
        }

        for section in self.sections.values_mut() {
            let available = XY::INFINITE;
            section.measure(&available, params);
            self.height += section.height;
        }

        for measure in self.measures.values_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(&available, params);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut _origin = self.xy;
        for measure in self.measures.values_mut() {
            measure.arrange(&_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for section in self.sections.values_mut() {
            section.arrange(&_origin);
            _origin = _origin.mv(0., section.height);
        }
    }
}
