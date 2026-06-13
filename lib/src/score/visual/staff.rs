use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::{Layout, UserLayout};
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Staff {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub measures: BTreeMap<u32, StaffMeasure>,

    pub color: Color,

    pub hidden: bool,
    pub distance_specified: Option<f32>,
    pub distance_final: f32,
}

impl Staff {
    // 5 lines, 4 spaces, 10 tenths for each space according to MusicXML spec.
    pub const SIZE: i8 = 40;
}

impl ScoreElement for Staff {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        self.measures
            .values_mut()
            .map(|m| m as &mut dyn ScoreElement)
            .collect()
    }

    fn apply_layout(&mut self, layout: &Layout, user_layout: &UserLayout) {
        self.color = layout.foreground_color;

        let user_color = user_layout.foreground_color;
        if let Some(user_color) = user_color {
            self.color = user_color;
        }

        for child in self.children() {
            child.apply_layout(layout, user_layout);
        }
    }
}

impl Layoutable for Staff {
    fn measure(&mut self, _available: &XY) {
        self.height = f32::from(Self::SIZE);
        self.width = 0.;

        if self.hidden {
            self.height = 0.;
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
            measure.arrange(&_origin);

            _origin = _origin.mv(measure.width, 0.)
        }
    }
}

impl Content for Staff {
    fn content(&self) -> Vec<&dyn Content> {
        self.measures.values().map(|m| m as &dyn Content).collect()
    }

    fn elements(&self) -> Vec<Element> {
        let mut elements: Vec<Element> = Vec::new();

        if self.hidden {
            return elements;
        }

        let mut start = XY {
            x: self.xy.x,
            y: self.xy.y,
        };
        let mut end = XY {
            x: self.xy.x + self.width,
            y: self.xy.y,
        };

        let stroke_color = self.color;
        let stroke_width = 1.;

        for _i in 0..5 {
            let line = Line {
                start,
                end,
                stroke_color,
                stroke_width,
            };

            elements.push(line.into());

            start = XY {
                x: start.x,
                y: start.y + 10.,
            };
            end = XY {
                x: end.x,
                y: end.y + 10.,
            };
        }

        elements
    }
}
