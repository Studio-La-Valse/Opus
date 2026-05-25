use crate::core::color::Color;
use std::collections::HashMap;

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

#[derive(Default)]
pub struct Part {
    pub name: String,
    pub abbr: String,

    pub section: i32,
    pub part_group: i32,

    pub brace: Option<String>,
}

#[derive(Default)]
pub struct Section {
    pub brace: Option<String>,
    pub groups: HashMap<i32, PartGroup>,
}

#[derive(Default)]
pub struct PartGroup {
    pub brace: Option<String>,
    pub name: Option<String>,
}

pub struct Layout {
    pub page_color: Color,
    pub foreground_color: Color,

    pub work_title: String,

    pub defaults: Defaults,

    pub page_margins_both: Option<PageMargins>,
    pub page_margins_even: Option<PageMargins>,
    pub page_margins_odd: Option<PageMargins>,

    pub system_margin_left: f32,
    pub system_margin_right: f32,
    pub system_distance: f32,
    pub top_system_distance: f32,

    pub staff_distance: f32,

    pub staff_line_thickness: f32,
    pub bar_line_light_thickness: f32,
    pub bar_line_heavy_thickness: f32,

    pub parts: HashMap<String, Part>,
    pub sections: HashMap<i32, Section>,
}

impl Default for Layout {
    fn default() -> Layout {
        Layout {
            page_color: Color::WHITE,
            foreground_color: Color::BLACK,

            work_title: Default::default(),

            defaults: Default::default(),

            page_margins_odd: None,
            page_margins_even: None,
            page_margins_both: None,

            system_margin_left: 0.,
            system_margin_right: 0.,
            system_distance: 130.,
            top_system_distance: 70.,

            staff_distance: 80.,

            staff_line_thickness: 1.25,
            bar_line_heavy_thickness: 5.,
            bar_line_light_thickness: 1.875,

            parts: HashMap::new(),
            sections: HashMap::new(),
        }
    }
}

impl Layout {
    pub fn apply_user_layout(&mut self, user_layout: &UserLayout) {
        self.page_color = user_layout.page_color.unwrap_or(self.page_color);
        self.foreground_color = user_layout
            .foreground_color
            .unwrap_or(self.foreground_color);

        self.staff_line_thickness = user_layout
            .staff_line_thickness
            .unwrap_or(self.staff_line_thickness);
        self.bar_line_heavy_thickness = user_layout
            .bar_line_heavy_thickness
            .unwrap_or(self.bar_line_light_thickness);
        self.bar_line_light_thickness = user_layout
            .bar_line_light_thickness
            .unwrap_or(self.bar_line_light_thickness);

        self.page_margins_both = user_layout.page_margins_both.or(self.page_margins_both);
        self.page_margins_even = user_layout.page_margins_even.or(self.page_margins_even);
        self.page_margins_odd = user_layout.page_margins_odd.or(self.page_margins_odd);
    }
}

#[derive(Default)]
pub struct UserLayout {
    pub page_color: Option<Color>,
    pub foreground_color: Option<Color>,

    pub staff_line_thickness: Option<f32>,
    pub bar_line_light_thickness: Option<f32>,
    pub bar_line_heavy_thickness: Option<f32>,

    pub page_margins_both: Option<PageMargins>,
    pub page_margins_even: Option<PageMargins>,
    pub page_margins_odd: Option<PageMargins>,
}
