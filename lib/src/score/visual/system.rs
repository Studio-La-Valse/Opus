use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::{Layout, UserLayout};
use crate::layout_ctx::Visibility;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::section::Section;
use crate::score::visual::system_measure::SystemMeasure;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct System {
    pub sections: BTreeMap<u32, Section>,
    pub measures: BTreeMap<u32, SystemMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,

    pub m_left: f32,
    pub m_right: f32,
    pub distance: f32,
    pub top: f32,
}

impl System {
    fn consolidate_measure_widths(&mut self) {
        for (idx, measure) in self.measures.iter_mut() {
            let width = measure.width;

            for (_, section) in self.sections.iter_mut() {
                let measure = section.measures.get_mut(idx).unwrap();
                measure.width = width;

                for (_, part_group) in section.part_groups.iter_mut() {
                    let measure = part_group.measures.get_mut(idx).unwrap();
                    measure.width = width;

                    for (_, part) in part_group.parts.iter_mut() {
                        let measure = part.measures.get_mut(idx).unwrap();
                        measure.width = width;

                        for (_, staff) in part.staves.iter_mut() {
                            let measure = staff.measures.get_mut(idx).unwrap();
                            measure.width = width;
                        }
                    }
                }
            }
        }
    }

    fn handle_first_visible_staff(&mut self) {
        let mut found: bool = false;

        for (_idx, section) in self.sections.iter_mut() {
            if found {
                break;
            }

            for (_idx, part_group) in section.part_groups.iter_mut() {
                if found {
                    break;
                }

                for (_idx, part) in part_group.parts.iter_mut() {
                    if found {
                        break;
                    }

                    if part.visibility == Visibility::Hidden {
                        continue;
                    }

                    for (_idx, staff) in part.staves.iter_mut() {
                        if staff.hidden {
                            continue;
                        }

                        // first visible staff found.
                        staff.distance_final = 0.;
                        found = true;
                        break;
                    }
                }
            }
        }
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for section in self.sections.values_mut() {
            section.rebeam(strategy);
        }
    }
}

impl ScoreElement for System {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.sections.iter_mut() {
            result.push(staff);
        }

        for (_idx, measure) in self.measures.iter_mut() {
            result.push(measure);
        }

        result
    }

    fn apply_layout(&mut self, layout: &Layout, user_layout: &UserLayout) {
        self.color = layout.foreground_color;

        let user_color = user_layout.foreground_color;
        if let Some(user_page_color) = user_color {
            self.color = user_page_color;
        }

        for child in self.children() {
            child.apply_layout(layout, user_layout);
        }
    }
}

impl Layoutable for System {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        self.consolidate_measure_widths();
        self.handle_first_visible_staff();

        for (_idx, section) in self.sections.iter_mut() {
            let available = XY::INFINITE;
            section.measure(&available);
            self.height += section.height;
        }

        for (_idx, measure) in self.measures.iter_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: self.height,
            };
            measure.measure(&available);
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
        for (_idx, section) in self.sections.iter_mut() {
            section.arrange(&_origin);
            _origin = _origin.mv(0., section.height);
        }
    }
}

impl Content for System {
    fn content(&self) -> Vec<&dyn Content> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Content));

        result.extend(self.sections.values().map(|s| s as &dyn Content));

        result
    }

    fn elements(&self) -> Vec<Element> {
        let mut elements: Vec<Element> = Vec::new();

        let stroke_color = self.color;
        let stroke_width = 1.;

        let left_line = Line {
            start: self.xy,
            end: self.xy.mv(0., self.height),
            stroke_width,
            stroke_color,
        };
        elements.push(left_line.into());

        // no right line, that is drawn by section measures.

        elements
    }
}
