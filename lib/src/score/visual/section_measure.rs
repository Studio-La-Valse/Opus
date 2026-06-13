use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::score::visual::layoutable::Layoutable;

#[derive(Default)]
pub struct SectionMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,
}

impl SectionMeasure {}

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

        let stroke_color = Color::BLACK;
        let stroke_width = 1.;

        let right_line = Line { start: self.xy.mv(self.width, 0.), end: self.xy.mv(self.width, self.height), stroke_width, stroke_color };
        elements.push(right_line.into());
        
        elements
    }
}
