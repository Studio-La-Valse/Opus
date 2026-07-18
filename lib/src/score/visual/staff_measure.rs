use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::element::ScoreElement;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
}

impl StaffMeasure {}

impl ScoreElement for StaffMeasure {}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}

impl DrawableContent for StaffMeasure {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let result: Vec<&dyn DrawableContent> = Vec::new();

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
