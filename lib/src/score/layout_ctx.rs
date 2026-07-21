use crate::score::core::clef::Clef;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::core::voice::Voice;
use std::collections::{BTreeMap, HashSet};

#[derive(Default, Copy, Clone, Eq, PartialEq, Hash)]
pub enum Visibility {
    #[default]
    Unset,
    Hidden,
    Shown,
}

#[derive(Clone)]
pub struct PageInfo {
    pub page_number: u32,
}

#[derive(Clone)]
pub struct SystemInfo {
    pub index: u32,
    pub margin_left: Option<f32>,
    pub margin_right: Option<f32>,
    pub distance: Option<f32>,
    pub distance_top: Option<f32>,
}

#[derive(Clone)]
pub struct StaffInfo {
    pub number: u32,
    pub visibility: Visibility,

    pub explicitly_hidden: HashSet<StaffIdx>,
    pub explicitly_shown: HashSet<StaffIdx>,

    pub distances: BTreeMap<StaffIdx, f32>,
    pub staff_scaling: BTreeMap<StaffIdx, f32>,
    pub content_scaling: BTreeMap<StaffIdx, f32>,

    /// Currently active clef, tracked across one part across staves.
    pub active_clef: BTreeMap<StaffIdx, Clef>,

    /// The opening clefs for each staff in this part in a system, reset on each new system.
    pub opening_clef: BTreeMap<StaffIdx, Clef>,

    /// The clef changes for each staff in this part in a single part measure.
    pub clef_changes: BTreeMap<StaffIdx, BTreeMap<u32, Clef>>,
}

impl StaffInfo {
    pub fn active_clef(&self, staff_idx: &StaffIdx, position: &u32) -> Clef {
        if let Some(clef_changes) = self.clef_changes.get(staff_idx) {
            let mut clef: Option<Clef> = None;
            for (clef_pos, clef_change) in clef_changes.iter() {
                if clef_pos > position {
                    break;
                }

                clef = Some(*clef_change);
            }

            if let Some(clef) = clef {
                return clef;
            }
        };

        if let Some(opening_clef) = self.opening_clef.get(staff_idx) {
            return *opening_clef;
        }

        Clef::Treble
    }
}

#[derive(Clone)]
pub struct MeasureInfo {
    pub number: u32,
    pub width: Option<f32>,
}

#[derive(Clone)]
pub struct LayoutCtx {
    pub part_id: String,
    pub part_hidden_specified: Visibility,
    pub page: PageInfo,
    pub system: SystemInfo,
    pub staff: StaffInfo,
    pub measure: MeasureInfo,

    pub divisions: u32,
    pub duration: u32,
    pub beats: u32,
    pub beat_type: u32,
    pub position: u32,
    pub voice: Voice,

    pub chord: bool,
}

impl LayoutCtx {
    pub fn reset(&mut self) {
        self.system.index = 0;

        self.page.page_number = 1;

        self.system.margin_left = None;
        self.system.margin_right = None;
        self.system.distance = None;
        self.system.distance_top = None;

        self.part_hidden_specified = Visibility::Unset;

        self.staff.number = 1;
        self.staff.distances.clear();
        self.staff.explicitly_hidden.clear();
        self.staff.explicitly_shown.clear();
        self.staff.staff_scaling.clear();
        self.staff.content_scaling.clear();
        self.staff.active_clef.clear();
        self.staff.opening_clef.clear();

        self.divisions = 8; // specifies the amounts of divisions in one beat (so in one 1/beat_type)
        self.beats = 4;
        self.beat_type = 4;
        self.voice = 1.into();

        self.chord = false;
    }
}

impl Default for LayoutCtx {
    fn default() -> Self {
        LayoutCtx {
            part_id: "".to_string(),
            part_hidden_specified: Visibility::Unset,
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
                distances: BTreeMap::new(),
                visibility: Visibility::Unset,
                explicitly_hidden: HashSet::new(),
                explicitly_shown: HashSet::new(),
                staff_scaling: BTreeMap::new(),
                content_scaling: BTreeMap::new(),
                active_clef: BTreeMap::new(),
                opening_clef: BTreeMap::new(),
                clef_changes: BTreeMap::new(),
            },
            measure: MeasureInfo {
                number: 0,
                width: None,
            },

            duration: 0,
            divisions: 8,
            beats: 4,
            beat_type: 4,
            position: 0,
            voice: 1.into(),

            chord: false,
        }
    }
}
