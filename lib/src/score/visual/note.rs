use crate::drawable::layoutable::Layoutable;
use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::app_defaults::AppDefaults;
use crate::score::core::staff_idx::StaffIdx;
use crate::score::score_defaults::ScoreDefaults;
use crate::score::user_layout::UserLayout;
use crate::score::visual::accidental::Accidental;
use crate::score::visual::score_element::ScoreElement;
use crate::score::visual::staff::Staff;
use crate::score::visual::staff_ctx::StaffCtx;
use crate::smufl::glyphs::notehead::Notehead;

pub struct Note {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub default_x: f32,
    pub staff_line: i32,
    pub staff: StaffIdx,

    pub scale: f32,

    pub color: Color,

    pub glyph: Notehead,
    pub accidental: Option<Accidental>,
}

impl Note {
    pub fn new(
        glyph: Notehead,
        default_x: f32,
        staff: StaffIdx,
        staff_line: i32,
        scale: f32,
    ) -> Self {
        Note {
            glyph,
            accidental: None,

            default_x,
            staff,
            staff_line,

            scale,

            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            color: Color::default(),
        }
    }

    pub fn arrange_ctx(&mut self, origin: &XY, staff_ctx: &StaffCtx) {
        let staff_top = origin.mv(0., staff_ctx.distance_from_top);
        let note_dy =
            self.staff_line as f32 * ((Staff::DEFAULT_SPACE_SIZE / 2.) * staff_ctx.scaling);
        let note_top = staff_top.mv(0., note_dy);
        self.xy = note_top.mv(self.default_x, 0.);

        self.arrange_accidental();
    }

    fn arrange_accidental(&mut self) {
        if let Some(accidental) = &mut self.accidental {
            accidental.arrange(&self.xy.mv(-2., 0.));
        }
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled: BoundingBox = BoundingBox {
            xy: bbox.xy.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
            size: bbox.size.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }

    /// Scales a normalized point to current position and scale.
    pub fn scale_pt(&self, xy: &XY) -> XY {
        let scaled = XY {
            x: xy.x * (Staff::DEFAULT_SPACE_SIZE * self.scale),
            y: xy.y * (Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }
}

impl ScoreElement for Note {
    fn children(&mut self) -> Vec<&mut dyn ScoreElement> {
        let mut children: Vec<&mut dyn ScoreElement> = Vec::new();

        if let Some(accidental) = &mut self.accidental {
            children.push(accidental);
        }

        children
    }

    fn _apply_layout(
        &mut self,
        _layout: &ScoreDefaults,
        user_layout: &UserLayout,
        app_defaults: &AppDefaults,
    ) {
        self.color = user_layout
            .foreground_color
            .unwrap_or(app_defaults.foreground_color);
    }
}

impl Layoutable for Note {
    fn measure(&mut self, _available: &XY) {
        self.height = Staff::DEFAULT_SPACE_SIZE * self.scale;

        let glyph = &self.glyph;
        let bbox = self.scale_box(&glyph.bbox);

        self.width = bbox.width();

        if let Some(accidental) = &mut self.accidental {
            accidental.measure(_available);
        }
    }

    /// here, origin is the origin of the part measure. Get the dy from the staff ctx.
    fn arrange(&mut self, _origin: &XY) {
        todo!("Use arrange_ctx instead")
    }
}
