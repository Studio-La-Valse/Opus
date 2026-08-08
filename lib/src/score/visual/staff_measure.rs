use crate::core::xy::XY;
use crate::drawable::layoutable::Layoutable;
use crate::score::visual::time_signature::TimeSignature;
use crate::visual::clef::Clef;
use crate::visual::key_signature::KeySignature;
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

    // Clef left side of measure
    pub clef_start: Option<Clef>,
    // Key signature at left side of measure
    pub key_signature_start: KeySignature,
    // Time signature left side of measure
    pub time_signature_start: Option<TimeSignature>,
    // Time signature right of measure
    pub time_signature_end: Option<TimeSignature>,
    // Clef right of measure
    pub clef_end: Option<Clef>,
}

impl StaffMeasure {
    pub fn line_space(&self) -> f32 {
        Staff::DEFAULT_SPACE_SIZE * self.scale
    }

    fn arrange_clef_start(&mut self) {
        let line_space = self.line_space() / 2.;
        if let Some(ref mut clef) = self.clef_start {
            let dy: f32 = clef.clef.line as f32 * line_space;
            let dx = 10. * self.scale;
            clef.arrange(&self.xy.mv(dx, dy));
        }
    }
    fn arrange_key_signature_start(&mut self) {
        let mut pos = self.xy.mv(5., 0.);

        if let Some(clef) = &self.clef_start {
            pos = pos.mv(clef.width + 15., 0.);
        }

        self.key_signature_start.arrange(&pos);
    }
    fn arrange_time_signature_start(&mut self) {
        if let Some(ref mut time_signature) = self.time_signature_start {
            let mut pos = self.xy.mv(5., 0.);

            if let Some(clef) = &self.clef_start {
                pos = pos.mv(clef.width + 10., 0.);
            }

            pos = pos.mv(self.key_signature_start.width + 5., 0.);

            time_signature.arrange(&pos);
        }
    }
    fn arrange_time_signature_end(&mut self) {
        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            let pos = self
                .xy
                .mv(self.width - prepare_time_signature.width - 5., 0.);
            prepare_time_signature.arrange(&pos);
        }
    }
    fn arrange_clef_end(&mut self) {
        if let Some(ref mut clef) = self.clef_end {
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

        if let Some(ref mut clef) = self.clef_start {
            res.push(clef);
        }

        if let Some(ref mut key_signature) = self.time_signature_start {
            res.push(key_signature);
        }

        let key_sig: &mut dyn ScoreElement = &mut self.key_signature_start;
        res.push(key_sig);

        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            res.push(prepare_time_signature);
        }

        if let Some(ref mut clef) = self.clef_end {
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

        if let Some(ref mut time_signature) = self.time_signature_start {
            time_signature.measure(available);
        }

        self.key_signature_start.measure(available);

        if let Some(ref mut prepare_time_signature) = self.time_signature_end {
            prepare_time_signature.measure(available);
        }

        if let Some(ref mut clef) = self.clef_end {
            clef.measure(available);
        }

        for rest in self.rests.iter_mut() {
            rest.measure(available);
        }
    }

    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;

        self.arrange_clef_start();
        self.arrange_key_signature_start();
        self.arrange_time_signature_start();
        self.arrange_time_signature_end();
        self.arrange_clef_end();
        self.arrange_rests();
    }
}
