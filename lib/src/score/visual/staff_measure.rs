use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::score::visual::layoutable::Layoutable;
use crate::visual::clef::Clef;
use crate::visual::element::ScoreElement;
use crate::visual::staff::Staff;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,

    pub prepare_clef_change: Option<Clef>,
}

impl StaffMeasure {
    fn arrange_clef(&mut self) {
        if let Some(ref mut clef) = self.prepare_clef_change {
            let clef_origin = self.xy;
            let measure_right = clef_origin.mv(self.width, 0.);
            let arrange_left = measure_right.mv(-5. - clef.width, 0.);

            let dy = clef.clef.line as f32 * Staff::DEFAULT_SPACE_SIZE / 2. * self.scale;

            let origin = arrange_left.mv(0., dy);

            clef.arrange(&origin);

            clef.scale = self.scale * 0.8;
        }
    }
}

impl ScoreElement for StaffMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut res: Vec<&mut dyn ScoreElement> = Vec::new();

        if let Some(ref mut clef) = self.prepare_clef_change {
            res.push(clef);
        }

        res
    }
}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        self.arrange_clef();
    }
}

impl DrawableContent for StaffMeasure {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let mut result: Vec<&dyn DrawableContent> = Vec::new();

        if let Some(ref clef) = self.prepare_clef_change {
            result.push(clef);
        }

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
