use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::score::layout_ctx::Visibility;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part_measure::PartMeasure;
use crate::score::visual::staff::Staff;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct Part {
    pub measures: HashMap<u32, PartMeasure>,
    pub staves: HashMap<u32, Staff>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

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

    pub fn set_distances(&mut self, distances: &HashMap<u32, f32>, default: &f32) {
        for (&idx, &dist) in distances.iter() {
            let staff = self.staves.entry(idx).or_default();
            staff.distance_specified = Some(dist);
        }

        for (&idx, staff) in self.staves.iter_mut() {
            let mut distance: f32 = 0.;

            if idx > 1 {
                distance = staff.distance_specified.unwrap_or(*default);
            }

            staff.distance_final = distance;
        }
    }
}

impl Layoutable for Part {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        if let Visibility::Hidden = self.visibility {
            return;
        }

        for (_idx, staff) in self.staves.iter_mut() {
            let available = &XY::INFINITE;
            // a staff knows its own size (sum of measure widths, staff height)
            staff.measure(available);
            self.height += staff.height;
            self.height += staff.distance_final;
        }

        for (_idx, measure) in self.measures.iter_mut() {
            let available = &XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(available);
            self.width += measure.width;
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let mut _origin = self.xy;
        for (_idx, measure) in self.measures.iter_mut() {
            measure.arrange(origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for (_idx, staff) in self.staves.iter_mut() {
            _origin.mv(0., staff.distance_final);

            staff.arrange(&_origin);
            _origin.mv(0., staff.height);
        }
    }
}

impl Content for Part {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Content));

        result.extend(self.staves.values().map(|s| s as &dyn Content));

        result
    }

    fn elements(&self) -> Vec<Element> {
        vec![]
    }
}
