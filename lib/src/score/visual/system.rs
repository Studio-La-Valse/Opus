use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::layout_options::APP_DEFAULTS;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::clef::Clef;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::section::Section;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::tie::TieSegment;
use std::collections::BTreeMap;

/// Addresses one system in the score: its page's key and its own key, both taken
/// from the enclosing `BTreeMap`s.
///
/// Deliberately the map keys rather than any stored index: they are document
/// order by construction, which is exactly what "does the tie's start come
/// before its end" means, and what tells two beam fragments on either side of a
/// break apart. `Page::number` happens to carry the same value today, but it is
/// there for margin resolution and nothing here should depend on the two staying
/// in step.
///
/// Lives beside [`System`] rather than with either pass that keys on it: it
/// addresses a `System`, and both
/// [`BeamArranger`](crate::score::visual::arranger::BeamArranger) and
/// [`TieArranger`](crate::score::visual::arranger::TieArranger) file their
/// output under it.
pub type SystemKey = (u32, u32);

/// A system's horizontal extent, all a broken tie needs to know about it.
#[derive(Copy, Clone, Debug)]
pub struct SystemExtent {
    pub left: f32,
    pub right: f32,
}

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
    /// [`TieArranger`](crate::score::visual::arranger::TieArranger) once
    /// the pages have been arranged.
    ///
    /// A tie broken across a system break contributes one segment here and one
    /// to the next system; a tie broken across a *page* break is the same case,
    /// since the two systems are simply on different pages.
    pub ties: Vec<TieSegment>,

    /// The beam segments that fall inside this system, rebuilt from the chords
    /// in the tree by
    /// [`BeamArranger`](crate::score::visual::arranger::BeamArranger)
    /// once the pages have been arranged.
    ///
    /// A beam group is a run of consecutive chords, so unlike a tie it does not
    /// need naming from outside the tree -- but it breaks the same way: a group
    /// crossing a system break contributes segments here and to the next system,
    /// and a group crossing a *page* break is the same case again.
    pub beams: Vec<Polygon>,

    /// The mid-measure clef changes that fall inside this system, rebuilt from
    /// `Score::clef_changes` by
    /// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger)
    /// once the pages have been arranged.
    ///
    /// Unlike a tie or a beam group a clef change cannot straddle a break -- it
    /// sits next to one note or rest, and that note is on one system. It is
    /// filed here all the same, because what it needs to be placed is spread
    /// across the tree in the same way; see
    /// [`ClefChange`](crate::score::visual::clef::ClefChange).
    pub clef_changes: Vec<Clef>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub staff_line_thickness: f32,
    pub light_barline: f32,

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

    /// The mutable twin of [`visible_staves`](Self::visible_staves).
    pub fn visible_staves_mut(&mut self) -> impl Iterator<Item = &mut Staff> {
        self.sections
            .values_mut()
            .flat_map(|section| section.visible_staves_mut())
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

    /// The staff the system starts on, which must sit no distance at all below
    /// the top of the system. The mutable twin of
    /// [`find_first_visible_staff`](Self::find_first_visible_staff), and `None`
    /// for a system with nothing drawn in it at all.
    pub fn first_visible_staff(&mut self) -> Option<&mut Staff> {
        self.visible_staves_mut().next()
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for section in self.sections.values_mut() {
            section.rebeam(strategy);
        }
    }
}

impl System {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            font,
        } = params;

        self.color = params.foreground_color();

        self.staff_line_thickness = user_layout
            .staff
            .line_width
            .or(score_defaults.appearance.staff)
            .or(font.layout.staff.line_width)
            .unwrap_or(APP_DEFAULTS.staff.line_width);

        self.light_barline = user_layout
            .barline
            .light
            .or(score_defaults.appearance.light_barline)
            .or(font.layout.barline.light)
            .unwrap_or(APP_DEFAULTS.barline.light);

        for section in self.sections.values_mut() {
            section.resolve_layout(params);
        }

        for measure in self.measures.values_mut() {
            measure.resolve_layout(params);
        }
    }
}
