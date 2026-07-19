use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::score::rebeam_strategy::RebeamStrategy;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::part_group::PartGroup;
use crate::score::visual::section_measure::SectionMeasure;
use crate::visual::bracket::Bracket;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

pub struct Section {
    pub part_groups: BTreeMap<u32, PartGroup>,
    pub measures: BTreeMap<u32, SectionMeasure>,

    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub bracket: Bracket,
}

impl Section {
    pub fn new(bracket: Bracket) -> Section {
        Section {
            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            part_groups: Default::default(),
            measures: Default::default(),

            bracket,
        }
    }
}

impl Section {
    fn first_visible_staff_distance(&self) -> f32 {
        self.part_groups
            .values()
            .next()
            .map(|group| group.first_visible_staff_distance())
            .unwrap_or(0.0)
    }

    pub fn rebeam(&mut self, strategy: &dyn RebeamStrategy) {
        for part_group in self.part_groups.values_mut() {
            part_group.rebeam(strategy);
        }
    }

    pub fn shows_bracket(&self) -> bool {
        self.part_groups.len() > 1
    }
}

impl ScoreElement for Section {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let shows_bracket = self.shows_bracket();

        let mut result: Vec<&mut dyn ScoreElement> = Vec::new();

        for (_idx, staff) in self.part_groups.iter_mut() {
            result.push(staff);
        }

        for (_idx, measure) in self.measures.iter_mut() {
            result.push(measure);
        }

        if shows_bracket {
            let bracket: &mut Bracket = &mut self.bracket;
            result.push(bracket);
        }

        result
    }
}

impl Layoutable for Section {
    fn measure(&mut self, _: &XY) {
        self.width = 0.;
        self.height = 0.;

        for (_idx, pg) in self.part_groups.iter_mut() {
            let available = XY::INFINITE;
            pg.measure(&available);
            self.height += pg.height;
        }

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let staves_height = self.height - first_visible_staff_distance;

        for (_idx, measure) in self.measures.iter_mut() {
            let available = XY {
                x: f32::INFINITY,
                y: staves_height,
            };
            measure.measure(&available);
            self.width += measure.width;
        }

        if self.shows_bracket() {
            let avail = XY {
                x: self.width,
                y: staves_height,
            };
            self.bracket.measure(&avail);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        let first_visible_staff_distance = self.first_visible_staff_distance();
        let mut _origin = self.xy.mv(0., first_visible_staff_distance);

        for (_idx, measure) in self.measures.iter_mut() {
            measure.arrange(&_origin);
            _origin = _origin.mv(measure.width, 0.);
        }

        let mut _origin = self.xy;
        for (_idx, part_group) in self.part_groups.iter_mut() {
            part_group.arrange(&_origin);
            _origin = _origin.mv(0., part_group.height);
        }

        let _origin = self.xy.mv(-10., first_visible_staff_distance);
        self.bracket.arrange(&_origin);
    }
}

impl DrawableContent for Section {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let mut result = Vec::new();

        result.extend(self.measures.values().map(|m| m as &dyn DrawableContent));

        result.extend(self.part_groups.values().map(|s| s as &dyn DrawableContent));

        if self.shows_bracket() {
            result.push(&self.bracket);
        }

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
