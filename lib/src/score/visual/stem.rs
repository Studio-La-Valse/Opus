use crate::geometry::color::Color;
use crate::geometry::ray::Ray;
use crate::geometry::xy::XY;
use crate::score::core::duration_base::BaseDuration;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::visual::flag::Flag;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};
use crate::score::visual::note_scale::NoteScale;
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
            "forward hook" => BeamType::HookStart,
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

    /// What this stem's size is derived from -- the same [`NoteScale`] as the
    /// noteheads it carries.
    pub size: NoteScale,
    /// The factor `size` resolved to, written by `resolve_layout` and so only
    /// meaningful after `measure`.
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
        size: NoteScale,
        default_y: Option<f32>,
    ) -> Self {
        Self {
            default_y,
            direction,
            duration,
            staff,
            size,
            scale: size.content_scale,
            length: 0.,
            thickness: 0.,
            xy: XY::ZERO,
            color: Color::TRANSPARENT,
            beams: BTreeMap::new(),
            flag: None,
        }
    }

    pub fn tail(&self) -> XY {
        self.xy
    }

    pub fn tip(&self) -> XY {
        let length = self.length.abs();

        match self.direction {
            UpDown::Up => self.tail().mv(0., -length),
            UpDown::Down => self.tail().mv(0., length),
        }
    }

    pub fn nw(&self) -> XY {
        let thickness = self.thickness * self.scale;
        match self.direction {
            UpDown::Up => self.tip().mv(thickness / -2., 0.),
            UpDown::Down => self.tail().mv(thickness / -2., 0.),
        }
    }

    pub fn ne(&self) -> XY {
        let thickness = self.thickness * self.scale;
        self.nw().mv(thickness, 0.)
    }

    pub fn se(&self) -> XY {
        let length = self.length.abs();
        self.ne().mv(0., length)
    }

    pub fn sw(&self) -> XY {
        let length = self.length.abs();
        self.nw().mv(0., length)
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

    /// Positions the flag (if any) against this stem's own terminal corner.
    /// Must be called once `length` is finalized - stem geometry is only
    /// complete once the caller (`Chord::arrange_stem`) has computed and set
    /// `length`, so this can't happen inside `arrange`, which runs before
    /// `length` is known.
    pub fn arrange_flag(&mut self) {
        let anchor = match self.direction {
            UpDown::Up => self.nw(),
            UpDown::Down => self.sw(),
        };

        if let Some(flag) = self.flag.as_mut() {
            flag.arrange(&anchor);
        }
    }
}

impl Stem {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            score_defaults,
            user_layout,
            app_defaults,
            ..
        } = params;

        self.scale = self.size.resolve(params);

        self.thickness = user_layout
            .stem_thickness
            .or(score_defaults.appearance.stem_thickness)
            .unwrap_or(app_defaults.stem_thickness);
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Stem {
    fn measure(&mut self, available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);

        if let Some(flag) = self.flag.as_mut() {
            flag.measure(available, params);
        }
    }

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
