use crate::score::core::group_symbol::GroupLevel;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::score_defaults::ScorePart;
use crate::score::visual::clef::ClefChange;
use crate::score::visual::group_name::GroupName;
use crate::score::visual::group_symbol::GroupSymbol;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::note::{NoteAnchor, NoteId};
use crate::score::visual::page::Page;
use crate::score::visual::part::Part;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::score::visual::system::{System, SystemExtent, SystemKey};
use crate::score::visual::system_measure::SystemMeasure;
use crate::score::visual::tie::Tie;
use crate::score::walk_cursor::Visibility;
use std::collections::{BTreeMap, HashMap};

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
    /// [`TieArranger`](crate::score::visual::arranger::TieArranger).
    pub ties: Vec<Tie>,

    /// Every mid-measure clef change in the document, as a flat list.
    ///
    /// Here for a variation on the reason `ties` is: a clef change belongs to a
    /// staff, but only the note or rest it precedes can say where on the page it
    /// goes, and those live in a different branch of the tree. It used to be
    /// stored on the anchor itself, which left every `Chord` and `Rest` carrying
    /// a clef slot they almost never filled.
    ///
    /// Populated by the content walk and left alone by `arrange`; the drawn
    /// clefs live on
    /// [`System::clef_changes`](crate::score::visual::system::System) and are
    /// rebuilt from this list by
    /// [`ClefChangeArranger`](crate::score::visual::arranger::ClefChangeArranger).
    pub clef_changes: Vec<ClefChange>,
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
        // The first system of the score names in full, every later one draws
        // the abbreviation; see `System::index`.
        let abbreviate_names = system_index > 1;

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
            GroupName::new(
                assignment.part_group_name.clone(),
                assignment.part_group_abbr.clone(),
                abbreviate_names,
            ),
        );

        // MusicXML declares a part's own symbol in `<attributes><part-symbol>`,
        // which nothing reads yet, so this level has nothing to pass on.
        part_group.part_or_insert(
            part_id.to_string(),
            GroupSymbol::new(GroupLevel::Part, None),
            GroupName::new(
                assignment.name.clone(),
                assignment.abbr.clone(),
                abbreviate_names,
            ),
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
    /// tree ahead of
    /// [`measure_score`](crate::score::visual::measure_machine::MeasureMachine::measure_score)
    /// -- see [`arrange_score`](crate::score::engrave::arrange_score).
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        for page in self.pages.values_mut() {
            page.resolve_layout(params);
        }
    }

    /// Every note's final geometry, keyed by id.
    ///
    /// Indexes exactly the notes `RenderCompositor::walk_pages` draws, so a tie can
    /// never point at a note that is not on the page. That means skipping hidden
    /// *parts* but not hidden *staves*: the compositor's `walk_part` skips a hidden
    /// staff when drawing staff lines, yet still walks every `PartMeasure` chord
    /// regardless of which staff its notes sit on, so those noteheads do get drawn.
    /// Filtering them here instead would leave a notehead rendered with its tie
    /// missing. One O(notes) pass per arrange, negligible next to the arrange
    /// itself.
    pub fn note_anchors(&self) -> HashMap<NoteId, NoteAnchor> {
        let mut anchors = HashMap::new();

        for (page_key, page) in self.pages.iter() {
            for (system_key, system) in page.systems.iter() {
                let key = (*page_key, *system_key);

                for section in system.sections.values() {
                    for group in section.part_groups.values() {
                        for part in group.parts.values() {
                            if part.visibility == Visibility::Hidden {
                                continue;
                            }

                            for measure in part.measures.values() {
                                // `ArrangeMachine::arrange_part` walks its measures left to
                                // right, advancing the origin by each
                                // measure's width, so these two are the
                                // measure's own span.
                                let measure_right = measure.origin.x + measure.width;

                                for chord in measure.chords.values().flatten() {
                                    let stem = chord.stem.as_ref().map(|s| s.direction);

                                    for note in chord.notes.iter() {
                                        anchors.insert(
                                            note.id,
                                            NoteAnchor {
                                                key,
                                                left: note.xy,
                                                width: note.width,
                                                measure_right,
                                                scale: note.scale,
                                                staff_line: note.staff_line,
                                                stem,
                                                color: note.color,
                                            },
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        anchors
    }

    /// Every system's horizontal extent, keyed the same way as
    /// [`note_anchors`](Self::note_anchors).
    pub fn system_extents(&self) -> HashMap<SystemKey, SystemExtent> {
        let mut extents = HashMap::new();

        for (page_key, page) in self.pages.iter() {
            for (system_key, system) in page.systems.iter() {
                extents.insert(
                    (*page_key, *system_key),
                    SystemExtent {
                        left: system.xy.x,
                        right: system.xy.x + system.width,
                    },
                );
            }
        }

        extents
    }
}
