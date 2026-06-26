use std::collections::BTreeMap;
use crate::color::Color;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::{Layout, UserLayout};
use crate::visual::element::ScoreElement;
use crate::visual::layoutable::Layoutable;
use crate::xy::XY;

#[derive(Default)]
pub enum UpDown {
    #[default]
    Up,
    Down,
}

pub enum BeamType {
    Start,
    Continue,
    End,
    HookStart,
    HookEnd
}

impl From<&str> for BeamType {
    fn from(value: &str) -> BeamType {
        match value {
            "begin" => BeamType::Start,
            "continue" => BeamType::Continue,
            "end" => BeamType::End,
            "hookstart" => BeamType::HookStart,
            "hookend" => BeamType::HookEnd,
            _ => panic!("Unknown beam type {}", value)
        }
    }
}

#[derive(Default)]
pub struct Stem {
    // provided by the musicxml document
    pub default_y: f32,

    // worked out by arranging notes,
    // then calculating from stem anchor to default_y
    pub xy: XY,
    pub length: f32,
    pub thickness: f32,

    pub direction: UpDown,

    pub color: Color,

    pub beams: BTreeMap<u32, BeamType>
}

impl Stem {
    pub fn new(direction: UpDown, default_y: f32) -> Self {
        Self {
            default_y,
            direction,
            ..Default::default()
        }
    }
}

impl ScoreElement for Stem {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn apply_layout(&mut self, _layout: &Layout, _user_layout: &UserLayout) {
        self.thickness = 1.0; // pseudo code
        self.color = _user_layout.foreground_color.unwrap();
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
