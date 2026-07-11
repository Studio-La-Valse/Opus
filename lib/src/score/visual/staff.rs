use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::Layout;
use crate::score::visual::layoutable::Layoutable;
use crate::score::visual::staff_measure::StaffMeasure;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use std::collections::BTreeMap;

pub struct Staff {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub measures: BTreeMap<u32, StaffMeasure>,

    pub color: Color,
    pub line_thickness: f32,

    pub hidden: bool,

    pub scale: f32,

    pub distance_specified: Option<f32>,
    pub distance_final: f32,
}

impl Default for Staff {
    fn default() -> Staff {
        Staff {
            xy: Default::default(),
            width: Default::default(),
            height: Default::default(),

            measures: Default::default(),

            color: Default::default(),
            line_thickness: Default::default(),

            hidden: Default::default(),

            scale: 1.,

            distance_specified: Default::default(),
            distance_final: Default::default(),
        }
    }
}

impl Staff {
    pub const LINES: usize = 5;
    pub const SPACES: usize = Staff::LINES - 1;
    pub const DEFAULT_SPACE_SIZE: f32 = 10.;

    // 5 lines, 4 spaces, 10 tenths for each space according to MusicXML spec.
    pub fn height(&self) -> f32 {
        Staff::SPACES as f32 * self.line_space()
    }

    pub fn line_space(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }

    pub fn set_scale(&mut self, scale: f32) {
        self.scale = scale;
        for measure in self.measures.values_mut() {
            measure.scale = scale;
        }
    }
}

impl ScoreElement for Staff {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        self.measures
            .values_mut()
            .map(|m| m as &mut dyn ScoreElement)
            .collect()
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

        self.line_thickness = user_layout
            .staff
            .or(layout.appearance.staff)
            .unwrap_or(app_defaults.staff_line_thickness);
    }
}

impl Layoutable for Staff {
    fn measure(&mut self, _available: &XY) {
        self.height = self.height();
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
        let stroke_width = self.line_thickness;

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
                y: start.y + self.line_space(),
            };
            end = XY {
                x: end.x,
                y: end.y + self.line_space(),
            };
        }

        elements
    }
}
