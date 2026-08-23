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
