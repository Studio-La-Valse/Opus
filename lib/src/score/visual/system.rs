use crate::drawable::elements::polygon::Polygon;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::clef::Clef;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::section::Section;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_measure::{MeasureStartPaddings, StaffMeasure};
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

    /// Lines the elements that open a measure -- the clef, the key signature and
    /// the time signature -- up into three columns shared by every drawn staff
    /// of this system.
    ///
    /// Left to itself a staff places these one after the other from its own left
    /// edge, so a staff whose key signature is narrower than its neighbour's
    /// starts its time signature further left than theirs. That shows in any
    /// score with transposing instruments: a score in C flat major carries seven
    /// flats, the trumpets in B flat five, and an unpitched percussion staff
    /// none at all, which is three different time-signature positions down one
    /// system.
    ///
    /// A column, like a measure width, is a fact about the whole system that no
    /// single staff can work out on its own -- so, like
    /// [`consolidate_measure_width`](Self::consolidate_measure_width), the
    /// system settles it. Only the *start* of each column is shared: what a
    /// staff draws there is still its own, so a narrower key signature simply
    /// leaves more air before the next column.
    ///
    /// Run by
    /// [`ContentArranger`](crate::score::visual::arranger::ContentArranger),
    /// once the staves have been placed and the measures have the left edge
    /// these offsets are measured from. Only drawn staves take part, which is
    /// also all the compositor visits.
    pub fn arrange_measure_starts(&mut self, paddings: &MeasureStartPaddings) {
        let mut by_measure: BTreeMap<u32, Vec<&mut StaffMeasure>> = BTreeMap::new();
        for staff in self.visible_staves_mut() {
            for (number, measure) in staff.measures.iter_mut() {
                by_measure.entry(*number).or_default().push(measure);
            }
        }

        for measures in by_measure.values_mut() {
            // How far right each staff has reached so far, from the measure's
            // own left edge. The columns resolve outwards from here, one at a
            // time, because each is measured from where the previous one left
            // the widest staff.
            let mut edges = vec![0.; measures.len()];

            arrange_column(
                measures,
                &mut edges,
                paddings.clef,
                StaffMeasure::arrange_clef_start,
            );
            arrange_column(
                measures,
                &mut edges,
                paddings.key_signature,
                StaffMeasure::arrange_key_signature_start,
            );
            arrange_column(
                measures,
                &mut edges,
                paddings.time_signature,
                StaffMeasure::arrange_time_signature_start,
            );
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
    fn first_visible_staff(&mut self) -> Option<&mut Staff> {
        self.visible_staves_mut().next()
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

        self.staff_line_thickness = user_layout
            .staff_line_width
            .or(score_defaults.appearance.staff)
            .unwrap_or(app_defaults.staff_line_width);

        self.light_barline = user_layout
            .light_barline
            .or(score_defaults.appearance.light_barline)
            .unwrap_or(app_defaults.light_barline);

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

/// Places one of the three opening columns across the staves of a single
/// measure.
///
/// Every staff draws at the same offset -- a padding clear of the furthest
/// right any of them has reached -- and `edges` then advances to wherever each
/// staff's own element ended. A staff with nothing to draw in this column
/// answers `None` and keeps the edge it had, so it neither widens the column nor
/// carries a gap for an element it does not have.
fn arrange_column(
    measures: &mut [&mut StaffMeasure],
    edges: &mut [f32],
    padding: f32,
    place: fn(&mut StaffMeasure, f32) -> Option<f32>,
) {
    let column = measures
        .iter()
        .zip(edges.iter())
        .map(|(measure, edge)| edge + padding * measure.scale)
        .fold(0., f32::max);

    for (measure, edge) in measures.iter_mut().zip(edges.iter_mut()) {
        if let Some(right) = place(measure, column) {
            *edge = right;
        }
    }
}
