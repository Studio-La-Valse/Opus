use crate::score::visual::section::Section;
use std::collections::HashMap;

#[derive(Default)]
pub struct System {
    pub sections: HashMap<i32, Section>,

    pub m_left: f32,
    pub m_right: f32,
    pub distance: f32,
    pub top: f32,
}

impl System {}
