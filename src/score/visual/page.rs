use crate::core::color::Color;
use crate::score::layout::PageMargins;
use crate::score::visual::system::System;
use std::collections::HashMap;

pub struct Page {
    pub number: u32,
    pub margins: PageMargins,
    pub color: Color,
    pub foreground: Color,
    pub systems: HashMap<u32, System>,
}

impl Page {
    pub fn m_left(&self) -> f32 {
        self.margins.left
    }

    pub fn m_right(&self) -> f32 {
        self.margins.right
    }
}
