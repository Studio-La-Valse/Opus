use crate::score::part_list::entries::{PartListSection, ScorePart};
use std::collections::BTreeMap;

#[derive(Copy, Clone)]
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

#[derive(Default, Clone)]
pub struct Appearance {
    pub light_barline: Option<f32>,
    pub heavy_barline: Option<f32>,
    pub beam_thickness: Option<f32>,
    pub staff: Option<f32>,
    pub stem_thickness: Option<f32>,
    pub note_size_grace: Option<f32>,
    pub note_size_cue: Option<f32>,
}

#[derive(Clone)]
pub struct Layout {
    pub work_title: String,

    pub defaults: Defaults,
    pub appearance: Appearance,

    pub page_margins_both: Option<PageMargins>,
    pub page_margins_even: Option<PageMargins>,
    pub page_margins_odd: Option<PageMargins>,

    pub system_margin_left: f32,
    pub system_margin_right: f32,
    pub system_distance: f32,
    pub top_system_distance: f32,

    pub staff_distance: f32,

    pub parts: BTreeMap<String, ScorePart>,
    pub sections: BTreeMap<u32, PartListSection>,
}

impl Layout {
    pub fn get_margins(&self, page_number: u32) -> PageMargins {
        let is_even = page_number.is_multiple_of(2);

        if is_even {
            self.page_margins_even
                .or(self.page_margins_both)
                .unwrap_or_default()
        } else {
            self.page_margins_odd
                .or(self.page_margins_both)
                .unwrap_or_default()
        }
    }
}

impl Default for Layout {
    fn default() -> Layout {
        Layout {
            work_title: Default::default(),
            defaults: Default::default(),
            appearance: Default::default(),

            page_margins_odd: None,
            page_margins_even: None,
            page_margins_both: None,

            system_margin_left: 0.,
            system_margin_right: 0.,
            system_distance: 130.,
            top_system_distance: 70.,

            staff_distance: 80.,

            parts: BTreeMap::new(),
            sections: BTreeMap::new(),
        }
    }
}
