use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::layoutable::Layoutable;
use crate::layout::Layout;
use crate::ray::Ray;
use crate::score::core::duration_base::BaseDuration;
use crate::score::core::staff_idx::StaffIdx;
use crate::smufl::glyphs::flag::Flag;
use crate::user_layout::UserLayout;
use crate::visual::score_element::ScoreElement;
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

impl TryFrom<&str> for UpDown {
    type Error = String;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        match text {
            "up" => Ok(UpDown::Up),
            "down" => Ok(UpDown::Down),
            _ => Err(format!("Unknown direction: {text}")),
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
            "backward hook" => BeamType::HookEnd,
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

    pub scale: f32,

    pub direction: UpDown,

    pub color: Color,
    pub staff: StaffIdx,

    pub duration: BaseDuration,

    // TODO: flags and beams are mutually exclusive, should fix somehow
    pub beams: BTreeMap<u32, BeamType>,
    pub flag: Option<Flag>,
}

impl Stem {
    pub fn new(
        direction: UpDown,
        duration: BaseDuration,
        staff: StaffIdx,
        scale: f32,
        default_y: Option<f32>,
    ) -> Self {
        Self {
            default_y,
            direction,
            duration,
            staff,
            scale,
            length: 0.,
            thickness: 0.,
            xy: XY::ZERO,
            color: Color::TRANSPARENT,
            beams: BTreeMap::new(),
            flag: None,
        }
    }

    pub fn tip(&self) -> XY {
        self.xy.mv(0., self.length)
    }

    pub fn nw(&self) -> XY {
        let thickness = self.thickness * self.scale;
        match self.direction {
            UpDown::Up => self.tip().mv(thickness / -2., 0.),
            UpDown::Down => self.xy.mv(-thickness / -2., 0.),
        }
    }

    pub fn ne(&self) -> XY {
        let thickness = self.thickness * self.scale;
        self.nw().mv(thickness, 0.)
    }

    pub fn se(&self) -> XY {
        self.ne().mv(0., self.length)
    }

    pub fn sw(&self) -> XY {
        self.nw().mv(0., self.length)
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
        let thickness = self.thickness * self.scale;
        let canvas_offset = match self.direction {
            UpDown::Down => thickness / 2.,
            UpDown::Up => -thickness / 2.,
        };

        self.xy = XY {
            x: origin.x + canvas_offset,
            y: origin.y,
        }
    }
}
