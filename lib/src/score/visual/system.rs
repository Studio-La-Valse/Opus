use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::drawable_content::Drawable;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::elements::line::Line;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::layout_ctx::Visibility;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::section::Section;
use crate::score::visual::system_measure::SystemMeasure;
use crate::user_layout::UserLayout;
use crate::visual::bracket::Bracket;
use crate::visual::part::Part;
use crate::visual::part_measure::PartMeasure;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::visual::staff_measure::StaffMeasure;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct System {
    pub sections: BTreeMap<u32, Section>,
    pub measures: BTreeMap<u32, SystemMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
    pub line_width: f32,
    pub staff_line_width: f32,

    pub m_left: f32,
    pub m_right: f32,
    pub distance: f32,
    pub top: f32,
}

impl System {
    pub fn get_section_or_insert<F: FnOnce() -> Bracket>(
        &mut self,
        section_id: u32,
        bracket_factory: F,
    ) -> &mut Section {
        self.sections.entry(section_id).or_insert_with(|| {
            let brace = bracket_factory();
            Section::new(brace)
        })
    }

    pub fn locate_system_measure_mut(&mut self, measure_number: u32) -> Option<&mut SystemMeasure> {
        self.measures.get_mut(&measure_number)
    }

    pub fn locate_part_mut(&mut self, part_id: &str) -> Option<&mut Part> {
        self.sections
            .values_mut()
            .find_map(|pg| pg.locate_part_mut(part_id))
    }

    pub fn locate_part_measure_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Option<&mut PartMeasure> {
        // First check if the system even contains this measure
        if !self.measures.contains_key(&measure_number) {
            return None;
        }

        self.sections
            .values_mut()
            .find_map(|section| section.locate_part_measure_mut(part_id, measure_number))
    }

    pub fn locate_staff_measures_mut(
        &mut self,
        part_id: &str,
        measure_number: u32,
    ) -> Vec<&mut StaffMeasure> {
        if !self.measures.contains_key(&measure_number) {
            return Vec::new();
        }

        self.locate_part_mut(part_id)
            .map(|p| p.locate_staff_measures_mut(measure_number))
            .unwrap_or_default()
    }

    pub fn locate_staff_measure_mut(
        &mut self,
        part_id: &str,
        staff_number: StaffIdx,
        measure_number: u32,
    ) -> Option<&mut StaffMeasure> {
        // First check if the system even contains this measure
        if !self.measures.contains_key(&measure_number) {
            return None;
        }

        self.sections.values_mut().find_map(|section| {
            section.locate_staff_measure_mut(part_id, staff_number, measure_number)
        })
    }

    pub fn consolidate_measure_width(&mut self, measure_number: u32) {
        let measure = self.measures.get_mut(&measure_number).unwrap();
        let width = measure.width;

        for section in self.sections.values_mut() {
            let measure = section.measures.entry(measure_number).or_default();
            measure.width = width;

            for part_group in section.part_groups.values_mut() {
                let measure = part_group.measures.entry(measure_number).or_default();
                measure.width = width;

                for part in part_group.parts.values_mut() {
                    let measure = part.measures.entry(measure_number).or_default();
                    measure.width = width;

                    for staff in part.staves.values_mut() {
                        let measure = staff.measures.entry(measure_number).or_default();
                        measure.width = width;
                    }
                }
            }
        }
    }

    pub fn find_first_visible_staff(&self) -> &Staff {
        for section in self.sections.values() {
            for part_group in section.part_groups.values() {
                for part in part_group.parts.values() {
                    if part.visibility == Visibility::Hidden {
                        continue;
                    }

                    for staff in part.staves.values() {
                        if staff.hidden {
                            continue;
                        }

                        return staff;
                    }
                }
            }
        }

        panic!()
    }

    pub fn find_last_visible_staff(&self) -> &Staff {
        let mut last: Option<&Staff> = None;

        for section in self.sections.values() {
            for part_group in section.part_groups.values() {
                for part in part_group.parts.values() {
                    if part.visibility == Visibility::Hidden {
                        continue;
                    }

                    for staff in part.staves.values() {
                        if staff.hidden {
                            continue;
                        }

                        last = Some(staff);
                    }
                }
            }
        }

        last.unwrap()
    }

    /// Every first visible staff in a system must have a 0-distance to the top of the system.
    fn first_visible_staff(&mut self) -> Option<&mut Staff> {
        for section in self.sections.values_mut() {
            for part_group in section.part_groups.values_mut() {
                for part in part_group.parts.values_mut() {
                    if part.visibility == Visibility::Hidden {
                        continue;
                    }

                    for staff in part.staves.values_mut() {
                        if staff.hidden {
                            continue;
                        }

                        // first visible staff found.
                        return Some(staff);
                    }
                }
            }
        }

        None
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

        for staff in self.sections.values_mut() {
            result.push(staff);
        }

        for measure in self.measures.values_mut() {
            result.push(measure);
        }

        result
    }

    fn _apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);

        self.line_width = user_layout
            .light_barline
            .or(layout.appearance.light_barline)
            .unwrap_or(app_defaults.staff_line_thickness);

        self.staff_line_width = user_layout
            .staff
            .or(layout.appearance.staff)
            .unwrap_or(app_defaults.staff_line_thickness);
    }
}

impl Layoutable for System {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        if let Some(staff) = self.first_visible_staff() {
            staff.distance_final = 0.;
        }

        for section in self.sections.values_mut() {
            let available = XY::INFINITE;
            section.measure(&available);
            self.height += section.height;
        }

        for measure in self.measures.values_mut() {
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
        for measure in self.measures.values_mut() {
            measure.arrange(origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for section in self.sections.values_mut() {
            section.arrange(&_origin);
            _origin = _origin.mv(0., section.height);
        }
    }
}

impl Drawable for System {
    fn content(&self) -> Vec<&dyn Drawable> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn Drawable));

        result.extend(self.sections.values().map(|s| s as &dyn Drawable));

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        let mut elements: Vec<DrawableElement> = Vec::new();

        let stroke_color = self.color;
        let stroke_width = self.line_width;

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
