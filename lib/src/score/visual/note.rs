use crate::bounding_box::BoundingBox;
use crate::color::Color;
use crate::core::xy::XY;
use crate::drawable::content::Content;
use crate::drawable::element::Element;
use crate::drawable::elements::line::Line;
use crate::drawable::elements::rect::Rect;
use crate::layout::{Layout, UserLayout};
use crate::score::core::pitch::Pitch;
use crate::score::visual::layoutable::Layoutable;
use crate::smufl::glyphs::notehead_black::NoteheadBlack;
use crate::smufl::smufl_glyph::SmuflGlyph;
use crate::visual::element::ScoreElement;
use crate::visual::stem::Stem;

pub struct Note {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub pitch: Pitch,
    pub default_x: f32,
    pub staff_line: i32,
    pub staff: u32,

    pub color: Color,

    pub glyph: Option<NoteheadBlack>,
    pub stem: Option<Stem>,
}

impl Note {
    pub fn new(pitch: Pitch, default_x: f32, staff: u32, staff_line: i32) -> Self {
        Note {
            xy: XY::default(),
            width: f32::default(),
            height: f32::default(),

            pitch,
            default_x,
            staff,
            staff_line,

            glyph: None,
            stem: None,

            color: Color::default(),
        }
    }

    /// Scales a normalized bounding box to current position and scale.
    pub fn scale_box(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled = BoundingBox {
            x_min: bbox.x_min * 10.,
            y_min: bbox.y_min * 10.,
            x_max: bbox.x_max * 10.,
            y_max: bbox.y_max * 10.,
        };

        BoundingBox {
            x_min: scaled.x_min + self.xy.x,
            y_min: scaled.y_min + self.xy.y,
            x_max: scaled.x_max + self.xy.x,
            y_max: scaled.y_max + self.xy.y,
        }
    }

    /// Scales a normalized point to current position and scale.
    pub fn scale_pt(&self, xy: &XY) -> XY {
        let scaled = XY {
            x: xy.x * 10.,
            y: xy.y * 10.,
        };

        XY {
            x: scaled.x + self.xy.x,
            y: scaled.y + self.xy.y,
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

        self.glyph = Some(_user_layout.font.notehead_black())
    }
}

impl Layoutable for Note {
    fn measure(&mut self, _available: &XY) {
        self.height = 10.;

        let glyph = self.glyph.as_ref().unwrap();
        let bbox = self.scale_box(&glyph.bbox);

        self.width = bbox.width();
    }

    // here, origin is the origin of the staff measure.
    fn arrange(&mut self, origin: &XY) {
        let d_y = self.staff_line as f32 * 5.;

        self.xy = origin.mv(self.default_x, d_y);
    }
}

impl Content for Note {
    fn content(&self) -> Vec<&dyn Content> {
        Vec::new()
    }

    fn elements(&self) -> Vec<Element> {
        let mut result: Vec<Element> = Vec::new();

        let glyph = self.glyph.as_ref();

        if let Some(glyph) = glyph {
            let text = glyph.as_text(self.color, self.xy);
            let bbox = self.scale_box(&glyph.bbox);

            let rect = Rect {
                xy: XY {
                    x: bbox.x_min,
                    y: bbox.y_min,
                },
                width: bbox.x_max - bbox.x_min,
                height: bbox.y_max - bbox.y_min,
                color: Color::TRANSPARENT,
                stroke_width: Some(0.25),
                stroke_color: Some(Color {
                    a: 1.,
                    r: 255,
                    g: 0,
                    b: 0,
                }),
            };

            result.push(rect.into());

            let origin = Line {
                start: self.xy,
                end: self.xy.mv(self.width, 0.),
                stroke_color: Color {
                    a: 1.,
                    r: 255,
                    g: 0,
                    b: 0,
                },
                stroke_width: 0.2,
            };
            result.push(origin.into());

            for cutout in [
                glyph.cutouts.nw,
                glyph.cutouts.ne,
                glyph.cutouts.se,
                glyph.cutouts.sw,
            ]
            .into_iter()
            .flatten()
            {
                let bbox = self.scale_box(&cutout);

                let rect = Rect {
                    xy: XY {
                        x: bbox.x_min,
                        y: bbox.y_min,
                    },
                    width: bbox.x_max - bbox.x_min,
                    height: bbox.y_max - bbox.y_min,
                    color: Color {
                        a: 1.,
                        r: 255,
                        g: 0,
                        b: 0,
                    },
                    stroke_width: None,
                    stroke_color: None,
                };

                result.push(rect.into());
            }

            result.push(text.into());
        }

        result
    }
}
