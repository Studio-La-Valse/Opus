use crate::score::visual::part::Part;
use std::collections::HashMap;

#[derive(Default)]
pub struct PartGroup {
    pub parts: HashMap<String, Part>,
}

impl PartGroup {}
