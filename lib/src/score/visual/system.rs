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
use crate::score::visual::start_columns::{StartColumns, StartMetrics};
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::tie::TieSegment;
use crate::score::walk_cursor::Visibility;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct System {
    /// This system's global index across the whole score, stamped by
    /// [`Page::system_or_insert`](crate::score::visual::page::Page) the way
    /// [`Page::number`](crate::score::visual::page::Page) is stamped by
    /// `Score::page_or_insert`. It comes from the walk cursor, which counts from
    /// zero but increments on the score's first measure, so the first system of
    /// the score is index 1 and every later one is 2, 3, ... Used to decide
    /// whether names abbreviate: the first system names in full, every later one
    /// uses the abbreviation.
    pub index: u32,

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

    /// Settles the three columns the opening clef, key signature and time
    /// signature of every staff are drawn against, one set per measure, and
    /// writes them into the staff measures that will draw them.
    ///
    /// The counterpart of [`consolidate_measure_width`](Self::consolidate_measure_width)
    /// for what is *inside* a measure: a column, like a measure width, is a fact
    /// about the whole system that no single staff can work out on its own. Only
    /// the drawn staves have a say -- a hidden staff's key signature must not
    /// push everyone else's time signature right for a clef nobody sees -- but
    /// the result is written to every staff measure, drawn or not, so that
    /// nothing keeps a column from a previous layout pass.
    ///
    /// Run at the end of [`measure`](Layoutable::measure), where every element's
    /// width is finally known and nothing has been placed yet.
    fn consolidate_start_columns(&mut self) {
        let mut metrics: BTreeMap<u32, Vec<StartMetrics>> = BTreeMap::new();
        for staff in self.visible_staves() {
            for (number, measure) in staff.measures.iter() {
                metrics
                    .entry(*number)
                    .or_default()
                    .push(measure.start_metrics());
            }
        }

        let columns: BTreeMap<u32, StartColumns> = metrics
            .into_iter()
            .map(|(number, metrics)| (number, StartColumns::fold(&metrics)))
            .collect();

        for (number, measure) in self.staff_measures_mut() {
            if let Some(&shared) = columns.get(number) {
                measure.start_columns = shared;
            }
        }
    }

    /// Every staff measure in this system with the measure number it belongs
    /// to, hidden staves and hidden parts included.
    fn staff_measures_mut(&mut self) -> impl Iterator<Item = (&u32, &mut StaffMeasure)> {
        self.sections
            .values_mut()
            .flat_map(|section| section.part_groups.values_mut())
            .flat_map(|part_group| part_group.parts.values_mut())
            .flat_map(|part| part.staves.values_mut())
            .flat_map(|staff| staff.measures.iter_mut())
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
    /// offsets from [`System::xy`], the top line of the first visible staff:
    /// from that line down the whole height of the system. Every staff, a
    /// one-line percussion staff included, is barred its full height.
    pub fn barline_span(&self) -> (f32, f32) {
        (0., self.height)
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

        // The first system names its parts and part-groups in full; every later
        // one uses the abbreviation. Re-stamped here so the whole subtree below
        // resolves against it. The first system of the score is index 1 (see
        // `index`), so anything past it abbreviates.
        let params = LayoutParams {
            abbreviate_names: self.index > 1,
            ..params
        };

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

        // Every opening element now knows its width, which is all the columns
        // are folded from.
        self.consolidate_start_columns();

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

        // The page's left margin, in world coordinates: `page.rs` places a
        // system at `page_margin + system.m_left`, so undoing the system's own
        // margin lands back on it. This is where a name's box reaches left to.
        let margin_left = self.xy.x - self.m_left;

        let mut _origin = self.xy;
        for section in self.sections.values_mut() {
            section.arrange_within(&_origin, margin_left);
            _origin = _origin.mv(0., section.height);
        }
    }
}
