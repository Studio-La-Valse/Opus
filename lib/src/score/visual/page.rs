use crate::app_defaults::AppDefaults;
use crate::core::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::layout::Layout;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::layout::PageMargins;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::system::System;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::part_measure::PartMeasure;
use crate::visual::staff_measure::StaffMeasure;
use std::collections::BTreeMap;

#[derive(Default)]
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

impl Page {
    pub fn get_system_or_insert(&mut self, system_id: u32) -> &mut System {
        self.systems.entry(system_id).or_default()
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_idx: &StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        self.systems
            .values_mut()
            .find_map(|system| system.locate_staff_measure_mut(part_id, *staff_idx, measure_number))
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for system in self.systems.values_mut() {
            system.rebeam(strategy);
        }
    }
}

impl ScoreElement for Page {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.systems.iter_mut() {
            result.push(staff);
        }

        result
    }

    fn _apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.margins = layout.get_margins(self.number);
        self.width = layout.defaults.page_width;
        self.height = layout.defaults.page_height;
        self.color = user_layout.page_color.unwrap_or(app_defaults.page_color);
        self.foreground = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
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

        // top left of available space after margins
        let mut origin = origin.mv(m_left, m_top);

        let mut first = true;
        for (_idx, system) in self.systems.iter_mut() {
            let s_m_left = system.m_left;
            let s_left = origin.x + s_m_left;

            // space on top of system is either its margin to previous if any,
            // else the distance to top of margins
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

            origin = origin.mv(0., system.height + s_m_top);
        }
    }
}

impl DrawableContent for Page {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        self.systems
            .values()
            .map(|s| s as &dyn DrawableContent)
            .collect()
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut elements: Vec<DrawableElement> = Vec::new();

        let rect = Rect {
            xy: self.xy,
            width: self.width,
            height: self.height,
            color: self.color,
            stroke_color: Some(self.foreground),
            stroke_width: Some(1.),
        };
        elements.push(rect.into());

        let stroke_color = Color {
            a: 1.,
            ..Color::RED
        };
        let stroke_width = 1.;
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
