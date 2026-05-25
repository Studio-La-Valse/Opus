use crate::score::visual::part_group::PartGroup;
use std::collections::HashMap;

#[derive(Default)]
pub struct Section {
    pub part_groups: HashMap<i32, PartGroup>,
}

impl Section {}
