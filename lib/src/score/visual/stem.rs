use crate::app_defaults::AppDefaults;
use crate::color::Color;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::duration::BaseDuration;
use crate::layout::Layout;
use crate::ray::Ray;
use crate::score::core::staff_idx::StaffIdx;
use crate::smufl::glyphs::flag::Flag;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::user_layout::UserLayout;
use crate::visual::element::ScoreElement;
use crate::visual::layoutable::Layoutable;
use crate::visual::staff::Staff;
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
        match self.direction {
            UpDown::Up => self.tip().mv(self.thickness / -2., 0.),
            UpDown::Down => self.xy.mv(self.thickness / -2., 0.),
        }
    }

    pub fn ne(&self) -> XY {
        self.nw().mv(self.thickness, 0.)
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
        let canvas_offset = match self.direction {
            UpDown::Down => self.thickness / 2.,
            UpDown::Up => -self.thickness / 2.,
        };

        self.xy = XY {
            x: origin.x + canvas_offset,
            y: origin.y,
        }
    }
}

impl Content for Stem {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        let mut elements: Vec<Element> = Vec::new();

        let stem: Element = Line {
            start: self.xy,
            end: self.tip(),
            stroke_color: self.color,
            stroke_width: self.thickness,
        }
        .into();

        elements.push(stem);

        if let Some(flag) = &self.flag {
            let stem_anchor = match self.direction {
                UpDown::Up => self.nw(),
                UpDown::Down => self.sw(),
            };
            elements.push(display(&stem_anchor, &2., &Color::RED).into());

            let flag_anchor = flag.stem_anchor;
            let flag_anchor = scale_pt(&flag_anchor, &stem_anchor, self.scale);
            elements.push(display(&flag_anchor, &2.5, &Color::GREEN).into());

            let delta = stem_anchor - flag_anchor;

            let final_anchor = stem_anchor + delta;
            elements.push(display(&final_anchor, &3., &Color::BLUE).into());

            let flag: Element = flag.as_text(self.color, final_anchor, self.scale).into();

            elements.push(flag);
        }

        elements
    }
}

fn display(xy: &XY, size: &f32, color: &Color) -> Rect {
    Rect {
        xy: XY {
            x: xy.x - size / 2.,
            y: xy.y - size / 2.,
        },
        width: *size,
        height: *size,
        color: Color::TRANSPARENT,
        stroke_color: Some(*color),
        stroke_width: Some(0.25),
    }
}

/// Scales a normalized point to current position and scale.
fn scale_pt(flag_anchor: &XY, stem_anchor: &XY, scale: f32) -> XY {
    let scaled = XY {
        x: flag_anchor.x * (Staff::DEFAULT_SPACE_SIZE * scale),
        y: flag_anchor.y * (Staff::DEFAULT_SPACE_SIZE * scale),
    };

    scaled + *stem_anchor
}
