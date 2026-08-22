use std::collections::BTreeMap;

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
pub struct Part {
    pub name: String,
    pub abbr: String,

    pub section: u32,
    pub part_group: u32,

    pub brace: Option<String>,
}

#[derive(Default, Clone)]
pub struct Section {
    pub name: Option<String>,
    pub brace: Option<String>,
    pub groups: BTreeMap<u32, PartGroup>,
}

#[derive(Default, Clone)]
pub struct PartGroup {
    pub brace: Option<String>,
    pub name: Option<String>,
}

/// Tracks `<part-group>` start/stop nesting while walking a `<part-list>`,
/// assigning each `<score-part>` its section/part-group indices. This is the
/// exact state machine used to populate `Layout::sections` -- shared so a
/// lighter-weight walk (e.g. validation, which only wants counts) can never
/// disagree with what render actually assigns.
#[derive(Default)]
pub struct PartListTracker {
    first_order_index: u32,
    second_order_index: u32,
    first_order_group_open: bool,
    second_order_group_open: bool,

    pub sections_opened: u32,
    pub part_groups_opened: u32,
}

pub enum PartGroupLevel {
    Section { index: u32 },
    Group { section: u32, index: u32 },
}

impl PartListTracker {
    /// Call on a `<part-group type="start">`. Returns which level actually
    /// opened, or `None` if a section and a part-group were both already open.
    pub fn open_part_group(&mut self) -> Option<PartGroupLevel> {
        if !self.first_order_group_open {
            self.first_order_group_open = true;
            self.second_order_index = 0;
            self.second_order_group_open = false;
            self.sections_opened += 1;

            Some(PartGroupLevel::Section {
                index: self.first_order_index,
            })
        } else if !self.second_order_group_open {
            self.second_order_group_open = true;
            self.part_groups_opened += 1;

            Some(PartGroupLevel::Group {
                section: self.first_order_index,
                index: self.second_order_index,
            })
        } else {
            None
        }
    }

    /// Call on a `<part-group type="stop">`.
    pub fn close_part_group(&mut self) {
        if self.second_order_group_open {
            self.second_order_index += 1;
            self.second_order_group_open = false;
        } else if self.first_order_group_open {
            self.first_order_index += 1;
            self.first_order_group_open = false;
            self.second_order_index = 0;
            self.second_order_group_open = false;
        }
    }

    /// Call on a `<score-part>`. Returns its (section, part_group) assignment.
    pub fn register_score_part(&mut self) -> (u32, u32) {
        let assignment = (self.first_order_index, self.second_order_index);

        if !self.first_order_group_open {
            self.first_order_index += 1;
            self.second_order_index = 0;
        } else if !self.second_order_group_open {
            self.second_order_index += 1;
        }

        assignment
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

    pub parts: BTreeMap<String, Part>,
    pub sections: BTreeMap<u32, Section>,
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

            parts: BTreeMap::new(),
            sections: BTreeMap::new(),
        }
    }
}
