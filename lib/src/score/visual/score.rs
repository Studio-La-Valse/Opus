use crate::geometry::xy::XY;
use crate::score::core::group_symbol::GroupLevel;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::score_defaults::ScorePart;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::page::Page;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system::System;
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::tie::Tie;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Score {
    pub pages: BTreeMap<u32, Page>,

    /// Every tie in the document, as a flat list of note-id pairs.
    ///
    /// A tie is a *relation* between two notes, and the page tree can only own
    /// what it contains -- the two endpoints may be measures, systems or pages
    /// apart. Hanging the tie on its start `Note` instead would force every
    /// arrange to walk the whole tree just to discover which notes start ties,
    /// and would pick one endpoint as the owner arbitrarily. A flat list is also
    /// where the spanners that follow ties belong: slurs, hairpins, ottavas,
    /// pedal lines.
    ///
    /// Populated by the content walk and left alone by `arrange`; the drawn arcs
    /// live on [`System::ties`](crate::score::visual::system::System) and are
    /// rebuilt from this list by
    /// [`arrange_ties`](crate::score::visual::tie_arranger::arrange_ties).
    pub ties: Vec<Tie>,
}

impl Score {
    /// The page under `page_number`, creating it if it isn't there yet. The
    /// created page is stamped with its own number, which `Page::resolve_layout`
    /// needs to pick odd- or even-page margins.
    pub fn page_or_insert(&mut self, page_number: u32) -> &mut Page {
        self.pages.entry(page_number).or_insert_with(|| Page {
            number: page_number,
            ..Default::default()
        })
    }

    /// Walks page -> system -> section -> part group -> part, creating every
    /// level on the way down. Both the layout and the content walk pass need the
    /// same part to exist before they can populate its measure, so they share
    /// this descent.
    ///
    /// A level created here is given the group symbol its `<part-group>`
    /// declared, which is all this can settle: what is actually *drawn* depends
    /// on the user layout too, and is resolved on every layout pass. Which is
    /// also why no font is needed here -- a symbol reads its glyph when it
    /// measures, not when it is built.
    pub fn locate_or_create_part(
        &mut self,
        page_number: u32,
        system_index: u32,
        assignment: &ScorePart,
        part_id: &str,
    ) -> &mut Part {
        let system = self
            .page_or_insert(page_number)
            .system_or_insert(system_index);

        let section = system.section_or_insert(
            assignment.section,
            GroupSymbol::new(GroupLevel::Section, assignment.section_symbol),
        );
        let part_group = section.part_group_or_insert(
            assignment.part_group,
            GroupSymbol::new(GroupLevel::PartGroup, assignment.part_group_symbol),
        );

        // MusicXML declares a part's own symbol in `<attributes><part-symbol>`,
        // which nothing reads yet, so this level has nothing to pass on.
        part_group.part_or_insert(
            part_id.to_string(),
            GroupSymbol::new(GroupLevel::Part, None),
        )
    }

    pub fn locate_system_mut(&mut self, system_idx: &u32) -> Option<&mut System> {
        self.pages
            .values_mut()
            .find_map(|page| page.systems.get_mut(system_idx))
    }

    pub fn locate_system_measure_mut(&mut self, measure_number: u32) -> Option<&mut SystemMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_system_measure_mut(measure_number))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_idx: &StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| page.locate_staff_measure_mut(part_id, staff_idx, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        self.pages
            .values_mut()
            .find_map(|page| {
                if page.locate_system_measure_mut(measure_number).is_some() {
                    Some(page.locate_staff_measures_mut(part_id, measure_number))
                } else {
                    None
                }
            })
            .unwrap_or_default()
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for page in self.pages.values_mut() {
            page.rebeam(strategy);
        }
    }

    /// Resolves every page's appearance from `params`. Run once over the whole
    /// tree ahead of [`measure`](Score::measure) -- see
    /// [`arrange_score`](crate::score::engrave::arrange_score).
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for page in self.pages.values_mut() {
            page.resolve_layout(params);
        }
    }

    /// Sizes every page. Page *placement* is a separate pass -- see
    /// [`LayoutEngine::arrange_pages`](crate::score::visual::layout_engine::LayoutEngine::arrange_pages),
    /// which is why `Score` has no `arrange`.
    pub fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        for page in self.pages.values_mut() {
            page.measure(available, params);
        }
    }
}
