use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub enum Visibility {
    Auto,
    Hidden,
    Shown,
}

#[derive(Debug, Clone)]
pub struct PageInfo {
    pub page_number: u32,
}

#[derive(Debug, Clone)]
pub struct SystemInfo {
    pub index: u32,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub distance: Option<f32>,
    pub distance_top: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct StaffInfo {
    pub number: u32,
    pub distances: HashMap<u32, f32>,
    pub visibility: Visibility,
    pub explicitly_hidden: HashSet<u32>,
    pub explicitly_shown: HashSet<u32>,
}

#[derive(Debug, Clone)]
pub struct MeasureInfo {
    pub number: u32,
    pub width: Option<f32>,
}

#[derive(Debug, Clone)]
pub struct LayoutCtx {
    pub part_id: String,
    pub part_hidden_specified: Visibility,
    pub page: PageInfo,
    pub system: SystemInfo,
    pub staff: StaffInfo,
    pub measure: MeasureInfo,
}

impl LayoutCtx {
    pub fn reset(&mut self) {
        self.system.index = 0;

        self.page.page_number = 1;
        self.staff.number = 1;

        self.system.margin_left = None;
        self.system.margin_right = None;
        self.system.distance = None;
        self.system.distance_top = None;

        self.staff.distances.clear();

        self.part_hidden_specified = Visibility::Auto;
        self.staff.explicitly_hidden.clear();
        self.staff.explicitly_shown.clear();
    }
}

impl Default for LayoutCtx {
    fn default() -> Self {
        LayoutCtx {
            part_id: "".to_string(),
            part_hidden_specified: Visibility::Auto,
            page: PageInfo { page_number: 1 },
            system: SystemInfo {
                index: 0,
                margin_left: None,
                margin_right: None,
                distance: None,
                distance_top: None,
            },
            staff: StaffInfo {
                number: 1,
                distances: HashMap::new(),
                visibility: Visibility::Auto,
                explicitly_hidden: HashSet::new(),
                explicitly_shown: HashSet::new(),
            },
            measure: MeasureInfo {
                number: 0,
                width: None,
            },
        }
    }
}
