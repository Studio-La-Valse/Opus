use crate::score::visual::page::Page;
use std::collections::HashMap;

#[derive(Default)]
pub struct Score {
    pub pages: HashMap<u32, Page>,
}

impl Score {}
