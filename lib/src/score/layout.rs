use crate::score::part_list::tree::PartListNode;

/// A `<score-part>`'s section/part-group assignment, returned by
/// `Layout::lookup` for a given part id.
#[derive(Default, Clone, Copy)]
pub struct ScorePart {
    pub section: u32,
    pub part_group: u32,
}

#[derive(Copy, Clone)]
pub struct Defaults {
    pub scaling_millimeters: f32,
    pub scaling_tenths: f32,
    pub page_height: f32,
    pub page_width: f32,
}

impl Default for Defaults {
    fn default() -> Defaults {
        Defaults {
            scaling_millimeters: 6.35,
            scaling_tenths: 40.,
            page_height: 1760.,
            page_width: 1360.,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PageMargins {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Default for PageMargins {
    fn default() -> PageMargins {
        PageMargins {
            left: 80.,
            right: 80.,
            top: 80.,
            bottom: 80.,
        }
    }
}

#[derive(Default, Clone)]
pub struct Appearance {
    pub light_barline: Option<f32>,
    pub heavy_barline: Option<f32>,
    pub beam_thickness: Option<f32>,
    pub staff: Option<f32>,
    pub stem_thickness: Option<f32>,
    pub note_size_grace: Option<f32>,
    pub note_size_cue: Option<f32>,
}

#[derive(Clone)]
pub struct Layout {
    pub work_title: String,

    pub defaults: Defaults,
    pub appearance: Appearance,

    pub page_margins_both: Option<PageMargins>,
    pub page_margins_even: Option<PageMargins>,
    pub page_margins_odd: Option<PageMargins>,

    pub system_margin_left: f32,
    pub system_margin_right: f32,
    pub system_distance: f32,
    pub top_system_distance: f32,

    pub staff_distance: f32,

    pub part_list: Vec<PartListNode>,
}

impl Layout {
    pub fn get_margins(&self, page_number: u32) -> PageMargins {
        let is_even = page_number.is_multiple_of(2);

        if is_even {
            self.page_margins_even
                .or(self.page_margins_both)
                .unwrap_or_default()
        } else {
            self.page_margins_odd
                .or(self.page_margins_both)
                .unwrap_or_default()
        }
    }

    /// Finds a part's section/part-group assignment by id, searching the
    /// part-list tree directly -- a part-list is small enough that a linear
    /// walk costs nothing, so there's no need to also keep a flattened map.
    pub fn lookup(&self, part_id: &str) -> Option<ScorePart> {
        fn find(nodes: &[PartListNode], part_id: &str) -> Option<ScorePart> {
            for node in nodes {
                match node {
                    PartListNode::Part {
                        id,
                        section,
                        part_group,
                        ..
                    } if id == part_id => {
                        return Some(ScorePart {
                            section: *section,
                            part_group: *part_group,
                        });
                    }
                    PartListNode::Section { children, .. }
                    | PartListNode::Group { children, .. } => {
                        if let Some(found) = find(children, part_id) {
                            return Some(found);
                        }
                    }
                    PartListNode::Part { .. } => {}
                }
            }

            None
        }

        find(&self.part_list, part_id)
    }

    /// Ensures a part is present in the part-list, registering a bare
    /// top-level entry (no name/abbr, default section/part-group 0) if the
    /// id was never declared in `<part-list>`.
    pub fn ensure_part(&mut self, part_id: &str) {
        if self.lookup(part_id).is_some() {
            return;
        }

        self.part_list.push(PartListNode::Part {
            id: part_id.to_string(),
            name: String::new(),
            abbr: String::new(),
            section: 0,
            part_group: 0,
        });
    }
}

impl Default for Layout {
    fn default() -> Layout {
        Layout {
            work_title: Default::default(),
            defaults: Default::default(),
            appearance: Default::default(),

            page_margins_odd: None,
            page_margins_even: None,
            page_margins_both: None,

            system_margin_left: 0.,
            system_margin_right: 0.,
            system_distance: 130.,
            top_system_distance: 70.,

            staff_distance: 80.,

            part_list: Vec::new(),
        }
    }
}
