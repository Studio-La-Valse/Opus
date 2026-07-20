use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::layout_ctx::Visibility;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part::Part;
use crate::score::visual::part_group_measure::PartGroupMeasure;
use crate::visual::brace::Brace;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

pub struct PartGroup {
    pub parts: BTreeMap<String, Part>,
    pub measures: BTreeMap<u32, PartGroupMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub brace: Brace,
}

impl PartGroup {
    pub fn new(brace: Brace) -> Self {
        PartGroup {
            parts: Default::default(),
            measures: Default::default(),

            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            brace,
        }
    }
}

impl PartGroup {
    pub fn first_visible_staff_distance(&self) -> f32 {
        let mut dist = 0.;
        let mut found = false;

        for (_idx, part) in self.parts.iter() {
            if found {
                break;
            }

            if part.visibility == Visibility::Hidden {
                continue;
            }

            dist = part.first_visible_staff_distance();
            found = true;
        }

        dist
    }

    pub fn visible_staves(&self) -> usize {
        let mut count = 0;
        for (_idx, part) in self.parts.iter() {
            if part.visibility == Visibility::Hidden {
                continue;
            }

            for (_idx, staff) in part.staves.iter() {
                if staff.hidden {
                    continue;
                }

                count += 1;
            }
        }

        count
    }

    pub fn shows_brace(&self) -> bool {
        self.parts.len() > 1 && self.visible_staves() > 1
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for part in self.parts.values_mut() {
            part.rebeam(strategy);
        }
    }
}

impl ScoreElement for PartGroup {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let show_brace = self.shows_brace();

        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.parts.iter_mut() {
            result.push(staff);
        }

        for (_idx, measure) in self.measures.iter_mut() {
            result.push(measure);
        }

        if show_brace {
            let brace = &mut self.brace;
            result.push(brace);
        }

        result
    }
}

impl Layoutable for PartGroup {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        let show_brace = self.shows_brace();

        for (_idx, part) in self.parts.iter_mut() {
            let available = XY::INFINITE;
            part.measure(&available);
            self.height += part.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        for (_idx, measure) in self.measures.iter_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(&available);
            self.width += measure.width;
        }

        if show_brace {
            let available = XY {
                x: f32::INFINITY,
                y: staves_height,
            };
            self.brace.measure(&available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let show_brace = self.shows_brace();

        let mut _origin = self.xy;
        for (_idx, measure) in self.measures.iter_mut() {
            measure.arrange(origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for (_idx, part) in self.parts.iter_mut() {
            part.arrange(&_origin);
            _origin = _origin.mv(0., part.height);
        }

        if show_brace {
            let origin = self.xy.mv(-15., first_visible_staff_distance);
            self.brace.arrange(&origin);
        }
    }
}

impl DrawableContent for PartGroup {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let show_brace = self.shows_brace();

        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn DrawableContent));

        result.extend(
            self.parts
                .values()
                .filter(|p| p.visibility != Visibility::Hidden)
                .map(|s| s as &dyn DrawableContent),
        );

        if show_brace {
            result.push(&self.brace);
        }

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
