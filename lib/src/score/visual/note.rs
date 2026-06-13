use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::rect::Rect;
use crate::layout::{Layout, UserLayout};
use crate::score::core::pitch::Pitch;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;

pub struct Note {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub pitch: Pitch,
    pub default_x: f32,
    pub staff_line: i32,

    pub color: Color,
}

impl Note {
    pub fn new(pitch: Pitch, default_x: f32, staff_line: i32) -> Self {
        Note {
            xy: XY::default(),
            width: f32::default(),
            height: f32:: default(),

            pitch,
            default_x,
            staff_line,

            color: Color::default(),
        }
    }
}

impl ScoreElement for Note {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn apply_layout(&mut self, _layout: &Layout, _user_layout: &UserLayout) {
        self.color = _layout.foreground_color;

        if let Some(user_color) = _user_layout.foreground_color {
            self.color = user_color;
        }

        for child in self.children() {
            child.apply_layout(_layout, _user_layout);
        }
    }
}

impl Layoutable for Note {
    fn measure(&mut self, _available: &XY) {
        self.width = 10.;
        self.height = 10.;
    }
    fn arrange(&mut self, _origin: &XY) {
        let d_y = self.staff_line as f32 * 5. - 5.;

        self.xy = _origin.mv(self.default_x, d_y);
    }
}

impl Content for Note {
    fn content(&self) -> Vec<&dyn Content> {
        Vec::new()
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();

        let rect = Rect {
            xy: self.xy,
            width: self.width,
            height: self.height,
            color: self.color,
            ..Default::default()
        };
        result.push(rect.into());

        result
    }
}
