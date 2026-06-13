use crate::core::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::layout::{Layout, UserLayout};
use crate::score::layout::PageMargins;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::system::System;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

pub struct Page {
    pub systems: BTreeMap<u32, System>,

    pub number: u32,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub margins: PageMargins,
    pub color: Color,
    pub foreground: Color,
}

impl Page {}

impl ScoreElement for Page {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.systems.iter_mut() {
            result.push(staff);
        }

        result
    }

    fn apply_layout(&mut self, layout: &Layout, user_layout: &UserLayout) {
        self.color = layout.page_color;

        let user_page_color = user_layout.page_color;
        if let Some(user_page_color) = user_page_color {
            self.color = user_page_color;
        }

        for child in self.children() {
            child.apply_layout(layout, user_layout);
        }
    }
}

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

        let mut first = true;
        for (_idx, system) in self.systems.iter_mut() {
            let s_m_left = system.m_left;
            let s_left = origin.x + s_m_left;

            let mut s_m_top = system.distance;
            if first {
                s_m_top = system.top;
                first = false
            }

            let s_top = origin.y + s_m_top;

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
        let mut elements: Vec<Element> = Vec::new();

        let rect = Rect {
            xy: self.xy,
            width: self.width,
            height: self.height,
            color: self.color,
            stroke_color: self.foreground,
            stroke_width: 0.01,
        };
        elements.push(rect.into());

        let stroke_color = Color {
            a: 0.5,
            ..self.foreground
        };
        let stroke_width = 0.5;
        let left = Line {
            start: self.xy.mv(self.margins.left, 0.),
            end: self.xy.mv(self.margins.left, self.height),
            stroke_width,
            stroke_color,
        };
        elements.push(left.into());

        let right = Line {
            start: self.xy.mv(self.width - self.margins.right, 0.),
            end: self.xy.mv(self.width - self.margins.right, self.height),
            stroke_width,
            stroke_color,
        };
        elements.push(right.into());

        let top = Line {
            start: self.xy.mv(0., self.margins.top),
            end: self.xy.mv(self.width, self.margins.top),
            stroke_width,
            stroke_color,
        };
        elements.push(top.into());

        let bottom = Line {
            start: self.xy.mv(0., self.height - self.margins.bottom),
            end: self.xy.mv(self.width, self.height - self.margins.bottom),
            stroke_width,
            stroke_color,
        };
        elements.push(bottom.into());

        elements
    }
}
