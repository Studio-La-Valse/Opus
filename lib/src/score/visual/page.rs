use crate::core::color::Color;
use crate::core::xy::XY;
use crate::score::drawable::content::Content;
use crate::score::drawable::element::Element;
use crate::score::drawable::elements::rect::Rect;
use crate::score::layout::PageMargins;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::system::System;
use std::collections::HashMap;

pub struct Page {
    pub number: u32,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub margins: PageMargins,
    pub color: Color,
    pub foreground: Color,
    pub systems: HashMap<u32, System>,
}

impl Page {}

impl Layoutable for Page {
    fn measure(&mut self, available: &XY) {
        for (_idx, system) in self.systems.iter_mut() {
            system.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
        let m_left = self.margins.left;
        let m_top = self.margins.top;

        let mut origin = origin.mv(m_left, m_top);

        for (_idx, system) in self.systems.iter_mut() {
            let s_m_left = system.m_left;
            let s_left = origin.x + s_m_left;

            let s_m_top = system.top;
            let s_top = origin.y + m_top + s_m_top;

            let s_origin = XY {
                x: s_left,
                y: s_top,
            };
            system.arrange(&s_origin);

            origin = origin.mv(0., system.height);
        }
    }
}

impl Content for Page {
    fn content(&self) -> Vec<&dyn Content> {
        self.systems.values().map(|s| s as &dyn Content).collect()
    }

    fn elements(&self) -> Vec<Element> {
        vec![
            Rect {
                xy: self.xy,
                width: self.width,
                height: self.height,
                color: Color::WHITE,
                stroke_color: Color::BLACK,
                stroke_width: 0.01,
            }
            .into(),
        ]
    }
}
