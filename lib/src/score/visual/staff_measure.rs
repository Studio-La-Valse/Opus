use crate::core::xy::XY;
use crate::drawable::drawable_content::DrawableContent;
use crate::drawable::drawable_element::DrawableElement;
use crate::drawable::layoutable::Layoutable;
use crate::score::visual::time_signature::TimeSignature;
use crate::visual::clef::Clef;
use crate::visual::rest::Rest;
use crate::visual::score_element::ScoreElement;
use crate::visual::staff::Staff;
use crate::visual::staff_ctx::StaffCtx;

#[derive(Default)]
pub struct StaffMeasure {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,

    pub rests: Vec<Rest>,
    pub time_signature: Option<TimeSignature>,
    pub prepare_time_signature: Option<TimeSignature>,
    pub prepare_clef_change: Option<Clef>,
}

impl StaffMeasure {
    fn arrange_time_signature(&mut self) {
        if let Some(ref mut time_signature) = self.time_signature {
            let pos = self.xy.mv(5., 0.);
            time_signature.arrange(&pos);
        }

        if let Some(ref mut prepare_time_signature) = self.prepare_time_signature {
            let pos = self
                .xy
                .mv(self.width - prepare_time_signature.width - 5., 0.);
            prepare_time_signature.arrange(&pos);
        }
    }
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
    fn arrange_rests(&mut self) {
        let staff_ctx = StaffCtx {
            hidden: false,
            distance_from_top: 0.,
            scaling: self.scale,
        };
        for rest in self.rests.iter_mut() {
            let dx: f32 = if rest.is_measure {
                self.width / 2.
            } else {
                rest.default_x.unwrap()
            };

            let glyph_origin = self.xy.mv(dx, 0.);
            rest.arrange_ctx(&glyph_origin, &staff_ctx);
        }
    }
}

impl ScoreElement for StaffMeasure {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut res: Vec<&mut dyn ScoreElement> = Vec::new();

        if let Some(ref mut time_signature) = self.time_signature {
            res.push(time_signature);
        }

        if let Some(ref mut prepare_time_signature) = self.prepare_time_signature {
            res.push(prepare_time_signature);
        }

        if let Some(ref mut clef) = self.prepare_clef_change {
            res.push(clef);
        }

        for rest in self.rests.iter_mut() {
            res.push(rest);
        }

        res
    }
}

impl Layoutable for StaffMeasure {
    fn measure(&mut self, available: &XY) {
        self.height = available.y;

        if let Some(ref mut time_signature) = self.time_signature {
            time_signature.measure(available);
        }

        if let Some(ref mut prepare_time_signature) = self.prepare_time_signature {
            prepare_time_signature.measure(available);
        }

        if let Some(ref mut clef) = self.prepare_clef_change {
            clef.measure(available);
        }

        for rest in self.rests.iter_mut() {
            rest.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        self.arrange_time_signature();
        self.arrange_clef();
        self.arrange_rests();
    }
}

impl DrawableContent for StaffMeasure {
    fn content(&self) -> Vec<&dyn DrawableContent> {
        let mut result: Vec<&dyn DrawableContent> = Vec::new();

        if let Some(ref time_signature) = self.time_signature {
            result.push(time_signature);
        }

        if let Some(ref prepare_time_signature) = self.prepare_time_signature {
            result.push(prepare_time_signature);
        }

        if let Some(ref clef) = self.prepare_clef_change {
            result.push(clef);
        }

        for rest in self.rests.iter() {
            result.push(rest);
        }

        result
    }

    fn elements(&self) -> Vec<DrawableElement> {
        vec![]
    }
}
