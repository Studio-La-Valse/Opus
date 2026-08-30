use crate::geometry::color::Color;
use crate::geometry::xy::XY;
use crate::score::visual::layoutable::{LayoutParams, Layoutable};

/// A single augmentation dot belonging to a [`Note`](super::note::Note) or
/// [`Rest`](super::rest::Rest). Notes and rests own a `Vec<Dot>` with one entry
/// per dot; the owner computes each dot's centre during its arrange pass
/// (horizontal spacing, plus the half-space vertical nudge that keeps a dot off
/// a staff line) and hands it here through [`arrange`](Layoutable::arrange).
pub struct Dot {
    pub xy: XY,

    pub scale: f32,
    pub color: Color,

    /// Resolved dot radius in world units (tenths).
    pub radius: f32,
}

impl Dot {
    pub fn new(scale: f32) -> Self {
        Self {
            xy: XY::ZERO,

            scale,
            color: Color::BLACK,

            radius: 0.,
        }
    }
}

impl Dot {
    fn resolve_layout(&mut self, params: LayoutParams<'_>) {
        let LayoutParams {
            user_layout,
            app_defaults,
            ..
        } = params;

        self.color = user_layout
            .dot_color
            .or(user_layout.foreground_color)
            .unwrap_or(app_defaults.foreground_color);
        self.radius = user_layout.dot_radius.unwrap_or(app_defaults.dot_radius) * self.scale;
    }
}

impl Layoutable for Dot {
    fn measure(&mut self, _available: &XY, params: LayoutParams<'_>) {
        self.resolve_layout(params);
    }

    /// `origin` is the fully-resolved centre of this dot, worked out by the
    /// owning note or rest.
    fn arrange(&mut self, origin: &XY) {
        self.xy = *origin;
    }
}
