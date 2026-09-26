use crate::geometry::bounding_box::BoundingBox;
use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::core::accidental::Accidental as AccidentalCore;
use crate::score::visual::layoutable::LayoutParams;
use crate::score::visual::staff::Staff;
use crate::smufl::glyphs::accidental::Accidental as SmuflAccidental;

pub struct Accidental {
    pub xy: XY,
    pub width: f32,
    pub height: f32,

    pub scale: f32,
    pub color: Color,

    /// Which accidental this is, settled by the walk without knowing the font.
    pub accidental: AccidentalCore,
    /// `accidental` looked up in the font, written by `resolve_layout`.
    glyph: Option<SmuflAccidental>,
}

impl Accidental {
    pub fn new(accidental: AccidentalCore) -> Self {
        Self {
            xy: XY::ZERO,
            width: 0.,
            height: 0.,

            scale: 1.,
            color: Color::BLACK,

            accidental,
            glyph: None,
        }
    }

    /// The accidental glyph in the font the score is arranged with.
    ///
    /// # Panics
    ///
    /// Before `resolve_layout` has run: the walk that builds the accidental
    /// does not know the font.
    pub fn glyph(&self) -> &SmuflAccidental {
        self.glyph
            .as_ref()
            .expect("accidental glyph read before resolve_layout")
    }

    /// Sets the scale -- a key signature on a reduced staff, a cue note's
    /// accidental. The width and height are not re-derived here: they are the
    /// measure pass's to settle, so a caller that reads them straight after
    /// rescaling has to
    /// [`measure_accidental`](crate::score::visual::arranger::ScoreMeasurement::measure_accidental)
    /// first. The counterpart of
    /// [`Clef::rescale`](crate::score::visual::clef::Clef::rescale).
    pub fn rescale(&mut self, scale: f32) {
        self.scale = scale;
    }

    /// Full world-space bounding box of the glyph.
    pub fn world_bbox(&self) -> BoundingBox {
        self.glyph_bbox(&self.glyph().bbox)
    }

    /// World-space bounding boxes for all active cutouts.
    pub fn world_cutouts(&self) -> Vec<BoundingBox> {
        if let Some(ref cutouts) = self.glyph().cutouts {
            [cutouts.nw, cutouts.ne, cutouts.se, cutouts.sw]
                .into_iter()
                .flatten()
                .map(|c| self.glyph_bbox(&c))
                .collect()
        } else {
            vec![]
        }
    }

    /// Scales a (smufl-like-) normalized bounding box to current position and scale.
    pub fn glyph_bbox(&self, bbox: &BoundingBox) -> BoundingBox {
        let scaled: BoundingBox = BoundingBox {
            xy: bbox.xy.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
            size: bbox.size.scale(Staff::DEFAULT_SPACE_SIZE * self.scale),
        };

        scaled.mv(self.xy.x, self.xy.y)
    }

    /// Right-side cutouts of `self` (NE and SE) in world coordinates.
    fn right_cutouts(&self) -> Vec<BoundingBox> {
        match self.glyph().cutouts {
            Some(ref cutouts) => [cutouts.ne, cutouts.se]
                .into_iter()
                .flatten()
                .map(|c| self.glyph_bbox(&c))
                .collect(),
            _ => vec![],
        }
    }

    /// Left-side cutouts of `self` (NW and SW) in world coordinates.
    fn left_cutouts(&self) -> Vec<BoundingBox> {
        match self.glyph().cutouts {
            Some(ref cutouts) => [cutouts.nw, cutouts.sw]
                .into_iter()
                .flatten()
                .map(|c| self.glyph_bbox(&c))
                .collect(),
            _ => vec![],
        }
    }

    /// Calculates the exact minimal leftward shift needed for `self` to clear `other`.
    pub fn required_left_shift(&self, other: &Accidental) -> f32 {
        let box_a = self.world_bbox();
        let box_b = other.world_bbox();

        // 1. Calculate vertical overlap interval [y_min, y_max]
        let y_min = box_a.y_min().max(box_b.y_min());
        let y_max = box_a.y_max().min(box_b.y_max());

        // If no vertical overlap, no shift required
        if y_min >= y_max {
            return 0.0;
        }

        // 2. Collect key Y points to divide the overlap into uniform slices
        let a_right_cutouts = self.right_cutouts();
        let b_left_cutouts = other.left_cutouts();

        let mut y_points = vec![y_min, y_max];
        for c in a_right_cutouts.iter().chain(b_left_cutouts.iter()) {
            if c.y_min() > y_min && c.y_min() < y_max {
                y_points.push(c.y_min());
            }
            if c.y_max() > y_min && c.y_max() < y_max {
                y_points.push(c.y_max());
            }
        }

        y_points.sort_by(|p1, p2| p1.partial_cmp(p2).unwrap());
        y_points.dedup_by(|p1, p2| (*p1 - *p2).abs() < 1e-4);

        let mut max_needed_shift: f32 = 0.0;

        // 3. Evaluate each slice
        for window in y_points.windows(2) {
            let slice_y_min = window[0];
            let slice_y_max = window[1];

            // A's solid rightmost X in this slice
            let mut a_solid_right = box_a.x_max();
            for cutout in &a_right_cutouts {
                if cutout.y_min() <= slice_y_min && cutout.y_max() >= slice_y_max {
                    a_solid_right = a_solid_right.min(cutout.x_min());
                }
            }

            // B's solid leftmost X in this slice
            let mut b_solid_left = box_b.x_min();
            for cutout in &b_left_cutouts {
                if cutout.y_min() <= slice_y_min && cutout.y_max() >= slice_y_max {
                    b_solid_left = b_solid_left.max(cutout.x_max());
                }
            }

            // Minimal shift for this slice
            let slice_shift = a_solid_right - b_solid_left;
            if slice_shift > max_needed_shift {
                max_needed_shift = slice_shift;
            }
        }

        max_needed_shift
    }
}

impl Accidental {
    pub fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        self.glyph = Some(params.font.accidental(self.accidental));
        self.color = params.foreground_color();
    }
}
