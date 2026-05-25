use crate::score::layout_ctx::Visibility;
use crate::score::visual::staff::Staff;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Part {
    pub staves: HashMap<u32, Staff>,
    pub visibility: Visibility,
}

impl Part {
    pub fn set_visibility(&mut self, visibility: Visibility) {
        match visibility {
            Visibility::Auto => {
                // Auto means: do nothing
            }

            Visibility::Hidden | Visibility::Shown => {
                match self.visibility {
                    Visibility::Shown => {
                        // previously explicitly shown, not allowed to change.
                    }

                    Visibility::Auto | Visibility::Hidden => {
                        // first time set, allowed to set visibility, OR:
                        // hidden: allowed to hide or show.
                        self.visibility = visibility;
                    }
                }
            }
        }
    }

    pub fn ensure_staves(&mut self, staves: HashSet<u32>) {
        self.staves.entry(1).or_default();

        for staff in staves {
            self.staves.entry(staff).or_default();
        }
    }

    pub fn hide_staves(&mut self, staves: &HashSet<u32>) {
        for &staff in staves {
            let staff = self.staves.entry(staff).or_default();
            staff.hidden = true;
        }
    }

    pub fn show_staves(&mut self, staves: &HashSet<u32>) {
        for &staff in staves {
            let staff = self.staves.entry(staff).or_default();
            staff.hidden = false;
        }
    }

    pub fn set_distances(&mut self, distances: &HashMap<u32, f32>) {
        for (&staff, &d) in distances {
            let staff = self.staves.entry(staff).or_default();
            staff.distance_specified = Some(d);
        }
    }
}
