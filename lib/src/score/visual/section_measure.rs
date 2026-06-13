use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::layout::{Layout, UserLayout};
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub color: Color,
}

impl SectionMeasure {}

impl ScoreElement for SectionMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        vec![]
    }

    fn apply_layout(&mut self, layout: &Layout, user_layout: &UserLayout) {
        self.color = layout.foreground_color;

        let user_color = user_layout.foreground_color;
        if let Some(user_page_color) = user_color {
            self.color = user_page_color;
        }

        for child in self.children() {
            child.apply_layout(layout, user_layout);
        }
    }
}

impl Layoutable for SectionMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl Content for SectionMeasure {
    fn content(&self) -> Vec<&dyn Content> {
        vec![]
    }

    fn elements(&self) -> Vec<Element> {
        let mut elements: Vec<Element> = Vec::new();

        let stroke_color = self.color;
        let stroke_width = 1.;

        let right_line = Line {
            start: self.xy.mv(self.width, 0.),
            end: self.xy.mv(self.width, self.height),
            stroke_width,
            stroke_color,
        };
        elements.push(right_line.into());

        elements
    }
}
