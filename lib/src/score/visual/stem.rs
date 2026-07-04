use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::duration::BaseDuration;
use crate::layout::Layout;
use crate::ray::Ray;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::layoutable::Layoutable;
use crate::xy::XY;
use std::collections::BTreeMap;

#[derive(Default, Eq, PartialEq, Copy, Clone, Debug)]
pub enum UpDown {
    #[default]
    Up,
    Down,
}

impl UpDown {
    pub fn invert(self) -> Self {
        match self {
            UpDown::Up => UpDown::Down,
            UpDown::Down => UpDown::Up,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum BeamType {
    Start,
    Continue,
    End,
    HookStart,
    HookEnd,
}

impl From<&str> for BeamType {
    fn from(value: &str) -> BeamType {
        match value {
            "begin" => BeamType::Start,
            "continue" => BeamType::Continue,
            "end" => BeamType::End,
            "hookstart" => BeamType::HookStart,
            "hookend" => BeamType::HookEnd,
            _ => panic!("Unknown beam type: '{}'", value),
        }
    }
}

pub struct Stem {
    // provided by the musicxml document
    pub default_y: Option<f32>,

    // worked out by arranging notes,
    // then calculating from stem anchor to default_y
    pub xy: XY,
    pub length: f32,
    pub thickness: f32,

    pub direction: UpDown,

    pub color: Color,
    pub staff: u32,

    pub duration: BaseDuration,
    pub beams: BTreeMap<u32, BeamType>,
}

impl Stem {
    pub fn new(
        direction: UpDown,
        duration: BaseDuration,
        staff: u32,
        default_y: Option<f32>,
    ) -> Self {
        Self {
            default_y,
            direction,
            duration,
            staff,
            length: 0.,
            thickness: 0.,
            xy: XY::ZERO,
            color: Color::TRANSPARENT,
            beams: BTreeMap::new(),
        }
    }

    pub fn tip(&self) -> XY {
        self.xy.mv(0., self.length)
    }

    pub fn attach_ray(&mut self, ray: &Ray) {
        let dir = match self.direction {
            UpDown::Up => -1.,
            UpDown::Down => 1.,
        };
        let other = Ray {
            origin: self.xy,
            dir: XY { x: 0., y: dir },
        };
        let intersection = ray.intersect(other).unwrap();

        self.length = intersection.y - self.xy.y;
    }
}

impl ScoreElement for Stem {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn _apply_layout(
        &mut self,
        layout: &Layout,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.thickness = user_layout
            .stem_thickness
            .or(layout.appearance.stem_thickness)
            .unwrap_or(app_defaults.stem_thickness);
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Stem {
    fn measure(&mut self, _available: &XY) {}

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for Stem {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        let mut elements: Vec<Element> = Vec::new();

        let canvas_offset = match self.direction {
            UpDown::Down => self.thickness / 2.,
            UpDown::Up => -self.thickness / 2.,
        };

        let stem: Element = Line {
            start: self.xy.mv(canvas_offset, 0.),
            end: self.xy.mv(canvas_offset, self.length),
            stroke_color: self.color,
            stroke_width: self.thickness,
        }
        .into();

        elements.push(stem);

        elements
    }
}
